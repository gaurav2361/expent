use chrono::Utc;
use db::AppError;
use db::entities;
use sea_orm::{DatabaseConnection, EntityTrait, Iden, ActiveEnum, Set, ActiveModelTrait};

pub async fn update_profile(
    db: &DatabaseConnection,
    user_id: &str,
    name: Option<String>,
    username: Option<String>,
    image: Option<String>,
) -> Result<entities::users::Model, AppError> {
    let mut user: entities::users::ActiveModel =
        entities::users::Entity::find_by_id(user_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("User not found"))?
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

    user.update(db).await.map_err(AppError::from)
}
