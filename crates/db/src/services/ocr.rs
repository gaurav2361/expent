use crate::entities;
use crate::entities::enums::{TransactionDirection, TransactionSource, TransactionStatus};
use crate::{GPayExtraction, OcrResult, OcrTransactionResponse, ProcessedOcr};
use chrono::{DateTime, FixedOffset, Utc};
use rust_decimal::Decimal;
use sea_orm::*;

/// Processes OCR data by either merging it with an existing transaction or creating a new one.
pub async fn process_ocr(
    db: &DatabaseConnection,
    user_id: &str,
    processed: ProcessedOcr,
) -> Result<OcrTransactionResponse, DbErr> {
    let mut contact_id = None;
    let mut contact_created = false;

    if processed.doc_type == "GPAY" {
        let gpay: GPayExtraction = serde_json::from_value(processed.data.clone())
            .map_err(|e| DbErr::Custom(format!("Failed to parse GPay data: {}", e)))?;

        // 2.6 Auto-Contact Creation Logic
        if let Some(upi_id) = &gpay.counterparty_upi_id {
            // Check if identifier exists
            let identifier = entities::contact_identifiers::Entity::find()
                .filter(entities::contact_identifiers::Column::Value.eq(upi_id))
                .one(db)
                .await?;

            if let Some(ident) = identifier {
                contact_id = Some(ident.contact_id);
            } else {
                // Create new contact
                let new_contact = entities::contacts::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    name: Set(gpay.counterparty_name.clone()),
                    phone: Set(gpay.counterparty_phone.clone()),
                    is_pinned: Set(false),
                };
                let c_result = new_contact.insert(db).await?;
                contact_id = Some(c_result.id.clone());
                contact_created = true;

                // Create identifier
                let new_ident = entities::contact_identifiers::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    contact_id: Set(c_result.id.clone()),
                    r#type: Set("UPI".to_string()),
                    value: Set(upi_id.clone()),
                    linked_user_id: Set(None),
                };
                new_ident.insert(db).await?;

                // Create link for user
                let new_link = entities::contact_links::ActiveModel {
                    user_id: Set(user_id.to_string()),
                    contact_id: Set(c_result.id),
                };
                new_link.insert(db).await?;
            }
        }

        let direction = if gpay.direction == "IN" {
            TransactionDirection::In
        } else {
            TransactionDirection::Out
        };

        let date = if let Some(dt_str) = &gpay.datetime_str {
            match chrono::NaiveDateTime::parse_from_str(dt_str, "%d %b %Y, %I:%M %p") {
                Ok(naive) => DateTime::<FixedOffset>::from_naive_utc_and_offset(
                    naive,
                    FixedOffset::east_opt(0).unwrap(),
                ),
                Err(_) => Utc::now().into(),
            }
        } else {
            Utc::now().into()
        };

        let txn = entities::transactions::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            amount: Set(gpay.amount),
            direction: Set(direction),
            date: Set(date),
            source: Set(TransactionSource::Ocr),
            status: Set(TransactionStatus::Completed),
            category_id: Set(None),
            purpose_tag: Set(None),
            group_id: Set(None),
            source_wallet_id: Set(None),
            destination_wallet_id: Set(None),
            ledger_tab_id: Set(None),
            deleted_at: Set(None),
            notes: Set(None),
        };

        let result = txn.insert(db).await?;

        // 2.7 Store r2_file_url in transaction_sources
        let source = entities::transaction_sources::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            transaction_id: Set(result.id.clone()),
            source_type: Set("GPAY_OCR".to_string()),
            r2_file_url: Set(processed.r2_key),
            raw_metadata: Set(Some(processed.data)),
        };
        source.insert(db).await?;

        // 2.6 Create txn_parties record
        if let Some(c_id) = contact_id {
            let party = entities::txn_parties::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                user_id: Set(None),
                contact_id: Set(Some(c_id)),
                role: Set("COUNTERPARTY".to_string()),
            };
            party.insert(db).await?;
        }

        let transfer = entities::p2p_transfers::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            transaction_id: Set(result.id.clone()),
            direction: Set(gpay.direction),
            counterparty_name: Set(gpay.counterparty_name.clone()),
            counterparty_phone: Set(gpay.counterparty_phone),
            counterparty_upi_id: Set(gpay.counterparty_upi_id),
            is_merchant: Set(gpay.is_merchant),
            upi_transaction_id: Set(gpay.upi_transaction_id),
            google_transaction_id: Set(gpay.google_transaction_id),
            source_bank_account: Set(gpay.source_bank_account),
        };
        transfer.insert(db).await?;

        if gpay.is_merchant {
            let purchase = entities::purchases::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                transaction_id: Set(result.id.clone()),
                vendor: Set(gpay.counterparty_name),
                total: Set(gpay.amount),
                order_id: Set(None),
            };
            purchase.insert(db).await?;
        }

        Ok(OcrTransactionResponse {
            transaction: result,
            contact_created,
        })
    } else {
        // Generic OCR path
        let generic: OcrResult = serde_json::from_value(processed.data.clone())
            .map_err(|e| DbErr::Custom(format!("Failed to parse Generic data: {}", e)))?;

        let amount = generic.amount.unwrap_or(Decimal::ZERO);

        let txn = entities::transactions::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            user_id: Set(user_id.to_string()),
            amount: Set(amount),
            direction: Set(TransactionDirection::Out),
            date: Set(generic.date.unwrap_or_else(|| Utc::now().into())),
            source: Set(TransactionSource::Ocr),
            status: Set(TransactionStatus::Completed),
            category_id: Set(None),
            purpose_tag: Set(generic.vendor.clone()),
            group_id: Set(None),
            source_wallet_id: Set(None),
            destination_wallet_id: Set(None),
            ledger_tab_id: Set(None),
            deleted_at: Set(None),
            notes: Set(None),
        };

        let result = txn.insert(db).await?;

        let source = entities::transaction_sources::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            transaction_id: Set(result.id.clone()),
            source_type: Set("GENERIC_OCR".to_string()),
            r2_file_url: Set(processed.r2_key),
            raw_metadata: Set(Some(processed.data)),
        };
        source.insert(db).await?;

        let purchase = entities::purchases::ActiveModel {
            id: Set(uuid::Uuid::now_v7().to_string()),
            transaction_id: Set(result.id.clone()),
            vendor: Set(generic.vendor.unwrap_or_else(|| "Unknown".to_string())),
            total: Set(amount),
            order_id: Set(None),
        };
        let p_result = purchase.insert(db).await?;

        for item in generic.items {
            let p_item = entities::purchase_items::ActiveModel {
                id: Set(uuid::Uuid::now_v7().to_string()),
                purchase_id: Set(p_result.id.clone()),
                name: Set(item.name),
                quantity: Set(item.quantity),
                price: Set(item.price),
                sku: Set(None),
            };
            p_item.insert(db).await?;
        }

        Ok(OcrTransactionResponse {
            transaction: result,
            contact_created: false,
        })
    }
}
