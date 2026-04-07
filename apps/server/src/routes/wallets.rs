use axum::Router;
use axum::extract::{Json, Path, State};
use axum::routing::{delete, get, put};
use db::SmartMerge;
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_wallets_handler).post(create_wallet_handler))
        .route(
            "/{id}",
            put(update_wallet_handler).delete(delete_wallet_handler),
        )
}

pub async fn list_wallets_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::wallets::Model>>, ApiError> {
    let result = SmartMerge::list_wallets(&state.db, &session.user.id).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct CreateWalletRequest {
    pub name: String,
    pub r#type: db::entities::enums::WalletType,
    pub initial_balance: rust_decimal::Decimal,
}

pub async fn create_wallet_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<Json<db::entities::wallets::Model>, ApiError> {
    let result = SmartMerge::create_wallet(
        &state.db,
        &session.user.id,
        &payload.name,
        payload.r#type,
        payload.initial_balance,
    )
    .await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct UpdateWalletRequest {
    pub name: Option<String>,
    pub balance: Option<rust_decimal::Decimal>,
}

pub async fn update_wallet_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    Json(payload): Json<UpdateWalletRequest>,
) -> Result<Json<db::entities::wallets::Model>, ApiError> {
    let result = SmartMerge::update_wallet(
        &state.db,
        &session.user.id,
        &id,
        payload.name,
        payload.balance,
    )
    .await?;
    Ok(Json(result))
}

pub async fn delete_wallet_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<Json<u64>, ApiError> {
    let result = SmartMerge::delete_wallet(&state.db, &session.user.id, &id).await?;
    Ok(Json(result))
}
