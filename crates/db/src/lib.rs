use crate::entities::enums::{
    GroupRole, LedgerTabStatus, LedgerTabType, P2PRequestStatus, SubscriptionCycle,
    TransactionDirection, TransactionSource, TransactionStatus, WalletType,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rust_decimal::Decimal;
use sea_orm::prelude::{DateTimeWithTimeZone, Expr};
use sea_orm::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

pub mod entities;

/// Represents a single line item in a purchase, typically extracted via OCR.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LineItem {
    pub name: String,
    pub quantity: i32,
    pub price: Decimal,
}

/// The result of an OCR process, containing raw text and extracted transaction details.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrResult {
    pub raw_text: String,
    pub vendor: Option<String>,
    pub amount: Option<Decimal>,
    pub date: Option<DateTime<FixedOffset>>,
    pub upi_id: Option<String>,
    #[serde(default)]
    pub items: Vec<LineItem>,
}

/// Specialized extraction for Google Pay screenshots.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GPayExtraction {
    pub amount: Decimal,
    pub direction: String, // "IN" | "OUT"
    pub datetime_str: Option<String>,
    pub status: String,
    pub counterparty_name: String,
    pub counterparty_phone: Option<String>,
    pub counterparty_upi_id: Option<String>,
    pub is_merchant: bool,
    pub upi_transaction_id: Option<String>,
    pub google_transaction_id: Option<String>,
    pub source_bank_account: Option<String>,
}

/// Unified OCR data from the Python worker.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessedOcr {
    pub doc_type: String, // "GPAY" or "GENERIC"
    pub data: serde_json::Value,
    pub r2_key: Option<String>,
}

/// Details for splitting a transaction among multiple users.
#[derive(Debug, Serialize, Deserialize)]
pub struct SplitDetail {
    pub receiver_email: String,
    pub amount: Decimal,
}

/// P2P request with sender's name.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct P2PRequestWithSender {
    #[serde(flatten)]
    pub request: entities::p2p_requests::Model,
    pub sender_name: Option<String>,
}

/// Response for OCR transaction creation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OcrTransactionResponse {
    pub transaction: entities::transactions::Model,
    pub contact_created: bool,
}

/// Business logic for merging and processing transaction data.
pub struct SmartMerge;

impl SmartMerge {
    /// Processes OCR data by either merging it with an existing transaction or creating a new one.
    pub async fn process_ocr(
        db: &DatabaseConnection,
        user_id: &str,
        processed: ProcessedOcr,
    ) -> Result<OcrTransactionResponse, DbErr> {
        let mut contact_id = None;
        let mut contact_created = false;

        if processed.doc_type == "GPAY" {
            let gpay: GPayExtraction = serde_json::from_value(processed.data.clone())
                .map_err(|e| DbErr::Custom(format!("Failed to parse GPay data: {}", e)))?;

            // 2.6 Auto-Contact Creation Logic
            if let Some(upi_id) = &gpay.counterparty_upi_id {
                // Check if identifier exists
                let identifier = entities::contact_identifiers::Entity::find()
                    .filter(entities::contact_identifiers::Column::Value.eq(upi_id))
                    .one(db)
                    .await?;

                if let Some(ident) = identifier {
                    contact_id = Some(ident.contact_id);
                } else {
                    // Create new contact
                    let new_contact = entities::contacts::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        name: Set(gpay.counterparty_name.clone()),
                        phone: Set(gpay.counterparty_phone.clone()),
                        is_pinned: Set(false),
                    };
                    let c_result = new_contact.insert(db).await?;
                    contact_id = Some(c_result.id.clone());
                    contact_created = true;

                    // Create identifier
                    let new_ident = entities::contact_identifiers::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        contact_id: Set(c_result.id.clone()),
                        r#type: Set("UPI".to_string()),
                        value: Set(upi_id.clone()),
                        linked_user_id: Set(None),
                    };
                    new_ident.insert(db).await?;

                    // Create link for user
                    let new_link = entities::contact_links::ActiveModel {
                        user_id: Set(user_id.to_string()),
                        contact_id: Set(c_result.id),
                    };
                    new_link.insert(db).await?;
                }
            }

            let direction = if gpay.direction == "IN" {
                TransactionDirection::In
            } else {
                TransactionDirection::Out
            };

            let date = if let Some(dt_str) = &gpay.datetime_str {
                match chrono::NaiveDateTime::parse_from_str(dt_str, "%d %b %Y, %I:%M %p") {
                    Ok(naive) => DateTime::<FixedOffset>::from_naive_utc_and_offset(
                        naive,
                        FixedOffset::east_opt(0).unwrap(),
                    ),
                    Err(_) => Utc::now().into(),
                }
            } else {
                Utc::now().into()
            };

            let txn = entities::transactions::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                user_id: Set(user_id.to_string()),
                amount: Set(gpay.amount),
                direction: Set(direction),
                date: Set(date),
                source: Set(TransactionSource::Ocr),
                status: Set(TransactionStatus::Completed),
                purpose_tag: Set(None),
                group_id: Set(None),
                source_wallet_id: Set(None),
                destination_wallet_id: Set(None),
                ledger_tab_id: Set(None),
                deleted_at: Set(None),
            };

            let result = txn.insert(db).await?;

            // 2.7 Store r2_file_url in transaction_sources
            let source = entities::transaction_sources::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                source_type: Set("GPAY_OCR".to_string()),
                r2_file_url: Set(processed.r2_key),
                raw_metadata: Set(Some(processed.data)),
            };
            source.insert(db).await?;

            // 2.6 Create txn_parties record
            if let Some(c_id) = contact_id {
                let party = entities::txn_parties::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    transaction_id: Set(result.id.clone()),
                    user_id: Set(None),
                    contact_id: Set(Some(c_id)),
                    role: Set("COUNTERPARTY".to_string()),
                };
                party.insert(db).await?;
            }

            let transfer = entities::p2p_transfers::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                direction: Set(gpay.direction),
                counterparty_name: Set(gpay.counterparty_name.clone()),
                counterparty_phone: Set(gpay.counterparty_phone),
                counterparty_upi_id: Set(gpay.counterparty_upi_id),
                is_merchant: Set(gpay.is_merchant),
                upi_transaction_id: Set(gpay.upi_transaction_id),
                google_transaction_id: Set(gpay.google_transaction_id),
                source_bank_account: Set(gpay.source_bank_account),
            };
            transfer.insert(db).await?;

            if gpay.is_merchant {
                let purchase = entities::purchases::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    transaction_id: Set(result.id.clone()),
                    vendor: Set(gpay.counterparty_name),
                    total: Set(gpay.amount),
                    order_id: Set(None),
                };
                purchase.insert(db).await?;
            }

            Ok(OcrTransactionResponse {
                transaction: result,
                contact_created,
            })
        } else {
            // Generic OCR path
            let generic: OcrResult = serde_json::from_value(processed.data.clone())
                .map_err(|e| DbErr::Custom(format!("Failed to parse Generic data: {}", e)))?;

            let amount = generic.amount.unwrap_or(Decimal::ZERO);

            let txn = entities::transactions::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                user_id: Set(user_id.to_string()),
                amount: Set(amount),
                direction: Set(TransactionDirection::Out),
                date: Set(generic.date.unwrap_or_else(|| Utc::now().into())),
                source: Set(TransactionSource::Ocr),
                status: Set(TransactionStatus::Completed),
                purpose_tag: Set(generic.vendor.clone()),
                group_id: Set(None),
                source_wallet_id: Set(None),
                destination_wallet_id: Set(None),
                ledger_tab_id: Set(None),
                deleted_at: Set(None),
            };

            let result = txn.insert(db).await?;

            let source = entities::transaction_sources::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                source_type: Set("GENERIC_OCR".to_string()),
                r2_file_url: Set(processed.r2_key),
                raw_metadata: Set(Some(processed.data)),
            };
            source.insert(db).await?;

            let purchase = entities::purchases::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                vendor: Set(generic.vendor.unwrap_or_else(|| "Unknown".to_string())),
                total: Set(amount),
                order_id: Set(None),
            };
            let p_result = purchase.insert(db).await?;

            for item in generic.items {
                let p_item = entities::purchase_items::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    purchase_id: Set(p_result.id.clone()),
                    name: Set(item.name),
                    quantity: Set(item.quantity),
                    price: Set(item.price),
                    sku: Set(None),
                };
                p_item.insert(db).await?;
            }

            Ok(OcrTransactionResponse {
                transaction: result,
                contact_created: false,
            })
        }
    }

    /// Creates a peer-to-peer (P2P) request for a given transaction.
    pub async fn create_p2p_request(
        db: &DatabaseConnection,
        sender_id: &str,
        receiver_email: &str,
        txn_id: &str,
    ) -> Result<entities::p2p_requests::Model, DbErr> {
        let txn = entities::transactions::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

        let txn_json = serde_json::to_value(&txn)
            .map_err(|e| DbErr::Custom(format!("Failed to serialize transaction: {}", e)))?;

        let request = entities::p2p_requests::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            sender_user_id: Set(sender_id.to_string()),
            receiver_email: Set(receiver_email.to_string()),
            transaction_data: Set(txn_json),
            status: Set(P2PRequestStatus::Pending.to_string()),
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
    ) -> Result<Vec<entities::p2p_requests::Model>, DbErr> {
        let txn = entities::transactions::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

        let mut results = Vec::new();
        for split in splits {
            let request = entities::p2p_requests::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                sender_user_id: Set(sender_id.to_string()),
                receiver_email: Set(split.receiver_email),
                transaction_data: Set(serde_json::json!({
                    "amount": split.amount,
                    "date": txn.date,
                    "purpose": format!("Split for {}", txn.purpose_tag.as_deref().unwrap_or("Expense"))
                })),
                status: Set(P2PRequestStatus::Pending.to_string()),
                linked_txn_id: Set(None),
            };
            let result = request.insert(db).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Accepts a P2P request, potentially creating a mirrored transaction for the receiver.
    pub async fn accept_p2p_request(
        db: &DatabaseConnection,
        receiver_id: &str,
        request_id: &str,
    ) -> Result<entities::p2p_requests::Model, DbErr> {
        let request = entities::p2p_requests::Entity::find_by_id(request_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Request not found".to_string()))?;

        if request.status != P2PRequestStatus::Pending.to_string()
            && request.status != P2PRequestStatus::GroupInvite.to_string()
        {
            return Err(DbErr::Custom("Request is not pending".to_string()));
        }

        if request.status == P2PRequestStatus::GroupInvite.to_string() {
            let metadata: serde_json::Value =
                serde_json::from_value(request.transaction_data.clone())
                    .map_err(|e| DbErr::Custom(format!("Failed to parse invite data: {}", e)))?;

            let group_id = metadata["group_id"]
                .as_str()
                .ok_or_else(|| DbErr::Custom("Missing group_id in invite".to_string()))?;

            let user_group = entities::user_groups::ActiveModel {
                user_id: Set(receiver_id.to_string()),
                group_id: Set(group_id.to_string()),
                role: Set(GroupRole::Member.to_string()),
            };
            user_group.insert(db).await?;

            let mut request: entities::p2p_requests::ActiveModel = request.into();
            request.status = Set(P2PRequestStatus::Approved.to_string());
            return request.update(db).await;
        }

        let original_txn: serde_json::Value =
            serde_json::from_value(request.transaction_data.clone())
                .map_err(|e| DbErr::Custom(format!("Failed to parse transaction data: {}", e)))?;

        let mirrored_txn = entities::transactions::ActiveModel {
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
            source_wallet_id: Set(None),
            destination_wallet_id: Set(None),
            ledger_tab_id: Set(None),
            deleted_at: Set(None),
        };

        let result_txn = mirrored_txn.insert(db).await?;

        let mut request: entities::p2p_requests::ActiveModel = request.into();
        request.status = Set(P2PRequestStatus::Mapped.to_string());
        request.linked_txn_id = Set(Some(result_txn.id));

        request.update(db).await
    }

    /// Detects recurring subscriptions from transaction history over the last 90 days.
    pub async fn detect_subscriptions(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::subscriptions::Model>, DbErr> {
        let ninety_days_ago = Utc::now() - Duration::days(90);
        let transactions = entities::transactions::Entity::find()
            .filter(entities::transactions::Column::UserId.eq(user_id))
            .filter(entities::transactions::Column::Date.gte(ninety_days_ago))
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

                    let sub = entities::subscriptions::Model {
                        id: uuid::Uuid::now_v7().to_string(),
                        user_id: user_id.to_string(),
                        name: name.clone(),
                        amount,
                        cycle: cycle.to_string(),
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
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<entities::transactions::Model>, DbErr> {
        let mut query = entities::transactions::Entity::find()
            .filter(entities::transactions::Column::UserId.eq(user_id))
            .filter(entities::transactions::Column::DeletedAt.is_null())
            .order_by_desc(entities::transactions::Column::Date);

        if let Some(l) = limit {
            query = query.limit(l);
        }
        if let Some(o) = offset {
            query = query.offset(o);
        }

        query.all(db).await
    }

    pub async fn list_contacts(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::contacts::Model>, DbErr> {
        entities::contacts::Entity::find()
            .join(
                JoinType::InnerJoin,
                entities::contacts::Relation::ContactLinks.def(),
            )
            .filter(entities::contact_links::Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    pub async fn create_contact(
        db: &DatabaseConnection,
        user_id: &str,
        name: String,
        phone: Option<String>,
    ) -> Result<entities::contacts::Model, DbErr> {
        let contact = entities::contacts::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            name: Set(name),
            phone: Set(phone),
            is_pinned: Set(false),
        };
        let result = contact.insert(db).await?;

        let link = entities::contact_links::ActiveModel {
            user_id: Set(user_id.to_string()),
            contact_id: Set(result.id.clone()),
        };
        link.insert(db).await?;

        Ok(result)
    }

    pub async fn update_contact(
        db: &DatabaseConnection,
        user_id: &str,
        contact_id: &str,
        name: Option<String>,
        phone: Option<String>,
        is_pinned: Option<bool>,
    ) -> Result<entities::contacts::Model, DbErr> {
        let _link = entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Contact link not found".to_string()))?;

        let mut contact: entities::contacts::ActiveModel = entities::contacts::Entity::find_by_id(contact_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Contact not found".to_string()))?
            .into();

        if let Some(n) = name {
            contact.name = Set(n);
        }
        if let Some(p) = phone {
            contact.phone = Set(Some(p));
        }
        if let Some(ip) = is_pinned {
            contact.is_pinned = Set(ip);
        }

        contact.update(db).await
    }

    pub async fn delete_contact(
        db: &DatabaseConnection,
        user_id: &str,
        contact_id: &str,
    ) -> Result<(), DbErr> {
        entities::contact_links::Entity::delete_by_id((user_id.to_string(), contact_id.to_string()))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn get_contact_detail(
        db: &DatabaseConnection,
        user_id: &str,
        contact_id: &str,
    ) -> Result<
        (
            entities::contacts::Model,
            Vec<entities::contact_identifiers::Model>,
            Vec<entities::transactions::Model>,
        ),
        DbErr,
    > {
        let _link = entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Contact link not found".to_string()))?;

        let contact = entities::contacts::Entity::find_by_id(contact_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Contact not found".to_string()))?;

        let identifiers = entities::contact_identifiers::Entity::find()
            .filter(entities::contact_identifiers::Column::ContactId.eq(contact_id))
            .all(db)
            .await?;

        let transactions = entities::transactions::Entity::find()
            .join(
                JoinType::InnerJoin,
                entities::transactions::Relation::TxnParties.def(),
            )
            .filter(entities::txn_parties::Column::ContactId.eq(contact_id))
            .order_by_desc(entities::transactions::Column::Date)
            .all(db)
            .await?;

        Ok((contact, identifiers, transactions))
    }

    pub async fn add_contact_identifier(
        db: &DatabaseConnection,
        user_id: &str,
        contact_id: &str,
        r#type: String,
        value: String,
    ) -> Result<entities::contact_identifiers::Model, DbErr> {
        let _link = entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Contact link not found".to_string()))?;

        let identifier = entities::contact_identifiers::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            contact_id: Set(contact_id.to_string()),
            r#type: Set(r#type),
            value: Set(value),
            linked_user_id: Set(None),
        };

        identifier.insert(db).await
    }

    pub async fn list_pending_p2p_requests(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Vec<P2PRequestWithSender>, DbErr> {
        let results = entities::p2p_requests::Entity::find()
            .filter(entities::p2p_requests::Column::ReceiverEmail.eq(email))
            .filter(entities::p2p_requests::Column::Status.is_in(["PENDING", "GROUP_INVITE"]))
            .find_also_related(entities::users::Entity)
            .all(db)
            .await?;

        Ok(results
            .into_iter()
            .map(|(request, user)| P2PRequestWithSender {
                request,
                sender_name: user.map(|u| u.name),
            })
            .collect())
    }

    pub async fn create_group(
        db: &DatabaseConnection,
        user_id: &str,
        name: &str,
        description: Option<String>,
    ) -> Result<entities::groups::Model, DbErr> {
        let group = entities::groups::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            name: Set(name.to_string()),
            description: Set(description),
            created_at: Set(Utc::now().into()),
        };
        let result = group.insert(db).await?;

        let user_group = entities::user_groups::ActiveModel {
            user_id: Set(user_id.to_string()),
            group_id: Set(result.id.clone()),
            role: Set(GroupRole::Admin.to_string()),
        };
        user_group.insert(db).await?;

        Ok(result)
    }

    pub async fn invite_to_group(
        db: &DatabaseConnection,
        sender_id: &str,
        receiver_email: &str,
        group_id: &str,
    ) -> Result<entities::p2p_requests::Model, DbErr> {
        let group = entities::groups::Entity::find_by_id(group_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Group not found".to_string()))?;

        let request = entities::p2p_requests::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            sender_user_id: Set(sender_id.to_string()),
            receiver_email: Set(receiver_email.to_string()),
            transaction_data: Set(serde_json::json!({
                "type": "GROUP_INVITE",
                "group_id": group.id,
                "group_name": group.name
            })),
            status: Set(P2PRequestStatus::GroupInvite.to_string()),
            linked_txn_id: Set(None),
        };

        request.insert(db).await
    }

    pub async fn list_groups(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::groups::Model>, DbErr> {
        entities::groups::Entity::find()
            .inner_join(entities::user_groups::Entity)
            .filter(entities::user_groups::Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    pub async fn list_group_transactions(
        db: &DatabaseConnection,
        group_id: &str,
    ) -> Result<Vec<entities::transactions::Model>, DbErr> {
        entities::transactions::Entity::find()
            .filter(entities::transactions::Column::GroupId.eq(group_id))
            .filter(entities::transactions::Column::DeletedAt.is_null())
            .order_by_desc(entities::transactions::Column::Date)
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
    ) -> Result<entities::transactions::Model, DbErr> {
        let txn = entities::transactions::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

        if txn.user_id != user_id {
            return Err(DbErr::Custom("Unauthorized".to_string()));
        }

        let old_amount = txn.amount;
        let mut txn: entities::transactions::ActiveModel = txn.into();

        if let Some(amt) = amount {
            if amt != old_amount {
                // Log the edit
                let edit = entities::transaction_edits::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    transaction_id: Set(txn.id.as_ref().to_string()),
                    old_amount: Set(old_amount),
                    new_amount: Set(amt),
                    edited_at: Set(Utc::now().into()),
                };
                edit.insert(db).await?;
            }
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
        let txn = entities::transactions::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

        if txn.user_id != user_id {
            return Err(DbErr::Custom("Unauthorized".to_string()));
        }

        let mut txn: entities::transactions::ActiveModel = txn.into();
        txn.deleted_at = Set(Some(Utc::now().into()));
        txn.update(db).await?;

        Ok(1)
    }

    pub async fn revert_transaction(
        db: &DatabaseConnection,
        user_id: &str,
        txn_id: &str,
    ) -> Result<entities::transactions::Model, DbErr> {
        let txn = entities::transactions::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

        if txn.user_id != user_id {
            return Err(DbErr::Custom("Unauthorized".to_string()));
        }

        let mut txn: entities::transactions::ActiveModel = txn.into();
        txn.deleted_at = Set(None);
        txn.update(db).await
    }

    pub async fn register_repayment(
        db: &DatabaseConnection,
        user_id: &str,
        tab_id: &str,
        amount: Decimal,
        source_wallet_id: Option<String>,
    ) -> Result<entities::transactions::Model, DbErr> {
        let tab = entities::ledger_tabs::Entity::find_by_id(tab_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| DbErr::Custom("Ledger tab not found".to_string()))?;

        let txn = entities::transactions::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            amount: Set(amount),
            direction: Set(TransactionDirection::In),
            date: Set(Utc::now().into()),
            source: Set(TransactionSource::Manual),
            status: Set(TransactionStatus::Completed),
            purpose_tag: Set(Some(format!("Repayment for: {}", tab.title))),
            group_id: Set(None),
            source_wallet_id: Set(source_wallet_id),
            destination_wallet_id: Set(None),
            ledger_tab_id: Set(Some(tab.id.clone())),
            deleted_at: Set(None),
        };

        let result = txn.insert(db).await?;

        let total_paid: Decimal = entities::transactions::Entity::find()
            .filter(entities::transactions::Column::LedgerTabId.eq(tab.id.clone()))
            .filter(entities::transactions::Column::DeletedAt.is_null())
            .all(db)
            .await?
            .iter()
            .map(|t| t.amount)
            .sum();

        if total_paid >= tab.target_amount {
            let mut tab: entities::ledger_tabs::ActiveModel = tab.into();
            tab.status = Set(LedgerTabStatus::Settled.to_string());
            tab.update(db).await?;
        } else if total_paid > Decimal::ZERO {
            let mut tab: entities::ledger_tabs::ActiveModel = tab.into();
            tab.status = Set(LedgerTabStatus::PartiallyPaid.to_string());
            tab.update(db).await?;
        }

        Ok(result)
    }

    pub async fn create_transaction(
        db: &DatabaseConnection,
        user_id: &str,
        amount: Decimal,
        direction: TransactionDirection,
        date: DateTime<FixedOffset>,
        source: TransactionSource,
        purpose_tag: Option<String>,
    ) -> Result<entities::transactions::Model, DbErr> {
        let txn = entities::transactions::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            amount: Set(amount),
            direction: Set(direction),
            date: Set(date),
            source: Set(source),
            status: Set(TransactionStatus::Completed),
            purpose_tag: Set(purpose_tag),
            group_id: Set(None),
            source_wallet_id: Set(None),
            destination_wallet_id: Set(None),
            ledger_tab_id: Set(None),
            deleted_at: Set(None),
        };

        txn.insert(db).await
    }

    pub async fn list_wallets(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::wallets::Model>, DbErr> {
        entities::wallets::Entity::find()
            .filter(entities::wallets::Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    pub async fn create_wallet(
        db: &DatabaseConnection,
        user_id: &str,
        name: &str,
        wallet_type: WalletType,
        initial_balance: Decimal,
    ) -> Result<entities::wallets::Model, DbErr> {
        let wallet = entities::wallets::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            name: Set(name.to_string()),
            r#type: Set(wallet_type.to_string()),
            balance: Set(initial_balance),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        wallet.insert(db).await
    }

    pub async fn update_wallet(
        db: &DatabaseConnection,
        user_id: &str,
        wallet_id: &str,
        name: Option<String>,
        balance: Option<Decimal>,
    ) -> Result<entities::wallets::Model, DbErr> {
        let mut wallet: entities::wallets::ActiveModel = entities::wallets::Entity::find()
            .filter(entities::wallets::Column::UserId.eq(user_id))
            .filter(entities::wallets::Column::Id.eq(wallet_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Wallet not found".to_string()))?
            .into();

        if let Some(n) = name {
            wallet.name = Set(n);
        }
        if let Some(b) = balance {
            wallet.balance = Set(b);
        }
        wallet.updated_at = Set(Utc::now().into());

        wallet.update(db).await
    }


    pub async fn create_ledger_tab(
        db: &DatabaseConnection,
        creator_id: &str,
        counterparty_id: Option<String>,
        tab_type: LedgerTabType,
        title: &str,
        target_amount: Decimal,
    ) -> Result<entities::ledger_tabs::Model, DbErr> {
        let tab = entities::ledger_tabs::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            creator_id: Set(creator_id.to_string()),
            counterparty_id: Set(counterparty_id),
            tab_type: Set(tab_type.to_string()),
            title: Set(title.to_string()),
            target_amount: Set(target_amount),
            status: Set(LedgerTabStatus::Open.to_string()),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };
        tab.insert(db).await
    }

    pub async fn update_profile(
        db: &DatabaseConnection,
        user_id: &str,
        name: Option<String>,
        username: Option<String>,
        image: Option<String>,
    ) -> Result<entities::users::Model, DbErr> {
        let mut user: entities::users::ActiveModel = entities::users::Entity::find_by_id(user_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("User not found".to_string()))?
            .into();

        if let Some(n) = name {
            user.name = Set(n);
        }
        if let Some(u) = username {
            user.username = Set(Some(u));
        }
        if let Some(i) = image {
            user.image = Set(Some(i));
        }
        user.updated_at = Set(Utc::now().into());

        user.update(db).await
    }

    pub async fn list_categories(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::categories::Model>, DbErr> {
        entities::categories::Entity::find()
            .filter(entities::categories::Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    pub async fn create_category(
        db: &DatabaseConnection,
        user_id: &str,
        name: String,
        icon: Option<String>,
        color: Option<String>,
    ) -> Result<entities::categories::Model, DbErr> {
        let category = entities::categories::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            name: Set(name),
            icon: Set(icon),
            color: Set(color),
        };
        category.insert(db).await
    }

    pub async fn delete_category(
        db: &DatabaseConnection,
        user_id: &str,
        category_id: &str,
    ) -> Result<(), DbErr> {
        entities::categories::Entity::delete_many()
            .filter(entities::categories::Column::Id.eq(category_id))
            .filter(entities::categories::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn list_user_upi_ids(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::user_upi_ids::Model>, DbErr> {
        entities::user_upi_ids::Entity::find()
            .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    pub async fn add_user_upi_id(
        db: &DatabaseConnection,
        user_id: &str,
        upi_id: String,
        label: Option<String>,
    ) -> Result<entities::user_upi_ids::Model, DbErr> {
        let upi = entities::user_upi_ids::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            upi_id: Set(upi_id),
            is_primary: Set(false),
            label: Set(label),
        };
        upi.insert(db).await
    }

    pub async fn make_primary_upi(
        db: &DatabaseConnection,
        user_id: &str,
        upi_id: &str,
    ) -> Result<(), DbErr> {
        // Unset current primary
        entities::user_upi_ids::Entity::update_many()
            .col_expr(entities::user_upi_ids::Column::IsPrimary, Expr::value(false))
            .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
            .exec(db)
            .await?;

        // Set new primary
        entities::user_upi_ids::Entity::update_many()
            .col_expr(entities::user_upi_ids::Column::IsPrimary, Expr::value(true))
            .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
            .filter(entities::user_upi_ids::Column::Id.eq(upi_id))
            .exec(db)
            .await?;

        Ok(())
    }

    pub async fn reject_p2p_request(
        db: &DatabaseConnection,
        _user_id: &str,
        request_id: &str,
    ) -> Result<(), DbErr> {
        let mut request: entities::p2p_requests::ActiveModel =
            entities::p2p_requests::Entity::find_by_id(request_id.to_string())
                .one(db)
                .await?
                .ok_or(DbErr::Custom("Request not found".to_string()))?
                .into();

        request.status = Set("REJECTED".to_string());
        request.update(db).await?;
        Ok(())
    }

    pub async fn list_confirmed_subscriptions(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::subscriptions::Model>, DbErr> {
        entities::subscriptions::Entity::find()
            .filter(entities::subscriptions::Column::UserId.eq(user_id))
            .all(db)
            .await
    }

    pub async fn confirm_subscription(
        db: &DatabaseConnection,
        user_id: &str,
        name: String,
        amount: Decimal,
        cycle: String,
        start_date: DateTime<FixedOffset>,
        next_charge_date: DateTime<FixedOffset>,
        keywords: Option<serde_json::Value>,
    ) -> Result<entities::subscriptions::Model, DbErr> {
        let sub = entities::subscriptions::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            name: Set(name),
            amount: Set(amount),
            cycle: Set(cycle),
            start_date: Set(start_date.into()),
            next_charge_date: Set(next_charge_date.into()),
            detection_keywords: Set(keywords),
        };
        sub.insert(db).await
    }

    pub async fn stop_tracking_subscription(
        db: &DatabaseConnection,
        user_id: &str,
        sub_id: &str,
    ) -> Result<(), DbErr> {
        entities::subscriptions::Entity::delete_many()
            .filter(entities::subscriptions::Column::Id.eq(sub_id))
            .filter(entities::subscriptions::Column::UserId.eq(user_id))
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn configure_subscription_alert(
        db: &DatabaseConnection,
        sub_id: &str,
        days_before: i32,
        channel: String,
    ) -> Result<entities::sub_alerts::Model, DbErr> {
        let alert = entities::sub_alerts::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            subscription_id: Set(sub_id.to_string()),
            days_before: Set(days_before),
            channel: Set(channel),
            sent_at: Set(None),
        };
        alert.insert(db).await
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

        db.execute(db_backend.build(&schema.create_table_from_entity(entities::users::Entity)))
            .await?;
        db.execute(
            db_backend.build(&schema.create_table_from_entity(entities::transactions::Entity)),
        )
        .await?;
        db.execute(
            db_backend.build(&schema.create_table_from_entity(entities::p2p_requests::Entity)),
        )
        .await?;

        // Create a user
        let user = entities::users::ActiveModel {
            id: Set("user_1".to_string()),
            email: Set("user_1@example.com".to_string()),
            name: Set("User One".to_string()),
            email_verified: Set(true),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        };
        user.insert(&db).await?;

        // Create a transaction
        let txn_id = uuid::Uuid::now_v7().to_string();
        let txn = entities::transactions::ActiveModel {
            id: Set(txn_id.clone()),
            user_id: Set("user_1".to_string()),
            amount: Set(Decimal::new(100, 0)),
            direction: Set(TransactionDirection::Out),
            date: Set(Utc::now().into()),
            source: Set(TransactionSource::Manual),
            status: Set(TransactionStatus::Completed),
            purpose_tag: Set(Some("Lunch".to_string())),
            group_id: Set(None),
            source_wallet_id: Set(None),
            destination_wallet_id: Set(None),
            ledger_tab_id: Set(None),
            deleted_at: Set(None),
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

        println!(
            "\nBENCHMARK_RESULT: {} splits took {:?}",
            num_splits, duration
        );

        Ok(())
    }
}
