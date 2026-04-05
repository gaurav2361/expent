use crate::entities;
use sea_orm::*;

pub async fn list_contacts(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::contacts::Model>, DbErr> {
    entities::contacts::Entity::find()
        .join(
            JoinType::InnerJoin,
            entities::contacts::Relation::ContactLinks.def(),
        )
        .filter(entities::contact_links::Column::UserId.eq(user_id))
        .all(db)
        .await
}

pub async fn create_contact(
    db: &DatabaseConnection,
    user_id: &str,
    name: String,
    phone: Option<String>,
) -> Result<entities::contacts::Model, DbErr> {
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

pub async fn update_contact(
    db: &DatabaseConnection,
    user_id: &str,
    contact_id: &str,
    name: Option<String>,
    phone: Option<String>,
    is_pinned: Option<bool>,
) -> Result<entities::contacts::Model, DbErr> {
    let _link =
        entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Contact link not found".to_string()))?;

    let mut contact: entities::contacts::ActiveModel =
        entities::contacts::Entity::find_by_id(contact_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Contact not found".to_string()))?
            .into();

    if let Some(n) = name {
        contact.name = Set(n);
    }
    if let Some(p) = phone {
        contact.phone = Set(Some(p));
    }
    if let Some(ip) = is_pinned {
        contact.is_pinned = Set(ip);
    }

    contact.update(db).await
}

pub async fn delete_contact(
    db: &DatabaseConnection,
    user_id: &str,
    contact_id: &str,
) -> Result<(), DbErr> {
    entities::contact_links::Entity::delete_by_id((user_id.to_string(), contact_id.to_string()))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn get_contact_detail(
    db: &DatabaseConnection,
    user_id: &str,
    contact_id: &str,
) -> Result<
    (
        entities::contacts::Model,
        Vec<entities::contact_identifiers::Model>,
        Vec<entities::transactions::Model>,
    ),
    DbErr,
> {
    let _link =
        entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Contact link not found".to_string()))?;

    let contact = entities::contacts::Entity::find_by_id(contact_id.to_string())
        .one(db)
        .await?
        .ok_or(DbErr::Custom("Contact not found".to_string()))?;

    let identifiers = entities::contact_identifiers::Entity::find()
        .filter(entities::contact_identifiers::Column::ContactId.eq(contact_id))
        .all(db)
        .await?;

    let transactions = entities::transactions::Entity::find()
        .join(
            JoinType::InnerJoin,
            entities::transactions::Relation::TxnParties.def(),
        )
        .filter(entities::txn_parties::Column::ContactId.eq(contact_id))
        .order_by_desc(entities::transactions::Column::Date)
        .all(db)
        .await?;

    Ok((contact, identifiers, transactions))
}

pub async fn add_contact_identifier(
    db: &DatabaseConnection,
    user_id: &str,
    contact_id: &str,
    r#type: String,
    value: String,
) -> Result<entities::contact_identifiers::Model, DbErr> {
    let _link =
        entities::contact_links::Entity::find_by_id((user_id.to_string(), contact_id.to_string()))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Contact link not found".to_string()))?;

    let identifier = entities::contact_identifiers::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        contact_id: Set(contact_id.to_string()),
        r#type: Set(r#type),
        value: Set(value),
        linked_user_id: Set(None),
    };

    identifier.insert(db).await
}
