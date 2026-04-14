use db::AppError;
use db::entities;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Iden, Set};

pub async fn create_category(
    db: &DatabaseConnection,
    user_id: &str,
    name: String,
    icon: Option<String>,
    color: Option<String>,
) -> Result<entities::categories::Model, AppError> {
    let category = entities::categories::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(name),
        icon: Set(icon),
        color: Set(color),
    };
    category.insert(db).await.map_err(AppError::from)
}
