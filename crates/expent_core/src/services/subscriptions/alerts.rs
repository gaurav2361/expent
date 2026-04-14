use db::AppError;
use db::entities;
use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Iden, Set};

pub async fn configure_subscription_alert(
    db: &DatabaseConnection,
    sub_id: &str,
    days_before: i32,
    channel: String,
) -> Result<entities::sub_alerts::Model, AppError> {
    let alert = entities::sub_alerts::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        subscription_id: Set(sub_id.to_string()),
        days_before: Set(days_before),
        channel: Set(channel),
        sent_at: Set(None),
    };
    alert.insert(db).await.map_err(AppError::from)
}
