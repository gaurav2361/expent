use crate::entities;
use crate::entities::enums::SubscriptionCycle;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rust_decimal::Decimal;
use sea_orm::*;
use std::collections::HashMap;

pub async fn list_confirmed_subscriptions(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::subscriptions::Model>, DbErr> {
    entities::subscriptions::Entity::find()
        .filter(entities::subscriptions::Column::UserId.eq(user_id))
        .all(db)
        .await
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
) -> Result<entities::subscriptions::Model, DbErr> {
    let sub = entities::subscriptions::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        name: Set(name),
        amount: Set(amount),
        cycle: Set(cycle),
        start_date: Set(start_date.into()),
        next_charge_date: Set(next_charge_date.into()),
        detection_keywords: Set(keywords),
    };
    sub.insert(db).await
}

pub async fn stop_tracking_subscription(
    db: &DatabaseConnection,
    user_id: &str,
    sub_id: &str,
) -> Result<(), DbErr> {
    entities::subscriptions::Entity::delete_many()
        .filter(entities::subscriptions::Column::Id.eq(sub_id))
        .filter(entities::subscriptions::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn configure_subscription_alert(
    db: &DatabaseConnection,
    sub_id: &str,
    days_before: i32,
    channel: String,
) -> Result<entities::sub_alerts::Model, DbErr> {
    let alert = entities::sub_alerts::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        subscription_id: Set(sub_id.to_string()),
        days_before: Set(days_before),
        channel: Set(channel),
        sent_at: Set(None),
    };
    alert.insert(db).await
}

pub async fn detect_subscriptions(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::subscriptions::Model>, DbErr> {
    let ninety_days_ago = Utc::now() - Duration::days(90);
    let transactions = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Date.gte(ninety_days_ago))
        .all(db)
        .await?;

    let mut groups: HashMap<(String, Decimal), Vec<DateTime<FixedOffset>>> = HashMap::new();
    for txn in transactions {
        let name = txn.purpose_tag.unwrap_or_else(|| "Unknown".to_string());
        let entry = groups.entry((name, txn.amount)).or_default();
        entry.push(txn.date);
    }

    let mut potential_subs = Vec::new();
    for ((name, amount), mut dates) in groups {
        if dates.len() >= 2 {
            dates.sort();

            let mut detected_cycle = None;
            let last_date = *dates.last().unwrap();

            for i in 0..dates.len() - 1 {
                let diff = (dates[i + 1] - dates[i]).num_days();

                if (6..=8).contains(&diff) {
                    detected_cycle = Some(SubscriptionCycle::Weekly);
                } else if (27..=33).contains(&diff) {
                    detected_cycle = Some(SubscriptionCycle::Monthly);
                } else if (360..=370).contains(&diff) {
                    detected_cycle = Some(SubscriptionCycle::Yearly);
                }
            }

            if let Some(cycle) = detected_cycle {
                let next_charge = match cycle {
                    SubscriptionCycle::Weekly => last_date + Duration::days(7),
                    SubscriptionCycle::Yearly => last_date + Duration::days(365),
                    _ => last_date + Duration::days(30),
                };

                let sub = entities::subscriptions::Model {
                    id: uuid::Uuid::now_v7().to_string(),
                    user_id: user_id.to_string(),
                    name: name.clone(),
                    amount,
                    cycle: cycle.to_string(),
                    start_date: dates[0],
                    next_charge_date: next_charge,
                    detection_keywords: None,
                };
                potential_subs.push(sub);
            }
        }
    }

    Ok(potential_subs)
}
