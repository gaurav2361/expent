use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use db::SmartMerge;
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

#[derive(Deserialize)]
pub struct UpdateProfileRequest {
    pub name: Option<String>,
    pub username: Option<String>,
    pub image: Option<String>,
}

pub async fn update_profile_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<UpdateProfileRequest>,
) -> Result<Json<db::entities::users::Model>, ApiError> {
    let result = SmartMerge::update_profile(
        &state.db,
        &session.user.id,
        payload.name,
        payload.username,
        payload.image,
    )
    .await?;
    Ok(Json(result))
}

pub async fn list_user_upi_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::user_upi_ids::Model>>, ApiError> {
    let result = SmartMerge::list_user_upi_ids(&state.db, &session.user.id).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct AddUpiRequest {
    pub upi_id: String,
    pub label: Option<String>,
}

pub async fn add_user_upi_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<AddUpiRequest>,
) -> Result<Json<db::entities::user_upi_ids::Model>, ApiError> {
    let result =
        SmartMerge::add_user_upi_id(&state.db, &session.user.id, payload.upi_id, payload.label)
            .await?;
    Ok(Json(result))
}

pub async fn make_primary_upi_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    SmartMerge::make_primary_upi(&state.db, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
