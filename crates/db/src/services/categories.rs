use crate::entities;
use sea_orm::*;

pub async fn list_categories(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::categories::Model>, DbErr> {
    entities::categories::Entity::find()
        .filter(entities::categories::Column::UserId.eq(user_id))
        .all(db)
        .await
}

pub async fn create_category(
    db: &DatabaseConnection,
    user_id: &str,
    name: String,
    icon: Option<String>,
    color: Option<String>,
) -> Result<entities::categories::Model, DbErr> {
    let category = entities::categories::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(name),
        icon: Set(icon),
        color: Set(color),
    };
    category.insert(db).await
}

pub async fn delete_category(
    db: &DatabaseConnection,
    user_id: &str,
    category_id: &str,
) -> Result<(), DbErr> {
    entities::categories::Entity::delete_many()
        .filter(entities::categories::Column::Id.eq(category_id))
        .filter(entities::categories::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}
