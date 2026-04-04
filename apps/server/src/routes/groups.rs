use axum::extract::{Json, Path, State};
use db::SmartMerge;
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub async fn list_groups_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::groups::Model>>, ApiError> {
    let result = SmartMerge::list_groups(&state.db, &session.user.id).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct CreateGroupRequest {
    pub name: String,
    pub description: Option<String>,
}

pub async fn create_group_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<Json<db::entities::groups::Model>, ApiError> {
    let result =
        SmartMerge::create_group(&state.db, &session.user.id, &payload.name, payload.description)
            .await?;

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct InviteToGroupRequest {
    pub receiver_email: String,
    pub group_id: String,
}

pub async fn invite_to_group_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<InviteToGroupRequest>,
) -> Result<Json<db::entities::p2p_requests::Model>, ApiError> {
    let result = SmartMerge::invite_to_group(
        &state.db,
        &session.user.id,
        &payload.receiver_email,
        &payload.group_id,
    )
    .await?;

    Ok(Json(result))
}

pub async fn list_group_transactions_handler(
    State(state): State<AppState>,
    _session: AuthSession,
    Path(id): Path<String>,
) -> Result<Json<Vec<db::entities::transactions::Model>>, ApiError> {
    let result = SmartMerge::list_group_transactions(&state.db, &id).await?;

    Ok(Json(result))
}
