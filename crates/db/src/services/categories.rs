use crate::entities;
use sea_orm::*;

pub async fn list_categories(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::categories::Model>, DbErr> {
    entities::categories::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(entities::categories::Column::UserId.eq(user_id))
                .add(entities::categories::Column::UserId.eq("system")),
        )
        .all(db)
        .await
}

pub async fn ensure_system_categories(db: &DatabaseConnection) -> Result<(), DbErr> {
    // Ensure a "system" user exists so the FK constraint is satisfied
    let system_user = crate::entities::users::Entity::find_by_id("system".to_string())
        .one(db)
        .await?;
    if system_user.is_none() {
        let now = chrono::Utc::now().fixed_offset();
        let user = crate::entities::users::ActiveModel {
            id: Set("system".to_string()),
            name: Set("System".to_string()),
            email: Set("system@expent.internal".to_string()),
            email_verified: Set(true),
            image: Set(None),
            phone: Set(None),
            is_active: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
            username: Set(Some("system".to_string())),
            display_username: Set(Some("System".to_string())),
            role: Set(None),
            banned: Set(None),
            ban_reason: Set(None),
            ban_expires: Set(None),
            two_factor_enabled: Set(None),
            phone_number: Set(None),
            phone_number_verified: Set(None),
            metadata: Set(None),
            associated_contact_id: Set(None),
        };
        user.insert(db).await?;
    }

    let system_cats = vec![
        ("cat-sub-0001", "Subscription", "calendar", "#3b82f6"),
        ("cat-rnt-0002", "Rent & EMI", "home", "#8b5cf6"),
        ("cat-fod-0003", "Food & Dining", "coffee", "#f97316"),
        ("cat-trn-0004", "Transport", "car", "#eab308"),
        ("cat-ent-0005", "Entertainment", "tv", "#ec4899"),
        ("cat-gro-0006", "Groceries", "shopping-cart", "#10b981"),
        ("cat-hth-0007", "Health & Medical", "activity", "#ef4444"),
    ];

    for (id, name, icon, color) in system_cats {
        let exists = entities::categories::Entity::find_by_id(id.to_string())
            .one(db)
            .await?;
        if exists.is_none() {
            let cat = entities::categories::ActiveModel {
                id: Set(id.to_string()),
                user_id: Set("system".to_string()),
                name: Set(name.to_string()),
                icon: Set(Some(icon.to_string())),
                color: Set(Some(color.to_string())),
            };
            cat.insert(db).await?;
        }
    }
    Ok(())
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
        .filter(entities::categories::Column::UserId.ne("system")) // Extra protection
        .exec(db)
        .await?;
    Ok(())
}
