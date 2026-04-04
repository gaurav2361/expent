use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use db::SmartMerge;
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub async fn list_confirmed_subscriptions_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::subscriptions::Model>>, ApiError> {
    let result = SmartMerge::list_confirmed_subscriptions(&state.db, &session.user.id).await?;
    Ok(Json(result))
}

pub async fn detect_subscriptions_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::subscriptions::Model>>, ApiError> {
    let result = SmartMerge::detect_subscriptions(&state.db, &session.user.id).await?;

    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct ConfirmSubscriptionRequest {
    pub name: String,
    pub amount: rust_decimal::Decimal,
    pub cycle: String,
    pub start_date: chrono::DateTime<chrono::FixedOffset>,
    pub next_charge_date: chrono::DateTime<chrono::FixedOffset>,
    pub keywords: Option<serde_json::Value>,
}

pub async fn confirm_subscription_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<ConfirmSubscriptionRequest>,
) -> Result<Json<db::entities::subscriptions::Model>, ApiError> {
    let result = SmartMerge::confirm_subscription(
        &state.db,
        &session.user.id,
        payload.name,
        payload.amount,
        payload.cycle,
        payload.start_date,
        payload.next_charge_date,
        payload.keywords,
    )
    .await?;
    Ok(Json(result))
}

pub async fn stop_tracking_subscription_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    SmartMerge::stop_tracking_subscription(&state.db, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct ConfigureAlertRequest {
    pub days_before: i32,
    pub channel: String,
}

pub async fn configure_subscription_alert_handler(
    State(state): State<AppState>,
    _session: AuthSession,
    Path(id): Path<String>,
    Json(payload): Json<ConfigureAlertRequest>,
) -> Result<Json<db::entities::sub_alerts::Model>, ApiError> {
    let result = SmartMerge::configure_subscription_alert(
        &state.db,
        &id,
        payload.days_before,
        payload.channel,
    )
    .await?;
    Ok(Json(result))
}
