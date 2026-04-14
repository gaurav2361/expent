use db::AppError;
use db::entities;
use sea_orm::{DatabaseConnection, EntityTrait, Iden, Set, ActiveModelTrait};

pub async fn add_contact_identifier(
    db: &DatabaseConnection,
    user_id: &str,
    contact_id: &str,
    r#type: String,
    value: String,
) -> Result<entities::contact_identifiers::Model, AppError> {
    let _link =
        entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Contact link not found"))?;

    let identifier = entities::contact_identifiers::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        contact_id: Set(contact_id.to_string()),
        r#type: Set(r#type),
        value: Set(value),
        linked_user_id: Set(None),
    };

    identifier.insert(db).await.map_err(AppError::from)
}
