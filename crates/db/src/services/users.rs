use crate::entities;
use chrono::Utc;
use sea_orm::prelude::Expr;
use sea_orm::*;

pub async fn update_profile(
    db: &DatabaseConnection,
    user_id: &str,
    name: Option<String>,
    username: Option<String>,
    image: Option<String>,
) -> Result<entities::users::Model, DbErr> {
    let mut user: entities::users::ActiveModel =
        entities::users::Entity::find_by_id(user_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("User not found".to_string()))?
            .into();

    if let Some(n) = name {
        user.name = Set(n);
    }
    if let Some(u) = username {
        user.username = Set(Some(u));
    }
    if let Some(i) = image {
        user.image = Set(Some(i));
    }
    user.updated_at = Set(Utc::now().into());

    user.update(db).await
}

pub async fn list_user_upi(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::user_upi_ids::Model>, DbErr> {
    entities::user_upi_ids::Entity::find()
        .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
        .all(db)
        .await
}

pub async fn add_user_upi(
    db: &DatabaseConnection,
    user_id: &str,
    upi_id: String,
    label: Option<String>,
) -> Result<entities::user_upi_ids::Model, DbErr> {
    let upi = entities::user_upi_ids::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        upi_id: Set(upi_id),
        is_primary: Set(false),
        label: Set(label),
    };
    upi.insert(db).await
}

pub async fn make_primary_upi(
    db: &DatabaseConnection,
    user_id: &str,
    upi_id: &str,
) -> Result<(), DbErr> {
    // Unset current primary
    entities::user_upi_ids::Entity::update_many()
        .col_expr(
            entities::user_upi_ids::Column::IsPrimary,
            Expr::value(false),
        )
        .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
        .exec(db)
        .await?;

    // Set new primary
    entities::user_upi_ids::Entity::update_many()
        .col_expr(entities::user_upi_ids::Column::IsPrimary, Expr::value(true))
        .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
        .filter(entities::user_upi_ids::Column::Id.eq(upi_id))
        .exec(db)
        .await?;

    Ok(())
}
