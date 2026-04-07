use crate::entities;
use crate::entities::enums::{
    P2PRequestStatus, TransactionDirection, TransactionSource, TransactionStatus,
};
use crate::{SplitDetail, TransactionWithDetail};
use chrono::{DateTime, FixedOffset, Utc};
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::*;

pub async fn create_transaction(
    db: &DatabaseConnection,
    user_id: &str,
    amount: Decimal,
    direction: TransactionDirection,
    date: DateTime<FixedOffset>,
    source: TransactionSource,
    purpose_tag: Option<String>,
    category_id: Option<String>,
    source_wallet_id: Option<String>,
    destination_wallet_id: Option<String>,
    contact_id: Option<String>,
    notes: Option<String>,
) -> Result<entities::transactions::Model, DbErr> {
    let user_id = user_id.to_string();
    db.transaction::<_, entities::transactions::Model, DbErr>(|txn_db| {
        Box::pin(async move {
            let txn = entities::transactions::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                user_id: Set(user_id),
                amount: Set(amount),
                direction: Set(direction),
                date: Set(date),
                source: Set(source),
                status: Set(TransactionStatus::Completed),
                purpose_tag: Set(purpose_tag),
                category_id: Set(category_id),
                group_id: Set(None),
                source_wallet_id: Set(source_wallet_id.clone()),
                destination_wallet_id: Set(destination_wallet_id.clone()),
                ledger_tab_id: Set(None),
                deleted_at: Set(None),
                notes: Set(notes),
            };

            let result = txn.insert(txn_db).await?;

            if let Some(c_id) = contact_id {
                let party = entities::txn_parties::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    transaction_id: Set(result.id.clone()),
                    user_id: Set(None),
                    contact_id: Set(Some(c_id)),
                    role: Set("COUNTERPARTY".to_string()),
                };
                party.insert(txn_db).await?;
            }

            // Adjust wallet balances
            if let Some(sw_id) = source_wallet_id {
                super::wallets::adjust_balance(txn_db, &sw_id, -amount).await?;
            }
            if let Some(dw_id) = destination_wallet_id {
                super::wallets::adjust_balance(txn_db, &dw_id, amount).await?;
            }

            Ok(result)
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => ce,
        TransactionError::Transaction(te) => te,
    })
}

pub async fn list_transactions(
    db: &DatabaseConnection,
    user_id: &str,
    limit: Option<u64>,
    offset: Option<u64>,
) -> Result<Vec<TransactionWithDetail>, DbErr> {
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

    let results = query.all(db).await?;

    let mut final_results = Vec::new();
    for txn in results {
        let mut source_name = None;
        let mut dest_name = None;

        if let Some(sw_id) = &txn.source_wallet_id {
            if let Some(w) = entities::wallets::Entity::find_by_id(sw_id.clone())
                .one(db)
                .await?
            {
                source_name = Some(w.name);
            }
        }

        if let Some(dw_id) = &txn.destination_wallet_id {
            if let Some(w) = entities::wallets::Entity::find_by_id(dw_id.clone())
                .one(db)
                .await?
            {
                dest_name = Some(w.name);
            }
        }

        let mut contact_name = None;
        let mut contact_id = None;

        if let Some(party) = entities::txn_parties::Entity::find()
            .filter(entities::txn_parties::Column::TransactionId.eq(txn.id.clone()))
            .filter(entities::txn_parties::Column::Role.eq("COUNTERPARTY"))
            .one(db)
            .await?
        {
            if let Some(c_id) = party.contact_id {
                if let Some(c) = entities::contacts::Entity::find_by_id(c_id.clone())
                    .one(db)
                    .await?
                {
                    contact_name = Some(c.name);
                    contact_id = Some(c_id);
                }
            }
        }

        final_results.push(TransactionWithDetail {
            transaction: txn,
            source_wallet_name: source_name,
            destination_wallet_name: dest_name,
            contact_name,
            contact_id,
        });
    }

    Ok(final_results)
}

pub async fn update_transaction(
    db: &DatabaseConnection,
    user_id: &str,
    txn_id: &str,
    amount: Option<Decimal>,
    date: Option<DateTimeWithTimeZone>,
    purpose_tag: Option<String>,
    category_id: Option<String>,
    status: Option<TransactionStatus>,
    notes: Option<String>,
    source_wallet_id: Option<String>,
    destination_wallet_id: Option<String>,
    contact_id: Option<String>,
) -> Result<entities::transactions::Model, DbErr> {
    let user_id = user_id.to_string();
    let txn_id = txn_id.to_string();
    db.transaction::<_, entities::transactions::Model, DbErr>(|txn_db| {
        Box::pin(async move {
            let txn_model = entities::transactions::Entity::find_by_id(txn_id)
                .one(txn_db)
                .await?
                .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

            if txn_model.user_id != user_id {
                return Err(DbErr::Custom("Unauthorized".to_string()));
            }

            let old_amount = txn_model.amount;
            let old_source_wallet = txn_model.source_wallet_id.clone();
            let old_dest_wallet = txn_model.destination_wallet_id.clone();

            let mut txn: entities::transactions::ActiveModel = txn_model.clone().into();

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
                    edit.insert(txn_db).await?;
                }
                txn.amount = Set(amt);
            }
            if let Some(dt) = date {
                txn.date = Set(dt);
            }
            if let Some(tag) = purpose_tag {
                txn.purpose_tag = Set(Some(tag));
            }
            if let Some(c_id) = category_id {
                txn.category_id = Set(Some(c_id));
            }
            if let Some(s) = status {
                txn.status = Set(s);
            }
            if let Some(n) = notes {
                txn.notes = Set(Some(n));
            }
            if let Some(sw_id) = source_wallet_id.clone() {
                txn.source_wallet_id = Set(if sw_id.is_empty() { None } else { Some(sw_id) });
            }
            if let Some(dw_id) = destination_wallet_id.clone() {
                txn.destination_wallet_id = Set(if dw_id.is_empty() { None } else { Some(dw_id) });
            }

            let result = txn.update(txn_db).await?;

            if let Some(c_id) = contact_id {
                // Delete existing counterparty records for this transaction
                entities::txn_parties::Entity::delete_many()
                    .filter(entities::txn_parties::Column::TransactionId.eq(result.id.clone()))
                    .filter(entities::txn_parties::Column::Role.eq("COUNTERPARTY"))
                    .exec(txn_db)
                    .await?;

                if !c_id.is_empty() {
                    let party = entities::txn_parties::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        transaction_id: Set(result.id.clone()),
                        user_id: Set(None),
                        contact_id: Set(Some(c_id)),
                        role: Set("COUNTERPARTY".to_string()),
                    };
                    party.insert(txn_db).await?;
                }
            }

            // Adjust wallet balances
            // 1. Reverse old effect IF it was not cancelled
            let old_is_active = txn_model.status != TransactionStatus::Cancelled;
            if old_is_active {
                if let Some(sw_id) = old_source_wallet {
                    super::wallets::adjust_balance(txn_db, &sw_id, old_amount).await?;
                }
                if let Some(dw_id) = old_dest_wallet {
                    super::wallets::adjust_balance(txn_db, &dw_id, -old_amount).await?;
                }
            }

            // 2. Apply new effect IF it is not cancelled
            let new_is_active = result.status != TransactionStatus::Cancelled;
            if new_is_active {
                let new_amount = result.amount;
                if let Some(sw_id) = &result.source_wallet_id {
                    super::wallets::adjust_balance(txn_db, sw_id, -new_amount).await?;
                }
                if let Some(dw_id) = &result.destination_wallet_id {
                    super::wallets::adjust_balance(txn_db, dw_id, new_amount).await?;
                }
            }

            Ok(result)
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => ce,
        TransactionError::Transaction(te) => te,
    })
}

pub async fn delete_transaction(
    db: &DatabaseConnection,
    user_id: &str,
    txn_id: &str,
) -> Result<u64, DbErr> {
    let user_id = user_id.to_string();
    let txn_id = txn_id.to_string();
    db.transaction::<_, u64, DbErr>(|txn_db| {
        Box::pin(async move {
            let txn_model = entities::transactions::Entity::find_by_id(txn_id)
                .one(txn_db)
                .await?
                .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

            if txn_model.user_id != user_id {
                return Err(DbErr::Custom("Unauthorized".to_string()));
            }

            let old_amount = txn_model.amount;
            let old_source_wallet = txn_model.source_wallet_id.clone();
            let old_dest_wallet = txn_model.destination_wallet_id.clone();

            let mut txn: entities::transactions::ActiveModel = txn_model.into();
            txn.deleted_at = Set(Some(Utc::now().into()));
            let result_model = txn.update(txn_db).await?;

            // Reverse wallet effects IF it was not cancelled
            if result_model.status != TransactionStatus::Cancelled {
                if let Some(sw_id) = old_source_wallet {
                    super::wallets::adjust_balance(txn_db, &sw_id, old_amount).await?;
                }
                if let Some(dw_id) = old_dest_wallet {
                    super::wallets::adjust_balance(txn_db, &dw_id, -old_amount).await?;
                }
            }

            Ok(1)
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => ce,
        TransactionError::Transaction(te) => te,
    })
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

pub async fn revert_transaction(
    db: &DatabaseConnection,
    user_id: &str,
    txn_id: &str,
) -> Result<entities::transactions::Model, DbErr> {
    let user_id = user_id.to_string();
    let txn_id = txn_id.to_string();
    db.transaction::<_, entities::transactions::Model, DbErr>(|txn_db| {
        Box::pin(async move {
            let txn_model = entities::transactions::Entity::find_by_id(txn_id)
                .one(txn_db)
                .await?
                .ok_or_else(|| DbErr::Custom("Transaction not found".to_string()))?;

            if txn_model.user_id != user_id {
                return Err(DbErr::Custom("Unauthorized".to_string()));
            }

            let mut txn: entities::transactions::ActiveModel = txn_model.clone().into();
            txn.deleted_at = Set(None);
            let result = txn.update(txn_db).await?;

            // Re-apply wallet effects IF it was not cancelled
            if result.status != TransactionStatus::Cancelled {
                let amount = result.amount;
                if let Some(sw_id) = &result.source_wallet_id {
                    super::wallets::adjust_balance(txn_db, sw_id, -amount).await?;
                }
                if let Some(dw_id) = &result.destination_wallet_id {
                    super::wallets::adjust_balance(txn_db, dw_id, amount).await?;
                }
            }

            Ok(result)
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => ce,
        TransactionError::Transaction(te) => te,
    })
}
