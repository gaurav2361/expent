use chrono::Utc;
use db::AppError;
use db::entities;
use db::entities::enums::TransactionStatus;
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::*;

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
) -> Result<entities::transactions::Model, AppError> {
    let user_id = user_id.to_string();
    let txn_id = txn_id.to_string();
    db.transaction::<_, entities::transactions::Model, AppError>(|txn_db| {
        Box::pin(async move {
            let txn_model = entities::transactions::Entity::find_by_id(txn_id)
                .one(txn_db)
                .await?
                .ok_or_else(|| AppError::not_found("Transaction not found"))?;

            if txn_model.user_id != user_id {
                return Err(AppError::unauthorized("Unauthorized"));
            }

            let old_amount = txn_model.amount;
            let mut txn: entities::transactions::ActiveModel = txn_model.clone().into();

            if let Some(amt) = amount {
                if amt != old_amount {
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

            super::utils::adjust_transaction_wallets(txn_db, Some(&txn_model), Some(&result))
                .await?;

            Ok(result)
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => AppError::Db(ce),
        TransactionError::Transaction(te) => te,
    })
}
