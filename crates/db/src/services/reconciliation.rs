use crate::entities;
use chrono::{DateTime, Duration, FixedOffset, Utc};
use rust_decimal::Decimal;
use sea_orm::*;

pub async fn list_unmatched_rows(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::bank_statement_rows::Model>, DbErr> {
    entities::bank_statement_rows::Entity::find()
        .filter(entities::bank_statement_rows::Column::UserId.eq(user_id))
        .filter(entities::bank_statement_rows::Column::IsMatched.eq(false))
        .all(db)
        .await
}

pub async fn get_row_matches(
    db: &DatabaseConnection,
    user_id: &str,
    row_id: &str,
) -> Result<Vec<(entities::transactions::Model, i32)>, DbErr> {
    let row = entities::bank_statement_rows::Entity::find_by_id(row_id.to_string())
        .one(db)
        .await?
        .ok_or(DbErr::Custom("Statement row not found".to_string()))?;

    let amount = row.debit.or(row.credit).unwrap_or(Decimal::ZERO);

    // Find potential transactions within +/- 3 days and matching amount
    let start_date = row.date - Duration::days(3);
    let end_date = row.date + Duration::days(3);

    let txns = entities::transactions::Entity::find()
        .filter(entities::transactions::Column::UserId.eq(user_id))
        .filter(entities::transactions::Column::Amount.eq(amount.abs()))
        .filter(entities::transactions::Column::Date.between(start_date, end_date))
        .all(db)
        .await?;

    let mut matches = Vec::new();
    for txn in txns {
        let mut score = 70; // Base score for amount + date range

        if txn.amount == amount.abs() {
            score += 10;
        }
        if txn.date == row.date {
            score += 10;
        }

        // Check narration/description
        if let Some(tag) = &txn.purpose_tag {
            if row.description.to_lowercase().contains(&tag.to_lowercase()) {
                score += 10;
            }
        }

        matches.push((txn, score.min(100)));
    }

    Ok(matches)
}

pub async fn confirm_match(
    db: &DatabaseConnection,
    _user_id: &str,
    row_id: &str,
    txn_id: &str,
    confidence: i32,
) -> Result<(), DbErr> {
    let match_record = entities::statement_txn_matches::ActiveModel {
        row_id: Set(row_id.to_string()),
        transaction_id: Set(txn_id.to_string()),
        confidence: Set(Decimal::from(confidence)),
        matched_at: Set(Utc::now().into()),
    };
    match_record.insert(db).await?;

    let mut row: entities::bank_statement_rows::ActiveModel =
        entities::bank_statement_rows::Entity::find_by_id(row_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Row not found".to_string()))?
            .into();

    row.is_matched = Set(true);
    row.update(db).await?;

    Ok(())
}

pub async fn upload_statement(
    db: &DatabaseConnection,
    user_id: &str,
    date: DateTime<FixedOffset>,
    description: String,
    amount: Decimal,
    _raw_data: Option<serde_json::Value>,
) -> Result<entities::bank_statement_rows::Model, DbErr> {
    let (debit, credit) = if amount < Decimal::ZERO {
        (Some(amount.abs()), None)
    } else {
        (None, Some(amount))
    };

    let row = entities::bank_statement_rows::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        date: Set(date.into()),
        description: Set(description),
        debit: Set(debit),
        credit: Set(credit),
        balance: Set(Decimal::ZERO), // Balance should be updated from statement or ignored for simple matching
        is_matched: Set(false),
    };
    row.insert(db).await
}
