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
    let txn = entities::transactions::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        amount: Set(amount),
        direction: Set(direction),
        date: Set(date),
        source: Set(source),
        status: Set(TransactionStatus::Completed),
        purpose_tag: Set(purpose_tag),
        category_id: Set(category_id),
        group_id: Set(None),
        source_wallet_id: Set(source_wallet_id),
        destination_wallet_id: Set(destination_wallet_id),
        ledger_tab_id: Set(None),
        deleted_at: Set(None),
        notes: Set(notes),
    };

    let result = txn.insert(db).await?;

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

    Ok(result)
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

        final_results.push(TransactionWithDetail {
            transaction: txn,
            source_wallet_name: source_name,
            destination_wallet_name: dest_name,
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
    if let Some(c_id) = category_id {
        txn.category_id = Set(Some(c_id));
    }
    if let Some(s) = status {
        txn.status = Set(s);
    }
    if let Some(n) = notes {
        txn.notes = Set(Some(n));
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
