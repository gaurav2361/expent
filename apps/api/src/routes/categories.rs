use axum::Router;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, get};
use expent_core::categories;
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(list_categories_handler).post(create_category_handler),
        )
        .route("/{id}", delete(delete_category_handler))
}

pub async fn list_categories_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::categories::Model>>, ApiError> {
    let result = categories::list_categories(&state.core.db, &session.user.id).await?;
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
    let result = categories::create_category(
        &state.core.db,
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
    categories::delete_category(&state.core.db, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
