use db::AppError;
use db::entities;
use db::entities::enums::{GroupRole, P2PRequestStatus};
use sea_orm::{DatabaseConnection, EntityTrait, Iden, Set, ActiveModelBehavior, ActiveModelTrait, ActiveEnum};

pub async fn invite_to_group(
    db: &DatabaseConnection,
    sender_id: &str,
    receiver_email: &str,
    group_id: &str,
) -> Result<entities::p2p_requests::Model, AppError> {
    let group = entities::groups::Entity::find_by_id(group_id.to_string())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Group not found"))?;

    let request = entities::p2p_requests::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        sender_user_id: Set(sender_id.to_string()),
        receiver_email: Set(receiver_email.to_string()),
        transaction_data: Set(serde_json::json!({
            "type": "GROUP_INVITE",
            "group_id": group.id,
            "group_name": group.name
        })),
        status: Set(P2PRequestStatus::GroupInvite.to_string()),
        linked_txn_id: Set(None),
    };

    request.insert(db).await.map_err(AppError::from)
}

pub async fn remove_group_member(
    db: &DatabaseConnection,
    admin_id: &str,
    group_id: &str,
    target_user_id: &str,
) -> Result<(), AppError> {
    // Verify admin permissions
    let admin_membership =
        entities::user_groups::Entity::find_by_id((admin_id.to_string(), group_id.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| AppError::unauthorized("Admin not in group"))?;

    if admin_membership.role != GroupRole::Admin.to_string() {
        return Err(AppError::unauthorized("Insufficient permissions"));
    }

    entities::user_groups::Entity::delete_by_id((target_user_id.to_string(), group_id.to_string()))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn update_member_role(
    db: &DatabaseConnection,
    admin_id: &str,
    group_id: &str,
    target_user_id: &str,
    new_role: GroupRole,
) -> Result<(), AppError> {
    // Verify admin permissions
    let admin_membership =
        entities::user_groups::Entity::find_by_id((admin_id.to_string(), group_id.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| AppError::unauthorized("Admin not in group"))?;

    if admin_membership.role != GroupRole::Admin.to_string() {
        return Err(AppError::unauthorized("Insufficient permissions"));
    }

    let mut membership: entities::user_groups::ActiveModel =
        entities::user_groups::Entity::find_by_id((
            target_user_id.to_string(),
            group_id.to_string(),
        ))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("Member not found"))?
        .into();

    membership.role = Set(new_role.to_string());
    membership.update(db).await?;
    Ok(())
}
