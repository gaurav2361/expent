use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "subscription_charges")]
#[ts(export, export_to = "../../../packages/types/src/db/generated/subscription_charge.ts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub subscription_id: String,
    pub transaction_id: Option<String>,
    pub charged_on: DateTimeWithTimeZone,
    #[ts(type = "string")]
    pub amount: Decimal,
    pub status: String, // CHARGED, FAILED, PENDING
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subscription::Entity",
        from = "Column::SubscriptionId",
        to = "super::subscription::Column::Id"
    )]
    Subscription,
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::TransactionId",
        to = "super::transaction::Column::Id"
    )]
    Transaction,
}

impl Related<super::subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscription.def()
    }
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
