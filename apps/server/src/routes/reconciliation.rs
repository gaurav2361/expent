use axum::Router;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use db::SmartMerge;
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/rows", get(list_unmatched_rows_handler))
        .route("/rows/{id}/matches", get(get_row_matches_handler))
        .route("/rows/{id}/confirm", post(confirm_match_handler))
        .route("/upload", post(upload_statement_handler))
}

pub async fn list_unmatched_rows_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::bank_statement_rows::Model>>, ApiError> {
    let result = SmartMerge::list_unmatched_rows(&state.db, &session.user.id).await?;
    Ok(Json(result))
}

#[derive(Serialize)]
pub struct RowMatchesResponse {
    pub row: db::entities::bank_statement_rows::Model,
    pub matches: Vec<(db::entities::transactions::Model, i32)>,
}

pub async fn get_row_matches_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<Json<RowMatchesResponse>, ApiError> {
    let row = db::entities::bank_statement_rows::Entity::find_by_id(id.clone())
        .one(&state.db)
        .await?
        .ok_or(ApiError::NotFound("Row not found".to_string()))?;

    let matches = SmartMerge::get_row_matches(&state.db, &session.user.id, &id).await?;

    Ok(Json(RowMatchesResponse { row, matches }))
}

#[derive(Deserialize)]
pub struct ConfirmMatchRequest {
    pub transaction_id: String,
    pub confidence: i32,
}

pub async fn confirm_match_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    Json(payload): Json<ConfirmMatchRequest>,
) -> Result<StatusCode, ApiError> {
    SmartMerge::confirm_match(
        &state.db,
        &session.user.id,
        &id,
        &payload.transaction_id,
        payload.confidence,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct StatementUploadRequest {
    pub rows: Vec<StatementRow>,
}

#[derive(Deserialize)]
pub struct StatementRow {
    pub date: chrono::DateTime<chrono::FixedOffset>,
    pub description: String,
    pub amount: rust_decimal::Decimal,
}

pub async fn upload_statement_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<StatementUploadRequest>,
) -> Result<StatusCode, ApiError> {
    for row in payload.rows {
        SmartMerge::upload_statement(
            &state.db,
            &session.user.id,
            row.date,
            row.description,
            row.amount,
            None,
        )
        .await?;
    }
    Ok(StatusCode::CREATED)
}
