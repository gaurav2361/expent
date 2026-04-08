use db::AppError;
use db::entities;
use sea_orm::*;

pub async fn list_categories(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::categories::Model>, AppError> {
    entities::categories::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(entities::categories::Column::UserId.eq(user_id))
                .add(entities::categories::Column::UserId.eq("system")),
        )
        .all(db)
        .await
        .map_err(AppError::from)
}
