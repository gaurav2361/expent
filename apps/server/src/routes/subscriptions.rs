use axum::extract::{Json, State};
use db::SmartMerge;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub async fn detect_subscriptions_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::subscriptions::Model>>, ApiError> {
    let result = SmartMerge::detect_subscriptions(&state.db, &session.user.id).await?;

    Ok(Json(result))
}
