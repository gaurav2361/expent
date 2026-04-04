use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
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
    let result = SmartMerge::create_group(
        &state.db,
        &session.user.id,
        &payload.name,
        payload.description,
    )
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

pub async fn list_group_members_handler(
    State(state): State<AppState>,
    _session: AuthSession,
    Path(id): Path<String>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    use sea_orm::{ColumnTrait, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};

    // Join user_groups with users to get member details
    let results = db::entities::user_groups::Entity::find()
        .filter(db::entities::user_groups::Column::GroupId.eq(id))
        .join(
            JoinType::InnerJoin,
            db::entities::user_groups::Relation::Users.def(),
        )
        .column_as(db::entities::users::Column::Name, "name")
        .column_as(db::entities::users::Column::Email, "email")
        .column_as(db::entities::user_groups::Column::UserId, "user_id")
        .column_as(db::entities::user_groups::Column::Role, "role")
        .into_model::<serde_json::Value>()
        .all(&state.db)
        .await?;

    Ok(Json(results))
}

pub async fn remove_group_member_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path((group_id, user_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    SmartMerge::remove_group_member(&state.db, &session.user.id, &group_id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct UpdateMemberRoleRequest {
    pub role: db::entities::enums::GroupRole,
}

pub async fn update_member_role_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path((group_id, user_id)): Path<(String, String)>,
    Json(payload): Json<UpdateMemberRoleRequest>,
) -> Result<StatusCode, ApiError> {
    SmartMerge::update_member_role(
        &state.db,
        &session.user.id,
        &group_id,
        &user_id,
        payload.role,
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
