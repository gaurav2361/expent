use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "transaction_metadata")]
#[ts(export, export_to = "../../../packages/types/src/db/generated/transaction_metadata.ts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub transaction_id: String,
    pub upi_txn_id: Option<String>,
    pub app_txn_id: Option<String>,
    pub app_name: Option<String>,
    pub contact_number: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TransactionId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
