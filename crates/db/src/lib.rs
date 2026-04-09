use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
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
    pub doc_type: String,        // "GPAY" or "GENERIC"
    pub data: ExportedJsonValue, // Use ExportedJsonValue instead of serde_json::Value
    pub r2_key: Option<String>,
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
}
