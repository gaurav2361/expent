use db::entities;
use db::entities::enums::P2PRequestStatus;
use db::{AppError, SplitDetail};
use sea_orm::*;

/// Splits a transaction among multiple receivers.
pub async fn split_transaction(
    db: &DatabaseConnection,
    sender_id: &str,
    txn_id: &str,
    splits: Vec<SplitDetail>,
) -> Result<Vec<entities::p2p_requests::Model>, AppError> {
    let txn = entities::transactions::Entity::find_by_id(txn_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Transaction not found"))?;

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
