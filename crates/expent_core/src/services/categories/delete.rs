use db::AppError;
use db::entities;
use sea_orm::{DatabaseConnection, QueryFilter, EntityTrait, ColumnTrait};

pub async fn delete_category(
    db: &DatabaseConnection,
    user_id: &str,
    category_id: &str,
) -> Result<(), AppError> {
    entities::categories::Entity::delete_many()
        .filter(entities::categories::Column::Id.eq(category_id))
        .filter(entities::categories::Column::UserId.eq(user_id))
        .filter(entities::categories::Column::UserId.ne("system")) // Extra protection
        .exec(db)
        .await?;
    Ok(())
}
