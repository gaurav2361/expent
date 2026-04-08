use db::AppError;
use db::entities;
use sea_orm::*;

pub async fn list_group_transactions(
    db: &DatabaseConnection,
    group_id: &str,
) -> Result<Vec<entities::transactions::Model>, AppError> {
    entities::transactions::Entity::find()
        .filter(entities::transactions::Column::GroupId.eq(group_id))
        .filter(entities::transactions::Column::DeletedAt.is_null())
        .order_by_desc(entities::transactions::Column::Date)
        .all(db)
        .await
        .map_err(AppError::from)
}
