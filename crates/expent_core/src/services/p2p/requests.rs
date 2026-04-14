use chrono::Utc;
use db::entities;
use db::entities::enums::{
    P2PRequestStatus, TransactionDirection, TransactionSource, TransactionStatus,
};
use db::{AppError, P2PRequestWithSender};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveEnum, ActiveModelTrait, ColIdx, ColumnTrait, DatabaseConnection, EntityTrait, Iden,
    IdenStatic, QueryFilter, Set,
};
use std::str::FromStr;

pub async fn list_pending_p2p_requests(
    db: &DatabaseConnection,
    email: &str,
) -> Result<Vec<P2PRequestWithSender>, AppError> {
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

pub async fn create_p2p_request(
    db: &DatabaseConnection,
    sender_id: &str,
    receiver_email: &str,
    txn_id: &str,
) -> Result<entities::p2p_requests::Model, AppError> {
    let txn = entities::transactions::Entity::find_by_id(txn_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Transaction not found"))?;

    let txn_json = serde_json::to_value(&txn)
        .map_err(|e| AppError::Generic(format!("Failed to serialize transaction: {e}")))?;

    let request = entities::p2p_requests::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        sender_user_id: Set(sender_id.to_string()),
        receiver_email: Set(receiver_email.to_string()),
        transaction_data: Set(txn_json),
        status: Set(P2PRequestStatus::Pending.to_string()),
        linked_txn_id: Set(None),
    };

    request.insert(db).await.map_err(AppError::from)
}

pub async fn accept_p2p_request(
    db: &DatabaseConnection,
    receiver_id: &str,
    request_id: &str,
) -> Result<entities::p2p_requests::Model, AppError> {
    let request = entities::p2p_requests::Entity::find_by_id(request_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Request not found"))?;

    if request.status != P2PRequestStatus::Pending.to_string()
        && request.status != P2PRequestStatus::GroupInvite.to_string()
    {
        return Err(AppError::unauthorized("Request is not pending"));
    }

    if request.status == P2PRequestStatus::GroupInvite.to_string() {
        let metadata: serde_json::Value = serde_json::from_value(request.transaction_data.clone())
            .map_err(|e| AppError::Generic(format!("Failed to parse invite data: {e}")))?;

        let group_id = metadata["group_id"]
            .as_str()
            .ok_or_else(|| AppError::Generic("Missing group_id in invite".to_string()))?;

        let user_group = entities::user_groups::ActiveModel {
            user_id: Set(receiver_id.to_string()),
            group_id: Set(group_id.to_string()),
            role: Set(db::entities::enums::GroupRole::Member.to_string()),
        };
        user_group.insert(db).await?;

        let mut request: entities::p2p_requests::ActiveModel = request.into();
        request.status = Set(P2PRequestStatus::Approved.to_string());
        return request.update(db).await.map_err(AppError::from);
    }

    let original_txn: serde_json::Value = serde_json::from_value(request.transaction_data.clone())
        .map_err(|e| AppError::Generic(format!("Failed to parse transaction data: {e}")))?;

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
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map_or_else(
                || Utc::now().into(),
                |d| d.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
            )),
        source: Set(TransactionSource::P2p),
        status: Set(TransactionStatus::Completed),
        category_id: Set(None),
        purpose_tag: Set(original_txn["purpose"]
            .as_str()
            .map(std::string::ToString::to_string)),
        group_id: Set(None),
        source_wallet_id: Set(None),
        destination_wallet_id: Set(None),
        ledger_tab_id: Set(None),
        deleted_at: Set(None),
        notes: Set(None),
    };

    let result_txn = mirrored_txn.insert(db).await?;

    let mut request: entities::p2p_requests::ActiveModel = request.into();
    request.status = Set(P2PRequestStatus::Mapped.to_string());
    request.linked_txn_id = Set(Some(result_txn.id));

    request.update(db).await.map_err(AppError::from)
}

pub async fn reject_p2p_request(
    db: &DatabaseConnection,
    _user_id: &str,
    request_id: &str,
) -> Result<(), AppError> {
    let mut request: entities::p2p_requests::ActiveModel =
        entities::p2p_requests::Entity::find_by_id(request_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Request not found"))?
            .into();

    request.status = Set("REJECTED".to_string());
    request.update(db).await?;
    Ok(())
}
