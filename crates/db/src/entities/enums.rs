use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use ts_rs::TS;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/enums.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionDirection {
    #[sea_orm(string_value = "IN")]
    In,
    #[sea_orm(string_value = "OUT")]
    Out,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/enums.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionSource {
    #[sea_orm(string_value = "MANUAL")]
    Manual,
    #[sea_orm(string_value = "OCR")]
    Ocr,
    #[sea_orm(string_value = "STATEMENT")]
    Statement,
    #[sea_orm(string_value = "P2P")]
    P2p,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/enums.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    #[sea_orm(string_value = "COMPLETED")]
    Completed,
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "CANCELLED")]
    Cancelled,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/enums.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IdentifierType {
    #[sea_orm(string_value = "UPI")]
    Upi,
    #[sea_orm(string_value = "PHONE")]
    Phone,
    #[sea_orm(string_value = "BANK_ACC")]
    BankAcc,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/enums.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TxnPartyRole {
    #[sea_orm(string_value = "SENDER")]
    Sender,
    #[sea_orm(string_value = "RECEIVER")]
    Receiver,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/enums.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubscriptionCycle {
    #[sea_orm(string_value = "WEEKLY")]
    Weekly,
    #[sea_orm(string_value = "MONTHLY")]
    Monthly,
    #[sea_orm(string_value = "YEARLY")]
    Yearly,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/enums.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AlertChannel {
    #[sea_orm(string_value = "EMAIL")]
    Email,
    #[sea_orm(string_value = "PUSH")]
    Push,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/enums.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum P2PRequestStatus {
    #[sea_orm(string_value = "PENDING")]
    Pending,
    #[sea_orm(string_value = "MAPPED")]
    Mapped,
    #[sea_orm(string_value = "REJECTED")]
    Rejected,
    #[sea_orm(string_value = "APPROVED")]
    Approved,
    #[sea_orm(string_value = "GROUP_INVITE")]
    GroupInvite,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    Serialize,
    Deserialize,
    TS,
    Display,
    EnumString,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::N(20))")]
#[ts(
    export,
    export_to = "../../../packages/types/src/db/generated/enums.ts"
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GroupRole {
    #[sea_orm(string_value = "ADMIN")]
    Admin,
    #[sea_orm(string_value = "MEMBER")]
    Member,
}
