use chrono::{DateTime, FixedOffset};
use db::AppError;
use db::entities;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Iden, QueryFilter, Set,
};

pub async fn list_confirmed_subscriptions(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::subscriptions::Model>, AppError> {
    entities::subscriptions::Entity::find()
        .filter(entities::subscriptions::Column::UserId.eq(user_id))
        .all(db)
        .await
        .map_err(AppError::from)
}

pub async fn confirm_subscription(
    db: &DatabaseConnection,
    user_id: &str,
    name: String,
    amount: Decimal,
    cycle: String,
    start_date: DateTime<FixedOffset>,
    next_charge_date: DateTime<FixedOffset>,
    keywords: Option<serde_json::Value>,
) -> Result<entities::subscriptions::Model, AppError> {
    let sub = entities::subscriptions::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(name),
        amount: Set(amount),
        cycle: Set(cycle),
        start_date: Set(start_date),
        next_charge_date: Set(next_charge_date),
        detection_keywords: Set(keywords),
    };
    sub.insert(db).await.map_err(AppError::from)
}

pub async fn stop_tracking_subscription(
    db: &DatabaseConnection,
    user_id: &str,
    sub_id: &str,
) -> Result<(), AppError> {
    entities::subscriptions::Entity::delete_many()
        .filter(entities::subscriptions::Column::Id.eq(sub_id))
        .filter(entities::subscriptions::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}
