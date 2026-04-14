use chrono::Utc;
use db::AppError;
use db::entities;
use sea_orm::{DatabaseConnection, Iden, TransactionTrait, EntityTrait, Set, ActiveModelTrait, TransactionError};

pub async fn delete_transaction(
    db: &DatabaseConnection,
    user_id: &str,
    txn_id: &str,
) -> Result<u64, AppError> {
    let user_id = user_id.to_string();
    let txn_id = txn_id.to_string();
    db.transaction::<_, u64, AppError>(|txn_db| {
        Box::pin(async move {
            let txn_model = entities::transactions::Entity::find_by_id(txn_id)
                .one(txn_db)
                .await?
                .ok_or_else(|| AppError::not_found("Transaction not found"))?;

            if txn_model.user_id != user_id {
                return Err(AppError::unauthorized("Unauthorized"));
            }

            let mut txn: entities::transactions::ActiveModel = txn_model.clone().into();
            txn.deleted_at = Set(Some(Utc::now().into()));
            let result_model = txn.update(txn_db).await?;

            // Adjust wallet balances
            super::utils::adjust_transaction_wallets(txn_db, Some(&txn_model), Some(&result_model))
                .await?;

            Ok(1)
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => AppError::Db(ce),
        TransactionError::Transaction(te) => te,
    })
}
