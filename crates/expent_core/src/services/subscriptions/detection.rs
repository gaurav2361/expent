use chrono::{DateTime, Duration, FixedOffset, Utc};
use db::AppError;
use db::entities;
use db::entities::enums::SubscriptionCycle;
use rust_decimal::Decimal;
use sea_orm::{DatabaseConnection, QueryFilter, EntityTrait, ColumnTrait, ActiveModelBehavior, Iden};
use std::collections::HashMap;

pub async fn detect_subscriptions(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::subscriptions::Model>, AppError> {
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
