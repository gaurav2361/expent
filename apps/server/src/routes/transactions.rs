use axum::extract::{Json, Path, State, Query};
use axum::http::StatusCode;
use db::{SmartMerge, SplitDetail};
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

#[derive(Deserialize)]
pub struct CreateManualTransactionRequest {
    pub amount: rust_decimal::Decimal,
    pub date: chrono::DateTime<chrono::FixedOffset>,
    pub purpose_tag: String,
    pub direction: db::entities::enums::TransactionDirection,
    pub source_wallet_id: Option<String>,
    pub destination_wallet_id: Option<String>,
    pub contact_id: Option<String>,
}

pub async fn create_manual_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<CreateManualTransactionRequest>,
) -> Result<Json<db::entities::transactions::Model>, ApiError> {
    let result = SmartMerge::create_transaction(
        &state.db,
        &session.user.id,
        payload.amount,
        payload.direction,
        payload.date,
        db::entities::enums::TransactionSource::Manual,
        Some(payload.purpose_tag),
        payload.source_wallet_id,
        payload.destination_wallet_id,
        payload.contact_id,
    )
    .await?;

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct PaginationParams {
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

pub async fn list_transactions_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<db::entities::transactions::Model>>, ApiError> {
    let result = SmartMerge::list_transactions(
        &state.db,
        &session.user.id,
        params.limit,
        params.offset,
    )
    .await?;
    Ok(Json(result))
}

pub async fn create_from_ocr_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<db::ProcessedOcr>,
) -> Result<Json<db::OcrTransactionResponse>, ApiError> {
    let result = db::SmartMerge::process_ocr(&state.db, &session.user.id, payload).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct UpdateTransactionRequest {
    pub amount: Option<rust_decimal::Decimal>,
    pub date: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub purpose_tag: Option<String>,
    pub status: Option<db::entities::enums::TransactionStatus>,
}

pub async fn update_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTransactionRequest>,
) -> Result<Json<db::entities::transactions::Model>, ApiError> {
    let result = SmartMerge::update_transaction(
        &state.db,
        &session.user.id,
        &id,
        payload.amount,
        payload.date,
        payload.purpose_tag,
        payload.status,
    )
    .await?;

    Ok(Json(result))
}

pub async fn delete_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    SmartMerge::delete_transaction(&state.db, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct SplitTransactionRequest {
    pub transaction_id: String,
    pub splits: Vec<SplitDetail>,
}

pub async fn split_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<SplitTransactionRequest>,
) -> Result<Json<Vec<db::entities::p2p_requests::Model>>, ApiError> {
    let result = SmartMerge::split_transaction(
        &state.db,
        &session.user.id,
        &payload.transaction_id,
        payload.splits,
    )
    .await?;

    Ok(Json(result))
}
