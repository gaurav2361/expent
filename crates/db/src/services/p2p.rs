use crate::entities::enums::{
    LedgerTabStatus, LedgerTabType, P2PRequestStatus, TransactionDirection, TransactionSource,
    TransactionStatus,
};
use crate::{P2PRequestWithSender, entities};
use chrono::Utc;
use rust_decimal::Decimal;
use sea_orm::*;
use std::str::FromStr;

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
        let metadata: serde_json::Value = serde_json::from_value(request.transaction_data.clone())
            .map_err(|e| DbErr::Custom(format!("Failed to parse invite data: {}", e)))?;

        let group_id = metadata["group_id"]
            .as_str()
            .ok_or_else(|| DbErr::Custom("Missing group_id in invite".to_string()))?;

        let user_group = entities::user_groups::ActiveModel {
            user_id: Set(receiver_id.to_string()),
            group_id: Set(group_id.to_string()),
            role: Set(crate::entities::enums::GroupRole::Member.to_string()),
        };
        user_group.insert(db).await?;

        let mut request: entities::p2p_requests::ActiveModel = request.into();
        request.status = Set(P2PRequestStatus::Approved.to_string());
        return request.update(db).await;
    }

    let original_txn: serde_json::Value = serde_json::from_value(request.transaction_data.clone())
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
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| {
                d.with_timezone(&chrono::FixedOffset::east_opt(0).unwrap())
                    .into()
            })
            .unwrap_or_else(|| Utc::now().into())),
        source: Set(TransactionSource::P2p),
        status: Set(TransactionStatus::Completed),
        category_id: Set(None),
        purpose_tag: Set(original_txn["purpose"].as_str().map(|s| s.to_string())),
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

    request.update(db).await
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

pub async fn create_ledger_tab(
    db: &DatabaseConnection,
    creator_id: &str,
    counterparty_id: Option<String>,
    tab_type: LedgerTabType,
    title: &str,
    description: Option<String>,
    target_amount: Decimal,
) -> Result<entities::ledger_tabs::Model, DbErr> {
    let tab = entities::ledger_tabs::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        creator_id: Set(creator_id.to_string()),
        counterparty_id: Set(counterparty_id),
        tab_type: Set(tab_type.to_string()),
        title: Set(title.to_string()),
        description: Set(description),
        target_amount: Set(target_amount),
        status: Set(LedgerTabStatus::Open.to_string()),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    tab.insert(db).await
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
        category_id: Set(None),
        purpose_tag: Set(Some(format!("Repayment for: {}", tab.title))),
        group_id: Set(None),
        source_wallet_id: Set(source_wallet_id),
        destination_wallet_id: Set(None),
        ledger_tab_id: Set(Some(tab.id.clone())),
        deleted_at: Set(None),
        notes: Set(None),
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
