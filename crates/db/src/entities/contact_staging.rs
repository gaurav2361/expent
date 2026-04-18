use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "contact_staging")]
#[ts(
    export,
    rename = "ContactStaging",
    export_to = "../../../packages/types/src/db/ContactStaging.ts"
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: String,
    pub ocr_job_id: String,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub upi_id: Option<String>,
    pub status: String, // "PENDING", "APPROVED", "REJECTED"
    #[ts(type = "import('./JsonValue').JsonValue")]
    pub candidates: Option<Json>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
