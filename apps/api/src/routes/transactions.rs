use axum::Router;
use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, patch, post};
use expent_core::{SplitDetail, ocr, transactions};
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

use crate::extractors::ValidatedJson;
use validator::Validate;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_transactions_handler))
        .route("/manual", post(create_manual_transaction_handler))
        .route("/from-ocr", post(create_from_ocr_handler))
        .route("/{id}", patch(update_transaction_handler))
        .route("/{id}", delete(delete_transaction_handler))
        .route("/split", post(split_transaction_handler))
}

#[derive(Deserialize, Validate)]
pub struct CreateManualTransactionRequest {
    #[validate(custom(function = "validate_amount"))]
    pub amount: rust_decimal::Decimal,
    pub date: chrono::DateTime<chrono::FixedOffset>,
    #[validate(length(min = 1, max = 255))]
    pub purpose_tag: String,
    pub category_id: Option<String>,
    pub direction: db::entities::enums::TransactionDirection,
    pub source_wallet_id: Option<String>,
    pub destination_wallet_id: Option<String>,
    pub contact_id: Option<String>,
    pub notes: Option<String>,
}

fn validate_amount(amount: &rust_decimal::Decimal) -> Result<(), validator::ValidationError> {
    if amount <= &rust_decimal::Decimal::ZERO {
        return Err(validator::ValidationError::new("amount_must_be_positive"));
    }
    Ok(())
}

pub async fn create_manual_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    ValidatedJson(payload): ValidatedJson<CreateManualTransactionRequest>,
) -> Result<Json<db::entities::transactions::Model>, ApiError> {
    let result = transactions::create_transaction(
        &state.core.db,
        &session.user.id,
        payload.amount,
        payload.direction,
        payload.date,
        db::entities::enums::TransactionSource::Manual,
        Some(payload.purpose_tag),
        payload.category_id,
        payload.source_wallet_id,
        payload.destination_wallet_id,
        payload.contact_id,
        payload.notes,
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
) -> Result<Json<Vec<db::TransactionWithDetail>>, ApiError> {
    let result = transactions::list_transactions(
        &state.core.db,
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
    Json(payload): Json<expent_core::ProcessedOcr>,
) -> Result<Json<expent_core::OcrTransactionResponse>, ApiError> {
    let result = ocr::process_ocr(&state.core.db, &session.user.id, payload).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct UpdateTransactionRequest {
    pub amount: Option<rust_decimal::Decimal>,
    pub date: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub purpose_tag: Option<String>,
    pub category_id: Option<String>,
    pub status: Option<db::entities::enums::TransactionStatus>,
    pub notes: Option<String>,
    pub source_wallet_id: Option<String>,
    pub destination_wallet_id: Option<String>,
    pub contact_id: Option<String>,
}

pub async fn update_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTransactionRequest>,
) -> Result<Json<db::entities::transactions::Model>, ApiError> {
    let result = transactions::update_transaction(
        &state.core.db,
        &session.user.id,
        &id,
        payload.amount,
        payload.date,
        payload.purpose_tag,
        payload.category_id,
        payload.status,
        payload.notes,
        payload.source_wallet_id,
        payload.destination_wallet_id,
        payload.contact_id,
    )
    .await?;

    Ok(Json(result))
}

pub async fn delete_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    transactions::delete_transaction(&state.core.db, &session.user.id, &id).await?;
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
    let result = transactions::split_transaction(
        &state.core.db,
        &session.user.id,
        &payload.transaction_id,
        payload.splits,
    )
    .await?;

    Ok(Json(result))
}
