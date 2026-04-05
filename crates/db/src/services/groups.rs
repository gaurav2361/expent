use crate::entities;
use crate::entities::enums::{GroupRole, P2PRequestStatus};
use chrono::Utc;
use sea_orm::*;

pub async fn list_groups(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::groups::Model>, DbErr> {
    entities::groups::Entity::find()
        .inner_join(entities::user_groups::Entity)
        .filter(entities::user_groups::Column::UserId.eq(user_id))
        .all(db)
        .await
}

pub async fn create_group(
    db: &DatabaseConnection,
    user_id: &str,
    name: &str,
    description: Option<String>,
) -> Result<entities::groups::Model, DbErr> {
    let group = entities::groups::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        name: Set(name.to_string()),
        description: Set(description),
        created_at: Set(Utc::now().into()),
    };
    let result = group.insert(db).await?;

    let user_group = entities::user_groups::ActiveModel {
        user_id: Set(user_id.to_string()),
        group_id: Set(result.id.clone()),
        role: Set(GroupRole::Admin.to_string()),
    };
    user_group.insert(db).await?;

    Ok(result)
}

pub async fn invite_to_group(
    db: &DatabaseConnection,
    sender_id: &str,
    receiver_email: &str,
    group_id: &str,
) -> Result<entities::p2p_requests::Model, DbErr> {
    let group = entities::groups::Entity::find_by_id(group_id.to_string())
        .one(db)
        .await?
        .ok_or(DbErr::Custom("Group not found".to_string()))?;

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

    request.insert(db).await
}

pub async fn list_group_transactions(
    db: &DatabaseConnection,
    group_id: &str,
) -> Result<Vec<entities::transactions::Model>, DbErr> {
    entities::transactions::Entity::find()
        .filter(entities::transactions::Column::GroupId.eq(group_id))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .order_by_desc(entities::transactions::Column::Date)
        .all(db)
        .await
}

pub async fn remove_group_member(
    db: &DatabaseConnection,
    admin_id: &str,
    group_id: &str,
    target_user_id: &str,
) -> Result<(), DbErr> {
    // Verify admin permissions
    let admin_membership =
        entities::user_groups::Entity::find_by_id((admin_id.to_string(), group_id.to_string()))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Admin not in group".to_string()))?;

    if admin_membership.role != GroupRole::Admin.to_string() {
        return Err(DbErr::Custom("Insufficient permissions".to_string()));
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
) -> Result<(), DbErr> {
    // Verify admin permissions
    let admin_membership =
        entities::user_groups::Entity::find_by_id((admin_id.to_string(), group_id.to_string()))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Admin not in group".to_string()))?;

    if admin_membership.role != GroupRole::Admin.to_string() {
        return Err(DbErr::Custom("Insufficient permissions".to_string()));
    }

    let mut membership: entities::user_groups::ActiveModel =
        entities::user_groups::Entity::find_by_id((
            target_user_id.to_string(),
            group_id.to_string(),
        ))
        .one(db)
        .await?
        .ok_or(DbErr::Custom("Member not found".to_string()))?
        .into();

    membership.role = Set(new_role.to_string());
    membership.update(db).await?;
    Ok(())
}
