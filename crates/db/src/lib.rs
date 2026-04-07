use crate::entities::enums::{
    GroupRole, LedgerTabType, TransactionDirection, TransactionSource, TransactionStatus,
    WalletType,
};
use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod entities;
pub mod services;

/// Represents a single line item in a purchase, typically extracted via OCR.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "LineItem",
    export_to = "../../../packages/types/src/db/LineItem.ts"
)]
pub struct LineItem {
    pub name: String,
    pub quantity: i32,
    #[ts(type = "string")]
    #[serde(with = "rust_decimal::serde::str")]
    pub price: Decimal,
}

/// The result of an OCR process, containing raw text and extracted transaction details.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "OcrResult",
    export_to = "../../../packages/types/src/db/OcrResult.ts"
)]
pub struct OcrResult {
    pub raw_text: String,
    pub vendor: Option<String>,
    #[ts(type = "string | null")]
    pub amount: Option<Decimal>,
    pub date: Option<DateTime<FixedOffset>>,
    pub upi_id: Option<String>,
    #[serde(default)]
    pub items: Vec<LineItem>,
}

/// Specialized extraction for Google Pay screenshots.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "GPayExtraction",
    export_to = "../../../packages/types/src/db/GPayExtraction.ts"
)]
pub struct GPayExtraction {
    #[ts(type = "string")]
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
    pub direction: String, // "IN" | "OUT"
    pub datetime_str: Option<String>,
    pub status: Option<String>,
    pub counterparty_name: String,
    pub counterparty_phone: Option<String>,
    pub counterparty_upi_id: Option<String>,
    pub is_merchant: bool,
    pub upi_transaction_id: Option<String>,
    pub google_transaction_id: Option<String>,
    pub source_bank_account: Option<String>,
}

/// Unified OCR data from the Python worker.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "ProcessedOcr",
    export_to = "../../../packages/types/src/db/ProcessedOcr.ts"
)]
pub struct ProcessedOcr {
    pub doc_type: String, // "GPAY" or "GENERIC"
    #[ts(type = "any")]
    pub data: serde_json::Value,
    pub r2_key: Option<String>,
}

/// Details for splitting a transaction among multiple users.
#[derive(Debug, Serialize, Deserialize, TS)]
#[ts(
    export,
    rename = "SplitDetail",
    export_to = "../../../packages/types/src/db/SplitDetail.ts"
)]
pub struct SplitDetail {
    pub receiver_email: String,
    #[ts(type = "string")]
    #[serde(with = "rust_decimal::serde::str")]
    pub amount: Decimal,
}

/// P2P request with sender's name.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "P2PRequestWithSender",
    export_to = "../../../packages/types/src/db/P2PRequestWithSender.ts"
)]
pub struct P2PRequestWithSender {
    #[serde(flatten)]
    pub request: entities::p2p_requests::Model,
    pub sender_name: Option<String>,
}

/// Response for OCR transaction creation.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "OcrTransactionResponse",
    export_to = "../../../packages/types/src/db/OcrTransactionResponse.ts"
)]
pub struct OcrTransactionResponse {
    pub transaction: entities::transactions::Model,
    pub contact_created: bool,
}

/// Transaction with optional wallet and contact info.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "TransactionWithDetail",
    export_to = "../../../packages/types/src/db/TransactionWithDetail.ts"
)]
pub struct TransactionWithDetail {
    #[serde(flatten)]
    pub transaction: entities::transactions::Model,
    pub source_wallet_name: Option<String>,
    pub destination_wallet_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_id: Option<String>,
}

/// Business logic for merging and processing transaction data.
pub struct SmartMerge;

impl SmartMerge {
    /// Processes OCR data by either merging it with an existing transaction or creating a new one.
    pub async fn process_ocr(
        db: &DatabaseConnection,
        user_id: &str,
        processed: ProcessedOcr,
    ) -> Result<OcrTransactionResponse, DbErr> {
        services::ocr::process_ocr(db, user_id, processed).await
    }

    /// Creates a peer-to-peer (P2P) request for a given transaction.
    pub async fn create_p2p_request(
        db: &DatabaseConnection,
        sender_id: &str,
        receiver_email: &str,
        txn_id: &str,
    ) -> Result<entities::p2p_requests::Model, DbErr> {
        services::p2p::create_p2p_request(db, sender_id, receiver_email, txn_id).await
    }

    /// Splits a transaction among multiple receivers.
    pub async fn split_transaction(
        db: &DatabaseConnection,
        sender_id: &str,
        txn_id: &str,
        splits: Vec<SplitDetail>,
    ) -> Result<Vec<entities::p2p_requests::Model>, DbErr> {
        services::transactions::split_transaction(db, sender_id, txn_id, splits).await
    }

    /// Accepts a P2P request, potentially creating a mirrored transaction for the receiver.
    pub async fn accept_p2p_request(
        db: &DatabaseConnection,
        receiver_id: &str,
        request_id: &str,
    ) -> Result<entities::p2p_requests::Model, DbErr> {
        services::p2p::accept_p2p_request(db, receiver_id, request_id).await
    }

    /// Detects recurring subscriptions from transaction history over the last 90 days.
    pub async fn detect_subscriptions(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::subscriptions::Model>, DbErr> {
        services::subscriptions::detect_subscriptions(db, user_id).await
    }

    pub async fn list_transactions(
        db: &DatabaseConnection,
        user_id: &str,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<TransactionWithDetail>, DbErr> {
        services::transactions::list_transactions(db, user_id, limit, offset).await
    }

    pub async fn list_contacts(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::contacts::Model>, DbErr> {
        services::contacts::list_contacts(db, user_id).await
    }

    pub async fn create_contact(
        db: &DatabaseConnection,
        user_id: &str,
        name: String,
        phone: Option<String>,
    ) -> Result<entities::contacts::Model, DbErr> {
        services::contacts::create_contact(db, user_id, name, phone).await
    }

    pub async fn update_contact(
        db: &DatabaseConnection,
        user_id: &str,
        contact_id: &str,
        name: Option<String>,
        phone: Option<String>,
        is_pinned: Option<bool>,
    ) -> Result<entities::contacts::Model, DbErr> {
        services::contacts::update_contact(db, user_id, contact_id, name, phone, is_pinned).await
    }

    pub async fn delete_contact(
        db: &DatabaseConnection,
        user_id: &str,
        contact_id: &str,
    ) -> Result<(), DbErr> {
        services::contacts::delete_contact(db, user_id, contact_id).await
    }

    pub async fn get_contact_detail(
        db: &DatabaseConnection,
        user_id: &str,
        contact_id: &str,
    ) -> Result<
        (
            entities::contacts::Model,
            Vec<entities::contact_identifiers::Model>,
            Vec<entities::transactions::Model>,
        ),
        DbErr,
    > {
        services::contacts::get_contact_detail(db, user_id, contact_id).await
    }

    pub async fn add_contact_identifier(
        db: &DatabaseConnection,
        user_id: &str,
        contact_id: &str,
        r#type: String,
        value: String,
    ) -> Result<entities::contact_identifiers::Model, DbErr> {
        services::contacts::add_contact_identifier(db, user_id, contact_id, r#type, value).await
    }

    pub async fn list_pending_p2p_requests(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Vec<P2PRequestWithSender>, DbErr> {
        services::p2p::list_pending_p2p_requests(db, email).await
    }

    pub async fn create_group(
        db: &DatabaseConnection,
        user_id: &str,
        name: &str,
        description: Option<String>,
    ) -> Result<entities::groups::Model, DbErr> {
        services::groups::create_group(db, user_id, name, description).await
    }

    pub async fn invite_to_group(
        db: &DatabaseConnection,
        sender_id: &str,
        receiver_email: &str,
        group_id: &str,
    ) -> Result<entities::p2p_requests::Model, DbErr> {
        services::groups::invite_to_group(db, sender_id, receiver_email, group_id).await
    }

    pub async fn list_groups(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::groups::Model>, DbErr> {
        services::groups::list_groups(db, user_id).await
    }

    pub async fn list_group_transactions(
        db: &DatabaseConnection,
        group_id: &str,
    ) -> Result<Vec<entities::transactions::Model>, DbErr> {
        services::groups::list_group_transactions(db, group_id).await
    }

    pub async fn remove_group_member(
        db: &DatabaseConnection,
        admin_id: &str,
        group_id: &str,
        target_user_id: &str,
    ) -> Result<(), DbErr> {
        services::groups::remove_group_member(db, admin_id, group_id, target_user_id).await
    }

    pub async fn update_member_role(
        db: &DatabaseConnection,
        admin_id: &str,
        group_id: &str,
        target_user_id: &str,
        new_role: GroupRole,
    ) -> Result<(), DbErr> {
        services::groups::update_member_role(db, admin_id, group_id, target_user_id, new_role).await
    }

    pub async fn update_transaction(
        db: &DatabaseConnection,
        user_id: &str,
        txn_id: &str,
        amount: Option<Decimal>,
        date: Option<DateTime<FixedOffset>>,
        purpose_tag: Option<String>,
        category_id: Option<String>,
        status: Option<TransactionStatus>,
        notes: Option<String>,
        source_wallet_id: Option<String>,
        destination_wallet_id: Option<String>,
        contact_id: Option<String>,
    ) -> Result<entities::transactions::Model, DbErr> {
        services::transactions::update_transaction(
            db,
            user_id,
            txn_id,
            amount,
            date,
            purpose_tag,
            category_id,
            status,
            notes,
            source_wallet_id,
            destination_wallet_id,
            contact_id,
        )
        .await
    }

    pub async fn delete_transaction(
        db: &DatabaseConnection,
        user_id: &str,
        txn_id: &str,
    ) -> Result<u64, DbErr> {
        services::transactions::delete_transaction(db, user_id, txn_id).await
    }

    pub async fn revert_transaction(
        db: &DatabaseConnection,
        user_id: &str,
        txn_id: &str,
    ) -> Result<entities::transactions::Model, DbErr> {
        services::transactions::revert_transaction(db, user_id, txn_id).await
    }

    pub async fn register_repayment(
        db: &DatabaseConnection,
        user_id: &str,
        tab_id: &str,
        amount: Decimal,
        source_wallet_id: Option<String>,
    ) -> Result<entities::transactions::Model, DbErr> {
        services::p2p::register_repayment(db, user_id, tab_id, amount, source_wallet_id).await
    }

    pub async fn create_transaction(
        db: &DatabaseConnection,
        user_id: &str,
        amount: Decimal,
        direction: TransactionDirection,
        date: DateTime<FixedOffset>,
        source: TransactionSource,
        purpose_tag: Option<String>,
        category_id: Option<String>,
        source_wallet_id: Option<String>,
        destination_wallet_id: Option<String>,
        contact_id: Option<String>,
        notes: Option<String>,
    ) -> Result<entities::transactions::Model, DbErr> {
        services::transactions::create_transaction(
            db,
            user_id,
            amount,
            direction,
            date,
            source,
            purpose_tag,
            category_id,
            source_wallet_id,
            destination_wallet_id,
            contact_id,
            notes,
        )
        .await
    }

    pub async fn list_wallets(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::wallets::Model>, DbErr> {
        services::wallets::list_wallets(db, user_id).await
    }

    pub async fn create_wallet(
        db: &DatabaseConnection,
        user_id: &str,
        name: &str,
        wallet_type: WalletType,
        initial_balance: Decimal,
    ) -> Result<entities::wallets::Model, DbErr> {
        services::wallets::create_wallet(db, user_id, name, wallet_type, initial_balance).await
    }

    pub async fn update_wallet(
        db: &DatabaseConnection,
        user_id: &str,
        wallet_id: &str,
        name: Option<String>,
        balance: Option<Decimal>,
    ) -> Result<entities::wallets::Model, DbErr> {
        services::wallets::update_wallet(db, user_id, wallet_id, name, balance).await
    }

    pub async fn delete_wallet(
        db: &DatabaseConnection,
        user_id: &str,
        wallet_id: &str,
    ) -> Result<u64, DbErr> {
        services::wallets::delete_wallet(db, user_id, wallet_id).await
    }

    pub async fn create_ledger_tab(
        db: &DatabaseConnection,
        creator_id: &str,
        counterparty_id: Option<String>,
        tab_type: LedgerTabType,
        title: &str,
        description: Option<String>,
        target_amount: Decimal,
    ) -> Result<entities::ledger_tabs::Model, DbErr> {
        services::p2p::create_ledger_tab(
            db,
            creator_id,
            counterparty_id,
            tab_type,
            title,
            description,
            target_amount,
        )
        .await
    }

    pub async fn update_profile(
        db: &DatabaseConnection,
        user_id: &str,
        name: Option<String>,
        username: Option<String>,
        image: Option<String>,
    ) -> Result<entities::users::Model, DbErr> {
        services::users::update_profile(db, user_id, name, username, image).await
    }

    pub async fn list_categories(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::categories::Model>, DbErr> {
        services::categories::list_categories(db, user_id).await
    }

    pub async fn ensure_system_categories(db: &DatabaseConnection) -> Result<(), DbErr> {
        services::categories::ensure_system_categories(db).await
    }

    pub async fn create_category(
        db: &DatabaseConnection,
        user_id: &str,
        name: String,
        icon: Option<String>,
        color: Option<String>,
    ) -> Result<entities::categories::Model, DbErr> {
        services::categories::create_category(db, user_id, name, icon, color).await
    }

    pub async fn delete_category(
        db: &DatabaseConnection,
        user_id: &str,
        category_id: &str,
    ) -> Result<(), DbErr> {
        services::categories::delete_category(db, user_id, category_id).await
    }

    pub async fn list_user_upi(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::user_upi_ids::Model>, DbErr> {
        services::users::list_user_upi(db, user_id).await
    }

    pub async fn add_user_upi(
        db: &DatabaseConnection,
        user_id: &str,
        upi_id: String,
        label: Option<String>,
    ) -> Result<entities::user_upi_ids::Model, DbErr> {
        services::users::add_user_upi(db, user_id, upi_id, label).await
    }

    pub async fn make_primary_upi(
        db: &DatabaseConnection,
        user_id: &str,
        upi_id: &str,
    ) -> Result<(), DbErr> {
        services::users::make_primary_upi(db, user_id, upi_id).await
    }

    pub async fn reject_p2p_request(
        db: &DatabaseConnection,
        user_id: &str,
        request_id: &str,
    ) -> Result<(), DbErr> {
        services::p2p::reject_p2p_request(db, user_id, request_id).await
    }

    pub async fn list_confirmed_subscriptions(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::subscriptions::Model>, DbErr> {
        services::subscriptions::list_confirmed_subscriptions(db, user_id).await
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
        services::subscriptions::confirm_subscription(
            db,
            user_id,
            name,
            amount,
            cycle,
            start_date,
            next_charge_date,
            keywords,
        )
        .await
    }

    pub async fn stop_tracking_subscription(
        db: &DatabaseConnection,
        user_id: &str,
        sub_id: &str,
    ) -> Result<(), DbErr> {
        services::subscriptions::stop_tracking_subscription(db, user_id, sub_id).await
    }

    pub async fn configure_subscription_alert(
        db: &DatabaseConnection,
        sub_id: &str,
        days_before: i32,
        channel: String,
    ) -> Result<entities::sub_alerts::Model, DbErr> {
        services::subscriptions::configure_subscription_alert(db, sub_id, days_before, channel)
            .await
    }

    pub async fn upload_statement(
        db: &DatabaseConnection,
        user_id: &str,
        date: DateTime<FixedOffset>,
        description: String,
        amount: Decimal,
        raw_data: Option<serde_json::Value>,
    ) -> Result<entities::bank_statement_rows::Model, DbErr> {
        services::reconciliation::upload_statement(db, user_id, date, description, amount, raw_data)
            .await
    }

    pub async fn list_unmatched_rows(
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<Vec<entities::bank_statement_rows::Model>, DbErr> {
        services::reconciliation::list_unmatched_rows(db, user_id).await
    }

    pub async fn get_row_matches(
        db: &DatabaseConnection,
        user_id: &str,
        row_id: &str,
    ) -> Result<Vec<(entities::transactions::Model, i32)>, DbErr> {
        services::reconciliation::get_row_matches(db, user_id, row_id).await
    }

    pub async fn confirm_match(
        db: &DatabaseConnection,
        user_id: &str,
        row_id: &str,
        txn_id: &str,
        confidence: i32,
    ) -> Result<(), DbErr> {
        services::reconciliation::confirm_match(db, user_id, row_id, txn_id, confidence).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Instant;

    #[tokio::test]
    async fn export_types() {
        // This test is just to trigger ts-rs export
    }

    #[tokio::test]
    async fn benchmark_split_transaction() -> Result<(), DbErr> {
        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");
        let db = Database::connect(&db_url).await?;

        // Setup schema
        let db_backend = db.get_database_backend();
        let schema = Schema::new(db_backend);

        db.execute(db_backend.build(&schema.create_table_from_entity(entities::users::Entity)))
            .await?;
        db.execute(
            db_backend.build(&schema.create_table_from_entity(entities::categories::Entity)),
        )
        .await?;
        db.execute(
            db_backend.build(&schema.create_table_from_entity(entities::transactions::Entity)),
        )
        .await?;
        db.execute(
            db_backend.build(&schema.create_table_from_entity(entities::p2p_requests::Entity)),
        )
        .await?;

        // Create a user
        let user = entities::users::ActiveModel {
            id: Set("user_1".to_string()),
            email: Set("user_1@example.com".to_string()),
            name: Set("User One".to_string()),
            email_verified: Set(true),
            is_active: Set(true),
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
            ..Default::default()
        };
        user.insert(&db).await?;

        // Create a transaction
        let txn_id = uuid::Uuid::now_v7().to_string();
        let txn = entities::transactions::ActiveModel {
            id: Set(txn_id.clone()),
            user_id: Set("user_1".to_string()),
            amount: Set(Decimal::new(100, 0)),
            direction: Set(TransactionDirection::Out),
            date: Set(Utc::now().into()),
            source: Set(TransactionSource::Manual),
            status: Set(TransactionStatus::Completed),
            category_id: Set(None),
            purpose_tag: Set(Some("Lunch".to_string())),
            group_id: Set(None),
            source_wallet_id: Set(None),
            destination_wallet_id: Set(None),
            ledger_tab_id: Set(None),
            deleted_at: Set(None),
            notes: Set(None),
        };
        txn.insert(&db).await?;

        let num_splits = 100;
        let splits: Vec<SplitDetail> = (0..num_splits)
            .map(|i| SplitDetail {
                receiver_email: format!("user_{}@example.com", i),
                amount: Decimal::new(1, 0),
            })
            .collect();

        let start = Instant::now();
        SmartMerge::split_transaction(&db, "user_1", &txn_id, splits).await?;
        let duration = start.elapsed();

        println!(
            "\nBENCHMARK_RESULT: {} splits took {:?}",
            num_splits, duration
        );

        Ok(())
    }
}
