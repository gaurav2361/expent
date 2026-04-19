use chrono::{DateTime, FixedOffset};
use db::AppError;
use db::entities;
use rust_decimal::Decimal;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

pub async fn upload_statement(
    db: &DatabaseConnection,
    user_id: &str,
    date: DateTime<FixedOffset>,
    description: String,
    amount: Decimal,
    _raw_data: Option<serde_json::Value>,
) -> Result<entities::bank_statement_rows::Model, AppError> {
    let (debit, credit) = if amount < Decimal::ZERO {
        (Some(amount.abs()), None)
    } else {
        (None, Some(amount))
    };

    // Duplicate check: avoid uploading the same row multiple times
    let existing = entities::bank_statement_rows::Entity::find()
        .filter(entities::bank_statement_rows::Column::UserId.eq(user_id))
        .filter(entities::bank_statement_rows::Column::Date.eq(date))
        .filter(entities::bank_statement_rows::Column::Description.eq(description.clone()))
        .filter(entities::bank_statement_rows::Column::Debit.eq(debit))
        .filter(entities::bank_statement_rows::Column::Credit.eq(credit))
        .one(db)
        .await?;

    if let Some(row) = existing {
        tracing::info!("⏭️ Skipping duplicate statement row: {}", description);
        return Ok(row);
    }

    let row = entities::bank_statement_rows::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        date: Set(date),
        description: Set(description),
        debit: Set(debit),
        credit: Set(credit),
        balance: Set(Decimal::ZERO),
        is_matched: Set(false),
    };
    row.insert(db).await.map_err(AppError::from)
}
