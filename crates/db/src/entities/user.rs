use super::enums::GroupRole;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, TS)]
#[sea_orm(table_name = "users")]
#[ts(export, export_to = "../../../packages/types/src/db/generated/user.ts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub email: String,
    pub email_verified: bool,
    pub image: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,

    // Enhanced fields for better-auth and general usage
    pub username: Option<String>,
    pub display_username: Option<String>,
    pub role: Option<GroupRole>,
    pub banned: Option<bool>,
    pub ban_reason: Option<String>,
    pub ban_expires: Option<DateTimeWithTimeZone>,
    pub two_factor_enabled: Option<bool>,
    pub phone_number: Option<String>,
    pub phone_number_verified: Option<bool>,
    pub associated_contact_id: Option<String>,
    #[ts(type = "any")]
    pub metadata: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::session::Entity")]
    Sessions,
    #[sea_orm(has_many = "super::account::Entity")]
    Accounts,
    #[sea_orm(has_many = "super::contact_link::Entity")]
    ContactLinks,
    #[sea_orm(has_many = "super::user_group::Entity")]
    UserGroups,
    #[sea_orm(has_many = "super::transaction::Entity")]
    Transactions,
    #[sea_orm(has_many = "super::subscription::Entity")]
    Subscriptions,
    #[sea_orm(has_many = "super::p2p_request::Entity")]
    P2PRequests,
    #[sea_orm(has_many = "super::user_upi_id::Entity")]
    UpiIds,
    #[sea_orm(
        belongs_to = "super::contact::Entity",
        from = "Column::AssociatedContactId",
        to = "super::contact::Column::Id"
    )]
    AssociatedContact,
}

impl Related<super::session::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sessions.def()
    }
}

impl Related<super::account::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Accounts.def()
    }
}

impl Related<super::contact_link::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ContactLinks.def()
    }
}

impl Related<super::user_group::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserGroups.def()
    }
}

impl Related<super::transaction::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Transactions.def()
    }
}

impl Related<super::subscription::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriptions.def()
    }
}

impl Related<super::p2p_request::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::P2PRequests.def()
    }
}

impl Related<super::user_upi_id::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UpiIds.def()
    }
}

impl Related<super::contact::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AssociatedContact.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
