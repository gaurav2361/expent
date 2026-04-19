use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "ocr_job_edits")]
#[ts(
    export,
    rename = "OcrJobEdit",
    export_to = "../../../packages/types/src/db/OcrJobEdit.ts"
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub ocr_job_id: String,
    pub user_id: String,
    pub field_name: String,
    pub original_value: Option<String>,
    pub corrected_value: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ocr_jobs::Entity",
        from = "Column::OcrJobId",
        to = "super::ocr_jobs::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    OcrJobs,
}

impl Related<super::ocr_jobs::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OcrJobs.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
