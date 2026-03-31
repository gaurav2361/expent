use sea_orm::*;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use chrono::{DateTime, FixedOffset};

pub mod entities;

#[derive(Debug, Serialize, Deserialize)]
pub struct OcrResult {
    pub raw_text: String,
    pub amount: Option<Decimal>,
    pub date: Option<DateTime<FixedOffset>>,
    pub upi_id: Option<String>,
}

pub struct SmartMerge;

impl SmartMerge {
    pub async fn process_ocr(
        db: &DatabaseConnection,
        user_id: &str,
        ocr_data: OcrResult,
    ) -> Result<entities::transaction::Model, DbErr> {
        // 1. Check for existing transactions with same amount and within ±48h window
        let start_date = ocr_data.date.map(|d| d - chrono::Duration::hours(48));
        let end_date = ocr_data.date.map(|d| d + chrono::Duration::hours(48));

        let mut query = entities::transaction::Entity::find()
            .filter(entities::transaction::Column::UserId.eq(user_id));

        if let Some(amount) = ocr_data.amount {
            query = query.filter(entities::transaction::Column::Amount.eq(amount));
        }

        if let (Some(start), Some(end)) = (start_date, end_date) {
            query = query.filter(entities::transaction::Column::Date.between(start, end));
        }

        let existing_txns = query.all(db).await?;

        // 2. If existing found, link to it (Smart Merge)
        if !existing_txns.is_empty() {
            // For now, just return the first match
            // In reality, we'd check UPI ID in metadata
            let txn = existing_txns[0].clone();
            
            // Link new source to existing txn
            let source = entities::transaction_source::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(txn.id.clone()),
                source_type: Set("OCR_SCREENSHOT_MERGED".to_string()),
                r2_file_url: Set(None),
                raw_metadata: Set(Some(serde_json::to_value(&ocr_data).unwrap())),
            };
            source.insert(db).await?;
            
            return Ok(txn);
        }

        // 3. Create NEW transaction if no match
        let new_txn = entities::transaction::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            amount: Set(ocr_data.amount.unwrap_or(Decimal::ZERO)),
            direction: Set("OUT".to_string()), 
            date: Set(ocr_data.date.unwrap_or_else(|| chrono::Utc::now().into())),
            source: Set("OCR".to_string()),
            status: Set("COMPLETED".to_string()),
            purpose_tag: Set(None),
        };

        let result = new_txn.insert(db).await?;

        // 4. Save raw metadata
        let metadata = entities::transaction_metadata::ActiveModel {
            transaction_id: Set(result.id.clone()),
            upi_txn_id: Set(ocr_data.upi_id.clone()), 
            app_txn_id: Set(None),
            app_name: Set(None),
            contact_number: Set(None),
        };
        metadata.insert(db).await?;

        let source = entities::transaction_source::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            transaction_id: Set(result.id.clone()),
            source_type: Set("OCR_SCREENSHOT".to_string()),
            r2_file_url: Set(None),
            raw_metadata: Set(Some(serde_json::to_value(&ocr_data).unwrap())),
        };
        source.insert(db).await?;

        Ok(result)
    }

    pub async fn create_p2p_request(
        db: &DatabaseConnection,
        sender_id: &str,
        receiver_email: &str,
        txn_id: &str,
    ) -> Result<entities::p2p_request::Model, DbErr> {
        // Fetch original transaction
        let txn = entities::transaction::Entity::find_by_id(txn_id.to_string())
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Transaction not found".to_string()))?;

        // Create request
        let request = entities::p2p_request::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            sender_user_id: Set(sender_id.to_string()),
            receiver_email: Set(receiver_email.to_string()),
            transaction_data: Set(serde_json::to_value(&txn).unwrap()),
            status: Set("PENDING".to_string()),
            linked_txn_id: Set(None),
        };

        request.insert(db).await
    }
}
