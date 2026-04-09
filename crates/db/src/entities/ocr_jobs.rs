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
    pub status: String, // "PENDING", "COMPLETED", "FAILED"
    pub r2_key: String,
    #[ts(type = "import('./JsonValue').JsonValue")]
    pub processed_data: Option<Json>,
    pub error: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
