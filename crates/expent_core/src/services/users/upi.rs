use db::AppError;
use db::entities;
use sea_orm::prelude::Expr;
use sea_orm::{DatabaseConnection, QueryFilter, EntityTrait, ColumnTrait, Set, Iden, ActiveModelTrait};

pub async fn list_user_upi(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::user_upi_ids::Model>, AppError> {
    entities::user_upi_ids::Entity::find()
        .filter(entities::user_upi_ids::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(AppError::from)
}

pub async fn add_user_upi(
    db: &DatabaseConnection,
    user_id: &str,
    upi_id: String,
    label: Option<String>,
) -> Result<entities::user_upi_ids::Model, AppError> {
    let upi = entities::user_upi_ids::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        upi_id: Set(upi_id),
        is_primary: Set(false),
        label: Set(label),
    };
    upi.insert(db).await.map_err(AppError::from)
}

pub async fn make_primary_upi(
    db: &DatabaseConnection,
    user_id: &str,
    upi_id: &str,
) -> Result<(), AppError> {
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
