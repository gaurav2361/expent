use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "p2p_requests")]
#[ts(export, export_to = "../../../packages/types/src/db/generated/p2p_request.ts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub sender_user_id: String,
    pub receiver_email: String,
    #[ts(type = "any")]
    pub transaction_data: Json, // JSONB snapshot
    pub status: String, // PENDING, MAPPED, REJECTED, APPROVED
    pub linked_txn_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::SenderUserId",
        to = "super::user::Column::Id"
    )]
    Sender,
    #[sea_orm(
        belongs_to = "super::transaction::Entity",
        from = "Column::LinkedTxnId",
        to = "super::transaction::Column::Id"
    )]
    LinkedTransaction,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sender.def()
    }
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LinkedTransaction.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
