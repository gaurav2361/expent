use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "transactions")]
#[ts(export, export_to = "../../../packages/types/src/db/generated/transaction.ts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub user_id: String,
    #[ts(type = "string")]
    pub amount: Decimal,
    pub direction: String, // IN, OUT
    pub date: DateTimeWithTimeZone,
    pub source: String, // MANUAL, OCR, STATEMENT
    pub status: String, // COMPLETED, PENDING, CANCELLED
    pub purpose_tag: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(has_one = "super::transaction_metadata::Entity")]
    Metadata,
    #[sea_orm(has_many = "super::transaction_source::Entity")]
    Sources,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::transaction_metadata::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Metadata.def()
    }
}

impl Related<super::transaction_source::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sources.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
