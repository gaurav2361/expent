use axum::Router;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use expent_core::contacts;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::extractors::ValidatedJson;
use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_contacts_handler).post(create_contact_handler))
        .route("/suggestions", get(get_merge_suggestions_handler))
        .route("/merge", post(merge_contacts_handler))
        .route(
            "/{id}",
            get(get_contact_detail_handler)
                .put(update_contact_handler)
                .delete(delete_contact_handler),
        )
        .route("/{id}/identifiers", post(add_contact_identifier_handler))
}

pub async fn list_contacts_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::contacts::Model>>, ApiError> {
    let result = contacts::list_contacts(&state.core.db, &session.user.id).await?;
    Ok(Json(result))
}

#[derive(Deserialize, Validate)]
pub struct CreateContactRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(min = 10, max = 15))]
    pub phone: Option<String>,
}

pub async fn create_contact_handler(
    State(state): State<AppState>,
    session: AuthSession,
    ValidatedJson(payload): ValidatedJson<CreateContactRequest>,
) -> Result<Json<db::entities::contacts::Model>, ApiError> {
    let result = contacts::create_contact(
        &state.core.db,
        &session.user.id,
        payload.name,
        payload.phone,
    )
    .await?;
    Ok(Json(result))
}

#[derive(Deserialize, Validate)]
pub struct UpdateContactRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(min = 10, max = 15))]
    pub phone: Option<String>,
    pub is_pinned: Option<bool>,
}

pub async fn update_contact_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    ValidatedJson(payload): ValidatedJson<UpdateContactRequest>,
) -> Result<Json<db::entities::contacts::Model>, ApiError> {
    let result = contacts::update_contact(
        &state.core.db,
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
    contacts::delete_contact(&state.core.db, &session.user.id, &id).await?;
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
        contacts::get_contact_detail(&state.core.db, &session.user.id, &id).await?;
    Ok(Json(ContactDetailResponse {
        contact,
        identifiers,
        transactions,
    }))
}

#[derive(Deserialize, Validate)]
pub struct AddIdentifierRequest {
    pub r#type: String,
    #[validate(length(min = 1, max = 255))]
    pub value: String,
}

pub async fn add_contact_identifier_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    ValidatedJson(payload): ValidatedJson<AddIdentifierRequest>,
) -> Result<Json<db::entities::contact_identifiers::Model>, ApiError> {
    let result = contacts::add_contact_identifier(
        &state.core.db,
        &session.user.id,
        &id,
        payload.r#type,
        payload.value,
    )
    .await?;
    Ok(Json(result))
}

pub async fn get_merge_suggestions_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<contacts::MergeSuggestion>>, ApiError> {
    let result = contacts::get_merge_suggestions(&state.core.db, &session.user.id).await?;
    Ok(Json(result))
}

#[derive(Deserialize, Validate)]
pub struct MergeContactsRequest {
    #[validate(length(min = 1, max = 255))]
    pub primary_id: String,
    #[validate(length(min = 1, max = 255))]
    pub secondary_id: String,
}

pub async fn merge_contacts_handler(
    State(state): State<AppState>,
    session: AuthSession,
    ValidatedJson(payload): ValidatedJson<MergeContactsRequest>,
) -> Result<Json<db::entities::contacts::Model>, ApiError> {
    let result = contacts::merge_contacts(
        &state.core.db,
        &session.user.id,
        &payload.primary_id,
        &payload.secondary_id,
    )
    .await?;
    Ok(Json(result))
}
