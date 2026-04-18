use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub mod entities;
pub mod error;

pub use error::AppError;

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
    pub category_id: Option<String>,
    pub wallet_id: Option<String>,
    pub contact_id: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence_score: f32,
    #[serde(default)]
    pub items: Vec<LineItem>,
}

fn default_confidence() -> f32 {
    1.0
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
    pub category_id: Option<String>,
    pub wallet_id: Option<String>,
    pub contact_id: Option<String>,
    #[serde(default = "default_confidence")]
    pub confidence_score: f32,
}

/// Unified OCR data from the Python worker.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "ProcessedOcr",
    export_to = "../../../packages/types/src/db/ProcessedOcr.ts"
)]
pub struct ProcessedOcr {
    pub doc_type: String,        // "GPAY" or "GENERIC"
    pub data: ExportedJsonValue, // Use ExportedJsonValue instead of serde_json::Value
    pub r2_key: Option<String>,
    #[serde(default)]
    pub is_high_res: bool,
}

/// A type alias for serde_json::Value to control its TypeScript export location.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    rename = "JsonValue",
    export_to = "../../../packages/types/src/db/JsonValue.ts"
)]
pub struct ExportedJsonValue(
    #[ts(
        type = "number | string | boolean | Array<JsonValue> | { [key: string]: JsonValue } | null"
    )]
    pub serde_json::Value,
);

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
    pub category_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS, FromQueryResult)]
#[ts(
    export,
    rename = "GroupMemberDetail",
    export_to = "../../../packages/types/src/db/GroupMemberDetail.ts"
)]
pub struct GroupMemberDetail {
    pub user_id: String,
    pub name: String,
    pub email: String,
    pub role: String,
}

/// Paginated response for transactions.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "PaginatedTransactions",
    export_to = "../../../packages/types/src/db/PaginatedTransactions.ts"
)]
pub struct PaginatedTransactions {
    pub items: Vec<TransactionWithDetail>,
    pub total_count: u64,
}

/// Trend data for a single month.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "MonthlyTrend",
    export_to = "../../../packages/types/src/db/MonthlyTrend.ts"
)]
pub struct MonthlyTrend {
    pub month: String,
    #[ts(type = "string")]
    pub income: Decimal,
    #[ts(type = "string")]
    pub expense: Decimal,
}

/// Distribution of expenses by category or contact.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "NamedAmount",
    export_to = "../../../packages/types/src/db/NamedAmount.ts"
)]
pub struct NamedAmount {
    pub name: String,
    #[ts(type = "string")]
    pub amount: Decimal,
}

/// Summary data for the dashboard.
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    rename = "DashboardSummary",
    export_to = "../../../packages/types/src/db/DashboardSummary.ts"
)]
pub struct DashboardSummary {
    #[ts(type = "string")]
    pub total_balance: Decimal,
    #[ts(type = "string")]
    pub monthly_spend: Decimal,
    #[ts(type = "string")]
    pub monthly_income: Decimal,
    pub pending_p2p_count: u64,
    pub total_transactions: u64,
    pub monthly_trends: Vec<MonthlyTrend>,
    pub weekly_trends: Vec<MonthlyTrend>, // Reuse MonthlyTrend for weekly too
    pub category_distribution: Vec<NamedAmount>,
    pub top_expenses: Vec<NamedAmount>,
    pub top_income: Vec<NamedAmount>,
}
