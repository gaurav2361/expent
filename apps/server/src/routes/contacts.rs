use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use db::SmartMerge;
use serde::{Deserialize, Serialize};

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub async fn list_contacts_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::contacts::Model>>, ApiError> {
    let result = SmartMerge::list_contacts(&state.db, &session.user.id).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct CreateContactRequest {
    pub name: String,
    pub phone: Option<String>,
}

pub async fn create_contact_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<CreateContactRequest>,
) -> Result<Json<db::entities::contacts::Model>, ApiError> {
    let result =
        SmartMerge::create_contact(&state.db, &session.user.id, payload.name, payload.phone).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct UpdateContactRequest {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub is_pinned: Option<bool>,
}

pub async fn update_contact_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    Json(payload): Json<UpdateContactRequest>,
) -> Result<Json<db::entities::contacts::Model>, ApiError> {
    let result = SmartMerge::update_contact(
        &state.db,
        &session.user.id,
        &id,
        payload.name,
        payload.phone,
        payload.is_pinned,
    )
    .await?;
    Ok(Json(result))
}

pub async fn delete_contact_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    SmartMerge::delete_contact(&state.db, &session.user.id, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct ContactDetailResponse {
    pub contact: db::entities::contacts::Model,
    pub identifiers: Vec<db::entities::contact_identifiers::Model>,
    pub transactions: Vec<db::entities::transactions::Model>,
}

pub async fn get_contact_detail_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<Json<ContactDetailResponse>, ApiError> {
    let (contact, identifiers, transactions) =
        SmartMerge::get_contact_detail(&state.db, &session.user.id, &id).await?;
    Ok(Json(ContactDetailResponse {
        contact,
        identifiers,
        transactions,
    }))
}

#[derive(Deserialize)]
pub struct AddIdentifierRequest {
    pub r#type: String,
    pub value: String,
}

pub async fn add_contact_identifier_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    Json(payload): Json<AddIdentifierRequest>,
) -> Result<Json<db::entities::contact_identifiers::Model>, ApiError> {
    let result = SmartMerge::add_contact_identifier(
        &state.db,
        &session.user.id,
        &id,
        payload.r#type,
        payload.value,
    )
    .await?;
    Ok(Json(result))
}
