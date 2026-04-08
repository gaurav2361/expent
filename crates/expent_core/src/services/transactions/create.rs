use chrono::{DateTime, FixedOffset};
use db::AppError;
use db::entities;
use db::entities::enums::{TransactionDirection, TransactionSource, TransactionStatus};
use rust_decimal::Decimal;
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
) -> Result<entities::transactions::Model, AppError> {
    let user_id = user_id.to_string();
    db.transaction::<_, entities::transactions::Model, AppError>(|txn_db| {
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
            super::utils::adjust_transaction_wallets(txn_db, None, Some(&result)).await?;

            Ok(result)
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => AppError::Db(ce),
        TransactionError::Transaction(te) => te,
    })
}
