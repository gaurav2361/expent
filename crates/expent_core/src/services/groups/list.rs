use db::AppError;
use db::entities;
use sea_orm::{DatabaseConnection, QueryFilter, EntityTrait, ColumnTrait};

pub async fn list_groups(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::groups::Model>, AppError> {
    entities::groups::Entity::find()
        .inner_join(entities::user_groups::Entity)
        .filter(entities::user_groups::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(AppError::from)
}
