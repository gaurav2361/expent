use db::AppError;
use db::entities;
use sea_orm::*;

pub async fn list_contacts(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::contacts::Model>, AppError> {
    entities::contacts::Entity::find()
        .join(
            JoinType::InnerJoin,
            entities::contacts::Relation::ContactLinks.def(),
        )
        .filter(entities::contact_links::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(AppError::from)
}
