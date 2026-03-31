use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "bank_statement_rows")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/bank_statement_row.ts"
)]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub date: DateTimeWithTimeZone,
    pub description: String,
    #[ts(type = "string")]
    pub debit: Option<Decimal>,
    #[ts(type = "string")]
    pub credit: Option<Decimal>,
    #[ts(type = "string")]
    pub balance: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::statement_txn_match::Entity")]
    Matches,
}

impl Related<super::statement_txn_match::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Matches.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
