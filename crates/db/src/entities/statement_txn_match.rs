use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "statement_txn_matches")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/statement_txn_match.ts"
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub row_id: String,
    #[sea_orm(primary_key, auto_increment = false)]
    pub transaction_id: String,
    #[ts(type = "string")]
    pub confidence: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::bank_statement_row::Entity",
        from = "Column::RowId",
        to = "super::bank_statement_row::Column::Id"
    )]
    BankStatementRow,
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TransactionId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
}

impl Related<super::bank_statement_row::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::BankStatementRow.def()
    }
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
