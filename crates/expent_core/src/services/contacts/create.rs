use db::AppError;
use db::entities;
use sea_orm::{DatabaseConnection, Set, Iden, EntityTrait, ActiveModelTrait};

pub async fn create_contact(
    db: &DatabaseConnection,
    user_id: &str,
    name: String,
    phone: Option<String>,
) -> Result<entities::contacts::Model, AppError> {
    let contact = entities::contacts::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        name: Set(name),
        phone: Set(phone),
        is_pinned: Set(false),
    };
    let result = contact.insert(db).await?;

    let link = entities::contact_links::ActiveModel {
        user_id: Set(user_id.to_string()),
        contact_id: Set(result.id.clone()),
    };
    link.insert(db).await?;

    Ok(result)
}
