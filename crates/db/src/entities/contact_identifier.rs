use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "contact_identifiers")]
#[ts(export, export_to = "../../../packages/types/src/db/generated/contact_identifier.ts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub contact_id: String,
    pub r#type: String, // UPI, PHONE, BANK_ACC
    pub value: String,
    pub linked_user_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::contact::Entity",
        from = "Column::ContactId",
        to = "super::contact::Column::Id"
    )]
    Contact,
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Contact.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
