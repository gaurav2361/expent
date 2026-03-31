use sea_orm::*;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, FixedOffset, Datelike, Utc, Duration};
use std::collections::HashMap;
use std::str::FromStr;

pub mod entities;

#[derive(Debug, Serialize, Deserialize)]
pub struct LineItem {
    pub name: String,
    pub quantity: i32,
    pub price: Decimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OcrResult {
    pub raw_text: String,
    pub amount: Option<Decimal>,
    pub date: Option<DateTime<FixedOffset>>,
    pub upi_id: Option<String>,
    #[serde(default)]
    pub items: Vec<LineItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SplitDetail {
    pub receiver_email: String,
    pub amount: Decimal,
}

pub struct SmartMerge;

impl SmartMerge {
    pub async fn process_ocr(
        db: &DatabaseConnection,
        user_id: &str,
        ocr_data: OcrResult,
    ) -> Result<entities::transaction::Model, DbErr> {
        let start_date = ocr_data.date.map(|d| d - Duration::hours(48));
        let end_date = ocr_data.date.map(|d| d + Duration::hours(48));

        let mut query = entities::transaction::Entity::find()
            .filter(entities::transaction::Column::UserId.eq(user_id));

        if let Some(amount) = ocr_data.amount {
            query = query.filter(entities::transaction::Column::Amount.eq(amount));
        }

        if let (Some(start), Some(end)) = (start_date, end_date) {
            query = query.filter(entities::transaction::Column::Date.between(start, end));
        }

        let existing_txns = query.all(db).await?;

        let txn = if !existing_txns.is_empty() {
            let existing = existing_txns[0].clone();
            let source = entities::transaction_source::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(existing.id.clone()),
                source_type: Set("OCR_SCREENSHOT_MERGED".to_string()),
                r2_file_url: Set(None),
                raw_metadata: Set(Some(serde_json::to_value(&ocr_data).unwrap())),
            };
            source.insert(db).await?;
            existing
        } else {
            let new_txn = entities::transaction::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                user_id: Set(user_id.to_string()),
                amount: Set(ocr_data.amount.unwrap_or(Decimal::ZERO)),
                direction: Set("OUT".to_string()), 
                date: Set(ocr_data.date.unwrap_or_else(|| Utc::now().into())),
                source: Set("OCR".to_string()),
                status: Set("COMPLETED".to_string()),
                purpose_tag: Set(None),
                group_id: Set(None),
            };

            let result = new_txn.insert(db).await?;

            let metadata = entities::transaction_metadata::ActiveModel {
                transaction_id: Set(result.id.clone()),
                upi_txn_id: Set(ocr_data.upi_id.clone()), 
                app_txn_id: Set(None),
                app_name: Set(None),
                contact_number: Set(None),
            };
            metadata.insert(db).await?;

            let source = entities::transaction_source::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                source_type: Set("OCR_SCREENSHOT".to_string()),
                r2_file_url: Set(None),
                raw_metadata: Set(Some(serde_json::to_value(&ocr_data).unwrap())),
            };
            source.insert(db).await?;
            result
        };

        if !ocr_data.items.is_empty() {
            let purchase = entities::purchase::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(txn.id.clone()),
                vendor: Set("Extracted Vendor".to_string()), 
                total: Set(ocr_data.amount.unwrap_or(Decimal::ZERO)),
                order_id: Set(None),
            };
            let p_result = purchase.insert(db).await?;

            for item in ocr_data.items {
                let p_item = entities::purchase_item::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    purchase_id: Set(p_result.id.clone()),
                    name: Set(item.name),
                    quantity: Set(item.quantity),
                    price: Set(item.price),
                    sku: Set(None),
                };
                p_item.insert(db).await?;
            }
        }

        Ok(txn)
    }

    pub async fn create_p2p_request(
        db: &DatabaseConnection,
        sender_id: &str,
        receiver_email: &str,
        txn_id: &str,
    ) -> Result<entities::p2p_request::Model, DbErr> {
        let txn = entities::transaction::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Transaction not found".to_string()))?;

        let request = entities::p2p_request::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            sender_user_id: Set(sender_id.to_string()),
            receiver_email: Set(receiver_email.to_string()),
            transaction_data: Set(serde_json::to_value(&txn).unwrap()),
            status: Set("PENDING".to_string()),
            linked_txn_id: Set(None),
        };

        request.insert(db).await
    }

    pub async fn split_transaction(
        db: &DatabaseConnection,
        sender_id: &str,
        txn_id: &str,
        splits: Vec<SplitDetail>,
    ) -> Result<Vec<entities::p2p_request::Model>, DbErr> {
        let txn = entities::transaction::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Transaction not found".to_string()))?;

        let mut results = Vec::new();
        for split in splits {
            let request = entities::p2p_request::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                sender_user_id: Set(sender_id.to_string()),
                receiver_email: Set(split.receiver_email),
                transaction_data: Set(serde_json::json!({
                    "amount": split.amount,
                    "date": txn.date,
                    "purpose": format!("Split for {}", txn.purpose_tag.as_deref().unwrap_or("Expense"))
                })),
                status: Set("PENDING".to_string()),
                linked_txn_id: Set(None),
            };
            results.push(request.insert(db).await?);
        }
        Ok(results)
    }

    pub async fn accept_p2p_request(
        db: &DatabaseConnection,
        receiver_id: &str,
        request_id: &str,
    ) -> Result<entities::p2p_request::Model, DbErr> {
        let request = entities::p2p_request::Entity::find_by_id(request_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Request not found".to_string()))?;

        if request.status != "PENDING" && request.status != "GROUP_INVITE" {
            return Err(DbErr::Custom("Request is not pending".to_string()));
        }

        if request.status == "GROUP_INVITE" {
            let metadata: serde_json::Value = serde_json::from_value(request.transaction_data.clone())
                .map_err(|e| DbErr::Custom(format!("Failed to parse invite data: {}", e)))?;
            
            let group_id = metadata["group_id"].as_str()
                .ok_or(DbErr::Custom("Missing group_id in invite".to_string()))?;

            let user_group = entities::user_group::ActiveModel {
                user_id: Set(receiver_id.to_string()),
                group_id: Set(group_id.to_string()),
                role: Set("MEMBER".to_string()),
            };
            user_group.insert(db).await?;

            let mut request: entities::p2p_request::ActiveModel = request.into();
            request.status = Set("APPROVED".to_string());
            return request.update(db).await;
        }

        let original_txn: serde_json::Value = serde_json::from_value(request.transaction_data.clone())
            .map_err(|e| DbErr::Custom(format!("Failed to parse transaction data: {}", e)))?;

        let mirrored_txn = entities::transaction::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(receiver_id.to_string()),
            amount: Set(original_txn["amount"].as_str().and_then(|s| Decimal::from_str(s).ok()).unwrap_or(Decimal::ZERO)),
            direction: Set("IN".to_string()), 
            date: Set(original_txn["date"].as_str().and_then(|s| DateTime::parse_from_rfc3339(s).ok()).map(|d| d.with_timezone(&FixedOffset::east_opt(0).unwrap())).unwrap_or_else(|| Utc::now().into())),
            source: Set("P2P".to_string()),
            status: Set("COMPLETED".to_string()),
            purpose_tag: Set(original_txn["purpose"].as_str().map(|s| s.to_string())),
            group_id: Set(None),
        };

        let result_txn = mirrored_txn.insert(db).await?;

        let mut request: entities::p2p_request::ActiveModel = request.into();
        request.status = Set("MAPPED".to_string());
        request.linked_txn_id = Set(Some(result_txn.id));

        request.update(db).await
    }

    pub async fn detect_subscriptions(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::subscription::Model>, DbErr> {
        let ninety_days_ago = Utc::now() - Duration::days(90);
        let transactions = entities::transaction::Entity::find()
            .filter(entities::transaction::Column::UserId.eq(user_id))
            .filter(entities::transaction::Column::Date.gte(ninety_days_ago))
            .all(db)
            .await?;

        let mut groups: HashMap<(String, Decimal), Vec<DateTime<FixedOffset>>> = HashMap::new();
        for txn in transactions {
            let name = txn.purpose_tag.clone().unwrap_or_else(|| "Unknown".to_string());
            let entry = groups.entry((name, txn.amount)).or_default();
            entry.push(txn.date);
        }

        let mut potential_subs = Vec::new();
        for ((name, amount), mut dates) in groups {
            if dates.len() >= 2 {
                dates.sort();
                
                let mut detected_cycle = None;
                let mut start_date = dates[0];
                let mut last_date = dates.last().unwrap().clone();

                for i in 0..dates.len() - 1 {
                    let diff = (dates[i+1] - dates[i]).num_days();
                    
                    if (6..=8).contains(&diff) {
                        detected_cycle = Some("WEEKLY");
                    } else if (27..=33).contains(&diff) {
                        detected_cycle = Some("MONTHLY");
                    } else if (360..=370).contains(&diff) {
                        detected_cycle = Some("YEARLY");
                    }
                }

                if let Some(cycle) = detected_cycle {
                    let next_charge = match cycle {
                        "WEEKLY" => last_date + Duration::days(7),
                        "YEARLY" => last_date + Duration::days(365),
                        _ => last_date + Duration::days(30),
                    };

                    let sub = entities::subscription::Model {
                        id: uuid::Uuid::now_v7().to_string(),
                        user_id: user_id.to_string(),
                        name: name.clone(),
                        amount,
                        cycle: cycle.to_string(),
                        start_date,
                        next_charge_date: next_charge,
                        detection_keywords: None,
                    };
                    potential_subs.push(sub);
                }
            }
        }

        Ok(potential_subs)
    }

    pub async fn list_transactions(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::transaction::Model>, DbErr> {
        entities::transaction::Entity::find()
            .filter(entities::transaction::Column::UserId.eq(user_id))
            .order_by_desc(entities::transaction::Column::Date)
            .all(db)
            .await
    }

    pub async fn list_pending_p2p_requests(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Vec<entities::p2p_request::Model>, DbErr> {
        entities::p2p_request::Entity::find()
            .filter(entities::p2p_request::Column::ReceiverEmail.eq(email))
            .filter(entities::p2p_request::Column::Status.is_in(["PENDING", "GROUP_INVITE"]))
            .all(db)
            .await
    }

    pub async fn create_group(
        db: &DatabaseConnection,
        user_id: &str,
        name: &str,
        description: Option<String>,
    ) -> Result<entities::group::Model, DbErr> {
        let group = entities::group::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            name: Set(name.to_string()),
            description: Set(description),
            created_at: Set(Utc::now().into()),
        };
        let result = group.insert(db).await?;

        let user_group = entities::user_group::ActiveModel {
            user_id: Set(user_id.to_string()),
            group_id: Set(result.id.clone()),
            role: Set("ADMIN".to_string()),
        };
        user_group.insert(db).await?;

        Ok(result)
    }

    pub async fn invite_to_group(
        db: &DatabaseConnection,
        sender_id: &str,
        receiver_email: &str,
        group_id: &str,
    ) -> Result<entities::p2p_request::Model, DbErr> {
        let group = entities::group::Entity::find_by_id(group_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Group not found".to_string()))?;

        let request = entities::p2p_request::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            sender_user_id: Set(sender_id.to_string()),
            receiver_email: Set(receiver_email.to_string()),
            transaction_data: Set(serde_json::json!({
                "type": "GROUP_INVITE",
                "group_id": group.id,
                "group_name": group.name
            })),
            status: Set("GROUP_INVITE".to_string()),
            linked_txn_id: Set(None),
        };

        request.insert(db).await
    }

    pub async fn list_groups(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::group::Model>, DbErr> {
        entities::group::Entity::find()
            .inner_join(entities::user_group::Entity)
            .filter(entities::user_group::Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    pub async fn list_group_transactions(
        db: &DatabaseConnection,
        group_id: &str,
    ) -> Result<Vec<entities::transaction::Model>, DbErr> {
        entities::transaction::Entity::find()
            .filter(entities::transaction::Column::GroupId.eq(group_id))
            .order_by_desc(entities::transaction::Column::Date)
            .all(db)
            .await
    }
}
