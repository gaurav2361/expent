use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use db::SmartMerge;
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub async fn list_categories_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::categories::Model>>, ApiError> {
    let result = SmartMerge::list_categories(&state.db, &session.user.id).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
}

pub async fn create_category_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<CreateCategoryRequest>,
) -> Result<Json<db::entities::categories::Model>, ApiError> {
    let result = SmartMerge::create_category(
        &state.db,
        &session.user.id,
        payload.name,
        payload.icon,
        payload.color,
    )
    .await?;
    Ok(Json(result))
}

pub async fn delete_category_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    SmartMerge::delete_category(&state.db, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
