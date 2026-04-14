use db::AppError;
use db::entities;
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Iden, Iterable, QueryFilter,
    Set, TransactionTrait,
};

pub async fn merge_contacts(
    db: &DatabaseConnection,
    user_id: &str,
    primary_id: &str,
    secondary_id: &str,
) -> Result<entities::contacts::Model, AppError> {
    if primary_id == secondary_id {
        return Err(AppError::validation("Cannot merge a contact into itself"));
    }

    // Verify both contacts belong to the user
    let _primary_link =
        entities::contact_links::Entity::find_by_id((user_id.to_string(), primary_id.to_string()))
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("Primary contact not found or access denied"))?;

    let _secondary_link = entities::contact_links::Entity::find_by_id((
        user_id.to_string(),
        secondary_id.to_string(),
    ))
    .one(db)
    .await?
    .ok_or_else(|| AppError::not_found("Secondary contact not found or access denied"))?;

    // Transaction for safety
    let txn = db.begin().await?;

    // 1. Update txn_parties
    entities::txn_parties::Entity::update_many()
        .col_expr(
            entities::txn_parties::Column::ContactId,
            Expr::value(primary_id.to_string()),
        )
        .filter(entities::txn_parties::Column::ContactId.eq(secondary_id))
        .exec(&txn)
        .await?;

    // 2. Update contact_identifiers
    // First get existing primary identifiers to avoid duplicates based on value and type
    let primary_identifiers = entities::contact_identifiers::Entity::find()
        .filter(entities::contact_identifiers::Column::ContactId.eq(primary_id))
        .all(&txn)
        .await?;

    let secondary_identifiers = entities::contact_identifiers::Entity::find()
        .filter(entities::contact_identifiers::Column::ContactId.eq(secondary_id))
        .all(&txn)
        .await?;

    for sec_id in secondary_identifiers {
        let is_duplicate = primary_identifiers
            .iter()
            .any(|p| p.r#type == sec_id.r#type && p.value == sec_id.value);

        if is_duplicate {
            // Delete the duplicate from secondary
            entities::contact_identifiers::Entity::delete_by_id(sec_id.id)
                .exec(&txn)
                .await?;
        } else {
            // Move to primary
            let mut active_model: entities::contact_identifiers::ActiveModel = sec_id.into();
            active_model.contact_id = Set(primary_id.to_string());
            active_model.update(&txn).await?;
        }
    }

    // 3. Move the phone number if primary doesn't have one and secondary does
    let mut primary_contact: entities::contacts::ActiveModel =
        entities::contacts::Entity::find_by_id(primary_id.to_string())
            .one(&txn)
            .await?
            .unwrap()
            .into();

    let secondary_contact = entities::contacts::Entity::find_by_id(secondary_id.to_string())
        .one(&txn)
        .await?
        .unwrap();

    let mut updated_primary = false;
    if primary_contact.phone.as_ref().is_none() && secondary_contact.phone.is_some() {
        primary_contact.phone = Set(secondary_contact.phone);
        updated_primary = true;
    }

    let final_primary = if updated_primary {
        primary_contact.update(&txn).await?
    } else {
        entities::contacts::Entity::find_by_id(primary_id.to_string())
            .one(&txn)
            .await?
            .unwrap()
    };

    // 4. Delete secondary contact_links
    entities::contact_links::Entity::delete_by_id((user_id.to_string(), secondary_id.to_string()))
        .exec(&txn)
        .await?;

    // 5. Delete secondary contact
    entities::contacts::Entity::delete_by_id(secondary_id.to_string())
        .exec(&txn)
        .await?;

    txn.commit().await?;

    Ok(final_primary)
}
