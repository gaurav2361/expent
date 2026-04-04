use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use db::SmartMerge;
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub async fn list_pending_p2p_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::P2PRequestWithSender>>, ApiError> {
    let email = session
        .user
        .email
        .clone()
        .ok_or(ApiError::BadRequest("Email missing".to_string()))?;
    let result = SmartMerge::list_pending_p2p_requests(&state.db, &email).await?;

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct CreateP2PRequest {
    pub receiver_email: String,
    pub transaction_id: String,
}

pub async fn create_p2p_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<CreateP2PRequest>,
) -> Result<Json<db::entities::p2p_requests::Model>, ApiError> {
    let result = SmartMerge::create_p2p_request(
        &state.db,
        &session.user.id,
        &payload.receiver_email,
        &payload.transaction_id,
    )
    .await?;

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct AcceptP2PRequest {
    pub request_id: String,
}

pub async fn accept_p2p_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<AcceptP2PRequest>,
) -> Result<Json<db::entities::p2p_requests::Model>, ApiError> {
    let result = SmartMerge::accept_p2p_request(&state.db, &session.user.id, &payload.request_id).await?;

    Ok(Json(result))
}

pub async fn reject_p2p_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    SmartMerge::reject_p2p_request(&state.db, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
