use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "ocr_jobs")]
#[ts(
    export,
    rename = "OcrJob",
    export_to = "../../../packages/types/src/db/OcrJob.ts"
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: String,
    pub status: String, // "QUEUED", "PROCESSING", "COMPLETED", "FAILED", "PENDING_REVIEW", "CONTACT_COLLISION", "DEAD_LETTER"
    pub r2_key: String,
    pub raw_key: Option<String>,
    pub p_hash: Option<String>,
    pub auto_confirm: bool,
    pub wallet_id: Option<String>,
    pub category_id: Option<String>,
    pub transaction_id: Option<String>,
    pub started_at: Option<DateTimeWithTimeZone>,
    pub scheduled_at: Option<DateTimeWithTimeZone>,
    pub retry_count: i32,
    pub is_high_res: bool,
    pub schema_version: i32,
    pub last_error: Option<String>,
    #[ts(type = "import('./JsonValue').JsonValue")]
    pub resolution_candidates: Option<Json>,
    #[ts(type = "import('./JsonValue').JsonValue")]
    pub processed_data: Option<Json>,
    pub error: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
