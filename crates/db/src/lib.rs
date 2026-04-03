use crate::entities::enums::{
    GroupRole, P2PRequestStatus, SubscriptionCycle, TransactionDirection, TransactionSource,
    TransactionStatus,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub mod entities;

/// Represents a single line item in a purchase, typically extracted via OCR.
#[derive(Debug, Serialize, Deserialize)]
pub struct LineItem {
    pub name: String,
    pub quantity: i32,
    pub price: Decimal,
}

/// The result of an OCR process, containing raw text and extracted transaction details.
#[derive(Debug, Serialize, Deserialize)]
pub struct OcrResult {
    pub raw_text: String,
    pub amount: Option<Decimal>,
    pub date: Option<DateTime<FixedOffset>>,
    pub upi_id: Option<String>,
    #[serde(default)]
    pub items: Vec<LineItem>,
}

/// Details for splitting a transaction among multiple users.
#[derive(Debug, Serialize, Deserialize)]
pub struct SplitDetail {
    pub receiver_email: String,
    pub amount: Decimal,
}

/// Business logic for merging and processing transaction data.
pub struct SmartMerge;

impl SmartMerge {
    /// Processes OCR data by either merging it with an existing transaction or creating a new one.
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

        let ocr_data_json = serde_json::to_value(&ocr_data)
            .map_err(|e| DbErr::Custom(format!("Failed to serialize OCR data: {}", e)))?;

        let txn = if !existing_txns.is_empty() {
            let existing = existing_txns[0].clone();
            let source = entities::transaction_source::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(existing.id.clone()),
                source_type: Set("OCR_SCREENSHOT_MERGED".to_string()),
                r2_file_url: Set(None),
                raw_metadata: Set(Some(ocr_data_json)),
            };
            source.insert(db).await?;
            existing
        } else {
            let new_txn = entities::transaction::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                user_id: Set(user_id.to_string()),
                amount: Set(ocr_data.amount.unwrap_or(Decimal::ZERO)),
                direction: Set(TransactionDirection::Out),
                date: Set(ocr_data.date.unwrap_or_else(|| Utc::now().into())),
                source: Set(TransactionSource::Ocr),
                status: Set(TransactionStatus::Completed),
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
                raw_metadata: Set(Some(ocr_data_json)),
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

    /// Creates a peer-to-peer (P2P) request for a given transaction.
    pub async fn create_p2p_request(
        db: &DatabaseConnection,
        sender_id: &str,
        receiver_email: &str,
        txn_id: &str,
    ) -> Result<entities::p2p_request::Model, DbErr> {
        let txn = entities::transaction::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

        let txn_json = serde_json::to_value(&txn)
            .map_err(|e| DbErr::Custom(format!("Failed to serialize transaction: {}", e)))?;

        let request = entities::p2p_request::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            sender_user_id: Set(sender_id.to_string()),
            receiver_email: Set(receiver_email.to_string()),
            transaction_data: Set(txn_json),
            status: Set(P2PRequestStatus::Pending),
            linked_txn_id: Set(None),
        };

        request.insert(db).await
    }

    /// Splits a transaction among multiple receivers.
    pub async fn split_transaction(
        db: &DatabaseConnection,
        sender_id: &str,
        txn_id: &str,
        splits: Vec<SplitDetail>,
    ) -> Result<Vec<entities::p2p_request::Model>, DbErr> {
        let txn = entities::transaction::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

        let requests: Vec<entities::p2p_request::ActiveModel> = splits
            .into_iter()
            .map(|split| entities::p2p_request::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                sender_user_id: Set(sender_id.to_string()),
                receiver_email: Set(split.receiver_email),
                transaction_data: Set(serde_json::json!({
                    "amount": split.amount,
                    "date": txn.date,
                    "purpose": format!("Split for {}", txn.purpose_tag.as_deref().unwrap_or("Expense"))
                })),
                status: Set(P2PRequestStatus::Pending),
                linked_txn_id: Set(None),
            })
            .collect();

        if requests.is_empty() {
            return Ok(Vec::new());
        }

        let results = entities::p2p_request::Entity::insert_many(requests)
            .exec_with_returning(db)
            .await?;

        Ok(results)
    }

    /// Accepts a P2P request, potentially creating a mirrored transaction for the receiver.
    pub async fn accept_p2p_request(
        db: &DatabaseConnection,
        receiver_id: &str,
        request_id: &str,
    ) -> Result<entities::p2p_request::Model, DbErr> {
        let request = entities::p2p_request::Entity::find_by_id(request_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Request not found".to_string()))?;

        if request.status != P2PRequestStatus::Pending
            && request.status != P2PRequestStatus::GroupInvite
        {
            return Err(DbErr::Custom("Request is not pending".to_string()));
        }

        if request.status == P2PRequestStatus::GroupInvite {
            let metadata: serde_json::Value =
                serde_json::from_value(request.transaction_data.clone())
                    .map_err(|e| DbErr::Custom(format!("Failed to parse invite data: {}", e)))?;

            let group_id = metadata["group_id"]
                .as_str()
                .ok_or_else(|| DbErr::Custom("Missing group_id in invite".to_string()))?;

            let user_group = entities::user_group::ActiveModel {
                user_id: Set(receiver_id.to_string()),
                group_id: Set(group_id.to_string()),
                role: Set(GroupRole::Member),
            };
            user_group.insert(db).await?;

            let mut request: entities::p2p_request::ActiveModel = request.into();
            request.status = Set(P2PRequestStatus::Approved);
            return request.update(db).await;
        }

        let original_txn: serde_json::Value =
            serde_json::from_value(request.transaction_data.clone())
                .map_err(|e| DbErr::Custom(format!("Failed to parse transaction data: {}", e)))?;

        let mirrored_txn = entities::transaction::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(receiver_id.to_string()),
            amount: Set(original_txn["amount"]
                .as_str()
                .and_then(|s| Decimal::from_str(s).ok())
                .unwrap_or(Decimal::ZERO)),
            direction: Set(TransactionDirection::In),
            date: Set(original_txn["date"]
                .as_str()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d.with_timezone(&FixedOffset::east_opt(0).unwrap()))
                .unwrap_or_else(|| Utc::now().into())),
            source: Set(TransactionSource::P2p),
            status: Set(TransactionStatus::Completed),
            purpose_tag: Set(original_txn["purpose"].as_str().map(|s| s.to_string())),
            group_id: Set(None),
        };

        let result_txn = mirrored_txn.insert(db).await?;

        let mut request: entities::p2p_request::ActiveModel = request.into();
        request.status = Set(P2PRequestStatus::Mapped);
        request.linked_txn_id = Set(Some(result_txn.id));

        request.update(db).await
    }

    /// Detects recurring subscriptions from transaction history over the last 90 days.
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
            let name = txn.purpose_tag.unwrap_or_else(|| "Unknown".to_string());
            let entry = groups.entry((name, txn.amount)).or_default();
            entry.push(txn.date);
        }

        let mut potential_subs = Vec::new();
        for ((name, amount), mut dates) in groups {
            if dates.len() >= 2 {
                dates.sort();

                let mut detected_cycle = None;
                let last_date = *dates.last().unwrap();

                for i in 0..dates.len() - 1 {
                    let diff = (dates[i + 1] - dates[i]).num_days();

                    if (6..=8).contains(&diff) {
                        detected_cycle = Some(SubscriptionCycle::Weekly);
                    } else if (27..=33).contains(&diff) {
                        detected_cycle = Some(SubscriptionCycle::Monthly);
                    } else if (360..=370).contains(&diff) {
                        detected_cycle = Some(SubscriptionCycle::Yearly);
                    }
                }

                if let Some(cycle) = detected_cycle {
                    let next_charge = match cycle {
                        SubscriptionCycle::Weekly => last_date + Duration::days(7),
                        SubscriptionCycle::Yearly => last_date + Duration::days(365),
                        _ => last_date + Duration::days(30),
                    };

                    let sub = entities::subscription::Model {
                        id: uuid::Uuid::now_v7().to_string(),
                        user_id: user_id.to_string(),
                        name: name.clone(),
                        amount,
                        cycle,
                        start_date: dates[0],
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
            role: Set(GroupRole::Admin),
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
            status: Set(P2PRequestStatus::GroupInvite),
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

    pub async fn update_transaction(
        db: &DatabaseConnection,
        user_id: &str,
        txn_id: &str,
        amount: Option<Decimal>,
        date: Option<DateTimeWithTimeZone>,
        purpose_tag: Option<String>,
        status: Option<TransactionStatus>,
    ) -> Result<entities::transaction::Model, DbErr> {
        let txn = entities::transaction::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

        if txn.user_id != user_id {
            return Err(DbErr::Custom("Unauthorized".to_string()));
        }

        let mut txn: entities::transaction::ActiveModel = txn.into();

        if let Some(amt) = amount {
            txn.amount = Set(amt);
        }
        if let Some(dt) = date {
            txn.date = Set(dt);
        }
        if let Some(tag) = purpose_tag {
            txn.purpose_tag = Set(Some(tag));
        }
        if let Some(st) = status {
            txn.status = Set(st);
        }

        txn.update(db).await
    }

    pub async fn delete_transaction(
        db: &DatabaseConnection,
        user_id: &str,
        txn_id: &str,
    ) -> Result<u64, DbErr> {
        let txn = entities::transaction::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

        if txn.user_id != user_id {
            return Err(DbErr::Custom("Unauthorized".to_string()));
        }

        let result = entities::transaction::Entity::delete_by_id(txn_id.to_string())
            .exec(db)
            .await?;

        Ok(result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn benchmark_split_transaction() -> Result<(), DbErr> {
        let db = Database::connect("sqlite::memory:").await?;

        // Setup schema
        let db_backend = db.get_database_backend();
        let schema = Schema::new(db_backend);

        db.execute(db_backend.build(&schema.create_table_from_entity(entities::transaction::Entity))).await?;
        db.execute(db_backend.build(&schema.create_table_from_entity(entities::p2p_request::Entity))).await?;

        // Create a transaction
        let txn_id = uuid::Uuid::now_v7().to_string();
        let txn = entities::transaction::ActiveModel {
            id: Set(txn_id.clone()),
            user_id: Set("user_1".to_string()),
            amount: Set(Decimal::new(100, 0)),
            direction: Set(TransactionDirection::Out),
            date: Set(Utc::now().into()),
            source: Set(TransactionSource::Manual),
            status: Set(TransactionStatus::Completed),
            purpose_tag: Set(Some("Lunch".to_string())),
            group_id: Set(None),
        };
        txn.insert(&db).await?;

        let num_splits = 100;
        let splits: Vec<SplitDetail> = (0..num_splits)
            .map(|i| SplitDetail {
                receiver_email: format!("user_{}@example.com", i),
                amount: Decimal::new(1, 0),
            })
            .collect();

        let start = Instant::now();
        SmartMerge::split_transaction(&db, "user_1", &txn_id, splits).await?;
        let duration = start.elapsed();

        println!("\nBENCHMARK_RESULT: {} splits took {:?}", num_splits, duration);

        Ok(())
    }
}
