use chrono::{DateTime, FixedOffset, Utc};
use db::entities;
use db::entities::enums::{TransactionDirection, TransactionSource, TransactionStatus};
use db::{AppError, GPayExtraction, OcrResult, OcrTransactionResponse, ProcessedOcr};
use rust_decimal::Decimal;
use sea_orm::{
    ActiveEnum, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Iden, QueryFilter,
    Set, TransactionError, TransactionTrait,
};

/// Processes OCR data by either merging it with an existing transaction or creating a new one.
pub async fn process_ocr(
    db: &DatabaseConnection,
    user_id: &str,
    processed: ProcessedOcr,
) -> Result<OcrTransactionResponse, AppError> {
    let user_id = user_id.to_string();

    // 3.2 Idempotency check: if r2_key is provided, check if we already have a transaction for it
    if let Some(ref key) = processed.r2_key {
        let existing_source = entities::transaction_sources::Entity::find()
            .filter(entities::transaction_sources::Column::R2FileUrl.eq(key))
            .one(db)
            .await?;

        if let Some(source) = existing_source {
            let txn = entities::transactions::Entity::find_by_id(source.transaction_id)
                .one(db)
                .await?
                .ok_or_else(|| {
                    AppError::Generic("Source exists but transaction not found".to_string())
                })?;

            return Ok(OcrTransactionResponse {
                transaction: txn,
                contact_created: false,
            });
        }
    }

    db.transaction::<_, OcrTransactionResponse, AppError>(|txn_db| {
        Box::pin(async move {
            let mut contact_created = false;

            if processed.doc_type == "GPAY" {
                let gpay: GPayExtraction = serde_json::from_value(processed.data.0.clone())
                    .map_err(|e| AppError::Generic(format!("Failed to parse GPay data: {e}")))?;

                let mut contact_id = gpay.contact_id.clone();
                let wallet_id = gpay.wallet_id.clone();
                let category_id = gpay.category_id.clone();

                // 2.6 Robust Contact Resolution
                if contact_id.is_none() {
                    let resolution = crate::services::contacts::resolve_contact(
                        txn_db,
                        &user_id,
                        crate::services::contacts::ResolveParams {
                            name: Some(gpay.counterparty_name.clone()),
                            phone: gpay.counterparty_phone.clone(),
                            email: None,
                            upi_id: gpay.counterparty_upi_id.clone(),
                        },
                    )
                    .await?;

                    if resolution.is_collision {
                        return Err(AppError::Generic("CONTACT_COLLISION".to_string()));
                    }

                    if let Some(c_id) = resolution.contact_id {
                        contact_id = Some(c_id);
                    } else {
                        // Create new contact
                        let new_contact = entities::contacts::ActiveModel {
                            id: Set(uuid::Uuid::now_v7().to_string()),
                            name: Set(gpay.counterparty_name.clone()),
                            phone: Set(gpay.counterparty_phone.clone()),
                            is_pinned: Set(false),
                        };
                        let c_result = new_contact.insert(txn_db).await?;
                        contact_id = Some(c_result.id.clone());
                        contact_created = true;

                        // Create identifier
                        if let Some(upi_id) = &gpay.counterparty_upi_id {
                            let new_ident = entities::contact_identifiers::ActiveModel {
                                id: Set(uuid::Uuid::now_v7().to_string()),
                                contact_id: Set(c_result.id.clone()),
                                r#type: Set("UPI".to_string()),
                                value: Set(upi_id.clone()),
                                linked_user_id: Set(None),
                            };
                            new_ident.insert(txn_db).await?;
                        }

                        // Create link for user
                        let new_link = entities::contact_links::ActiveModel {
                            user_id: Set(user_id.clone()),
                            contact_id: Set(c_result.id),
                        };
                        new_link.insert(txn_db).await?;
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

                let (source_wallet_id, destination_wallet_id): (Option<String>, Option<String>) =
                    if direction == TransactionDirection::In {
                        (None, wallet_id.clone())
                    } else {
                        (wallet_id.clone(), None)
                    };

                let txn = entities::transactions::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    user_id: Set(user_id.clone()),
                    amount: Set(gpay.amount),
                    direction: Set(direction),
                    date: Set(date),
                    source: Set(TransactionSource::Ocr),
                    status: Set(TransactionStatus::Completed),
                    category_id: Set(category_id),
                    purpose_tag: Set(None),
                    group_id: Set(None),
                    source_wallet_id: Set(source_wallet_id.clone()),
                    destination_wallet_id: Set(destination_wallet_id.clone()),
                    ledger_tab_id: Set(None),
                    deleted_at: Set(None),
                    notes: Set(None),
                };

                let result = txn.insert(txn_db).await?;

                // Adjust wallet balances
                crate::services::transactions::adjust_transaction_wallets(
                    txn_db,
                    None,
                    Some(&result),
                )
                .await?;

                // 2.7 Store r2_file_url in transaction_sources
                let source = entities::transaction_sources::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    transaction_id: Set(result.id.clone()),
                    source_type: Set("GPAY_OCR".to_string()),
                    r2_file_url: Set(processed.r2_key),
                    raw_metadata: Set(Some(processed.data.0.clone())),
                };
                source.insert(txn_db).await?;

                // 2.6 Create txn_parties record
                if let Some(c_id) = contact_id {
                    let party = entities::txn_parties::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        transaction_id: Set(result.id.clone()),
                        user_id: Set(None),
                        contact_id: Set(Some(c_id)),
                        role: Set("COUNTERPARTY".to_string()),
                    };
                    party.insert(txn_db).await?;
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
                transfer.insert(txn_db).await?;

                if gpay.is_merchant {
                    let purchase = entities::purchases::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        transaction_id: Set(result.id.clone()),
                        vendor: Set(gpay.counterparty_name),
                        total: Set(gpay.amount),
                        order_id: Set(None),
                    };
                    purchase.insert(txn_db).await?;
                }

                Ok(OcrTransactionResponse {
                    transaction: result,
                    contact_created,
                })
            } else {
                // Generic OCR path
                let generic: OcrResult = serde_json::from_value(processed.data.0.clone())
                    .map_err(|e| AppError::Generic(format!("Failed to parse Generic data: {e}")))?;

                let mut contact_id = generic.contact_id.clone();
                let wallet_id = generic.wallet_id.clone();
                let category_id = generic.category_id.clone();

                // 2.6 Robust Contact Resolution for Generic OCR
                if contact_id.is_none() && generic.vendor.is_some() {
                    let resolution = crate::services::contacts::resolve_contact(
                        txn_db,
                        &user_id,
                        crate::services::contacts::ResolveParams {
                            name: generic.vendor.clone(),
                            phone: None,
                            email: None,
                            upi_id: generic.upi_id.clone(),
                        },
                    )
                    .await?;

                    if resolution.is_collision {
                        return Err(AppError::Generic("CONTACT_COLLISION".to_string()));
                    }

                    if let Some(c_id) = resolution.contact_id {
                        contact_id = Some(c_id);
                    }
                }

                let amount = generic.amount.unwrap_or(Decimal::ZERO);

                let txn = entities::transactions::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    user_id: Set(user_id.clone()),
                    amount: Set(amount),
                    direction: Set(TransactionDirection::Out),
                    date: Set(generic.date.unwrap_or_else(|| Utc::now().into())),
                    source: Set(TransactionSource::Ocr),
                    status: Set(TransactionStatus::Completed),
                    category_id: Set(category_id),
                    purpose_tag: Set(generic.vendor.clone()),
                    group_id: Set(None),
                    source_wallet_id: Set(wallet_id.clone()),
                    destination_wallet_id: Set(None),
                    ledger_tab_id: Set(None),
                    deleted_at: Set(None),
                    notes: Set(None),
                };

                let result = txn.insert(txn_db).await?;

                // Adjust wallet balances
                crate::services::transactions::adjust_transaction_wallets(
                    txn_db,
                    None,
                    Some(&result),
                )
                .await?;

                let source = entities::transaction_sources::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    transaction_id: Set(result.id.clone()),
                    source_type: Set("GENERIC_OCR".to_string()),
                    r2_file_url: Set(processed.r2_key),
                    raw_metadata: Set(Some(processed.data.0)),
                };
                source.insert(txn_db).await?;

                // Create txn_parties record if contact_id is provided
                if let Some(c_id) = contact_id {
                    let party = entities::txn_parties::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        transaction_id: Set(result.id.clone()),
                        user_id: Set(None),
                        contact_id: Set(Some(c_id)),
                        role: Set("COUNTERPARTY".to_string()),
                    };
                    party.insert(txn_db).await?;
                }

                let purchase = entities::purchases::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    transaction_id: Set(result.id.clone()),
                    vendor: Set(generic.vendor.unwrap_or_else(|| "Unknown".to_string())),
                    total: Set(amount),
                    order_id: Set(None),
                };
                let p_result = purchase.insert(txn_db).await?;

                for item in generic.items {
                    let p_item = entities::purchase_items::ActiveModel {
                        id: Set(uuid::Uuid::now_v7().to_string()),
                        purchase_id: Set(p_result.id.clone()),
                        name: Set(item.name),
                        quantity: Set(item.quantity),
                        price: Set(item.price),
                        sku: Set(None),
                    };
                    p_item.insert(txn_db).await?;
                }

                Ok(OcrTransactionResponse {
                    transaction: result,
                    contact_created: false,
                })
            }
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(ce) => AppError::Db(ce),
        TransactionError::Transaction(te) => te,
    })
}
