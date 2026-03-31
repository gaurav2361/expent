use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Users
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).string().primary_key())
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::EmailVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Users::Image).string())
                    .col(ColumnDef::new(Users::Phone).string())
                    .col(
                        ColumnDef::new(Users::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Sessions
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Sessions::Id).string().primary_key())
                    .col(
                        ColumnDef::new(Sessions::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Sessions::Token)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Sessions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Sessions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Sessions::IpAddress).string())
                    .col(ColumnDef::new(Sessions::UserAgent).string())
                    .col(ColumnDef::new(Sessions::UserId).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-sessions-user_id")
                            .from(Sessions::Table, Sessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Accounts
        manager
            .create_table(
                Table::create()
                    .table(Accounts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Accounts::Id).string().primary_key())
                    .col(ColumnDef::new(Accounts::AccountId).string().not_null())
                    .col(ColumnDef::new(Accounts::ProviderId).string().not_null())
                    .col(ColumnDef::new(Accounts::UserId).string().not_null())
                    .col(ColumnDef::new(Accounts::AccessToken).string())
                    .col(ColumnDef::new(Accounts::RefreshToken).string())
                    .col(ColumnDef::new(Accounts::IdToken).string())
                    .col(ColumnDef::new(Accounts::AccessTokenExpiresAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Accounts::RefreshTokenExpiresAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Accounts::Scope).string())
                    .col(ColumnDef::new(Accounts::Password).string())
                    .col(
                        ColumnDef::new(Accounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Accounts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-accounts-user_id")
                            .from(Accounts::Table, Accounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Verifications
        manager
            .create_table(
                Table::create()
                    .table(Verifications::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Verifications::Id).string().primary_key())
                    .col(
                        ColumnDef::new(Verifications::Identifier)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Verifications::Value).string().not_null())
                    .col(
                        ColumnDef::new(Verifications::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Verifications::CreatedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Verifications::UpdatedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        // User UPI IDs
        manager
            .create_table(
                Table::create()
                    .table(UserUpiIds::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserUpiIds::Id).string().primary_key())
                    .col(ColumnDef::new(UserUpiIds::UserId).string().not_null())
                    .col(
                        ColumnDef::new(UserUpiIds::UpiId)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserUpiIds::IsPrimary)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(UserUpiIds::Label).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_upi_ids-user_id")
                            .from(UserUpiIds::Table, UserUpiIds::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Contacts
        manager
            .create_table(
                Table::create()
                    .table(Contacts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Contacts::Id).string().primary_key())
                    .col(ColumnDef::new(Contacts::Name).string().not_null())
                    .col(ColumnDef::new(Contacts::Phone).string())
                    .col(
                        ColumnDef::new(Contacts::IsPinned)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // Contact Identifiers
        manager
            .create_table(
                Table::create()
                    .table(ContactIdentifiers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ContactIdentifiers::Id)
                            .string()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ContactIdentifiers::ContactId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ContactIdentifiers::Type).string().not_null())
                    .col(
                        ColumnDef::new(ContactIdentifiers::Value)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ContactIdentifiers::LinkedUserId).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-contact_identifiers-contact_id")
                            .from(ContactIdentifiers::Table, ContactIdentifiers::ContactId)
                            .to(Contacts::Table, Contacts::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Contact Links
        manager
            .create_table(
                Table::create()
                    .table(ContactLinks::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ContactLinks::UserId).string().not_null())
                    .col(ColumnDef::new(ContactLinks::ContactId).string().not_null())
                    .primary_key(
                        Index::create()
                            .col(ContactLinks::UserId)
                            .col(ContactLinks::ContactId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-contact_links-user_id")
                            .from(ContactLinks::Table, ContactLinks::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-contact_links-contact_id")
                            .from(ContactLinks::Table, ContactLinks::ContactId)
                            .to(Contacts::Table, Contacts::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Transactions
        manager
            .create_table(
                Table::create()
                    .table(Transactions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Transactions::Id).string().primary_key())
                    .col(ColumnDef::new(Transactions::UserId).string().not_null())
                    .col(ColumnDef::new(Transactions::Amount).decimal().not_null())
                    .col(ColumnDef::new(Transactions::Direction).string().not_null())
                    .col(
                        ColumnDef::new(Transactions::Date)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Transactions::Source).string().not_null())
                    .col(ColumnDef::new(Transactions::Status).string().not_null())
                    .col(ColumnDef::new(Transactions::PurposeTag).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transactions-user_id")
                            .from(Transactions::Table, Transactions::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Transaction Metadata
        manager
            .create_table(
                Table::create()
                    .table(TransactionMetadata::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransactionMetadata::TransactionId)
                            .string()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TransactionMetadata::UpiTxnId).string())
                    .col(ColumnDef::new(TransactionMetadata::AppTxnId).string())
                    .col(ColumnDef::new(TransactionMetadata::AppName).string())
                    .col(ColumnDef::new(TransactionMetadata::ContactNumber).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_metadata-transaction_id")
                            .from(
                                TransactionMetadata::Table,
                                TransactionMetadata::TransactionId,
                            )
                            .to(Transactions::Table, Transactions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Transaction Sources
        manager
            .create_table(
                Table::create()
                    .table(TransactionSources::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransactionSources::Id)
                            .string()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(TransactionSources::TransactionId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransactionSources::SourceType)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransactionSources::R2FileUrl).string())
                    .col(ColumnDef::new(TransactionSources::RawMetadata).json())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_sources-transaction_id")
                            .from(TransactionSources::Table, TransactionSources::TransactionId)
                            .to(Transactions::Table, Transactions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Txn Parties
        manager
            .create_table(
                Table::create()
                    .table(TxnParties::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TxnParties::Id).string().primary_key())
                    .col(
                        ColumnDef::new(TxnParties::TransactionId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TxnParties::UserId).string())
                    .col(ColumnDef::new(TxnParties::ContactId).string())
                    .col(ColumnDef::new(TxnParties::Role).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-txn_parties-transaction_id")
                            .from(TxnParties::Table, TxnParties::TransactionId)
                            .to(Transactions::Table, Transactions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // P2P Requests
        manager
            .create_table(
                Table::create()
                    .table(P2PRequests::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(P2PRequests::Id).string().primary_key())
                    .col(
                        ColumnDef::new(P2PRequests::SenderUserId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(P2PRequests::ReceiverEmail)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(P2PRequests::TransactionData)
                            .json()
                            .not_null(),
                    )
                    .col(ColumnDef::new(P2PRequests::Status).string().not_null())
                    .col(ColumnDef::new(P2PRequests::LinkedTxnId).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-p2p_requests-sender_user_id")
                            .from(P2PRequests::Table, P2PRequests::SenderUserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Bank Statement Rows
        manager
            .create_table(
                Table::create()
                    .table(BankStatementRows::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(BankStatementRows::Id).string().primary_key())
                    .col(
                        ColumnDef::new(BankStatementRows::Date)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BankStatementRows::Description)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(BankStatementRows::Debit).decimal())
                    .col(ColumnDef::new(BankStatementRows::Credit).decimal())
                    .col(
                        ColumnDef::new(BankStatementRows::Balance)
                            .decimal()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Statement Txn Matches
        manager
            .create_table(
                Table::create()
                    .table(StatementTxnMatches::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(StatementTxnMatches::RowId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StatementTxnMatches::TransactionId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(StatementTxnMatches::Confidence)
                            .decimal()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(StatementTxnMatches::RowId)
                            .col(StatementTxnMatches::TransactionId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-statement_txn_matches-row_id")
                            .from(StatementTxnMatches::Table, StatementTxnMatches::RowId)
                            .to(BankStatementRows::Table, BankStatementRows::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-statement_txn_matches-transaction_id")
                            .from(
                                StatementTxnMatches::Table,
                                StatementTxnMatches::TransactionId,
                            )
                            .to(Transactions::Table, Transactions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Subscriptions
        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Subscriptions::Id).string().primary_key())
                    .col(ColumnDef::new(Subscriptions::UserId).string().not_null())
                    .col(ColumnDef::new(Subscriptions::Name).string().not_null())
                    .col(ColumnDef::new(Subscriptions::Amount).decimal().not_null())
                    .col(ColumnDef::new(Subscriptions::Cycle).string().not_null())
                    .col(
                        ColumnDef::new(Subscriptions::StartDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::NextChargeDate)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Subscriptions::DetectionKeywords).json())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-subscriptions-user_id")
                            .from(Subscriptions::Table, Subscriptions::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Subscription Charges
        manager
            .create_table(
                Table::create()
                    .table(SubscriptionCharges::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SubscriptionCharges::Id)
                            .string()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionCharges::SubscriptionId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SubscriptionCharges::TransactionId).string())
                    .col(
                        ColumnDef::new(SubscriptionCharges::ChargedOn)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionCharges::Amount)
                            .decimal()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SubscriptionCharges::Status)
                            .string()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-subscription_charges-subscription_id")
                            .from(
                                SubscriptionCharges::Table,
                                SubscriptionCharges::SubscriptionId,
                            )
                            .to(Subscriptions::Table, Subscriptions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Sub Alerts
        manager
            .create_table(
                Table::create()
                    .table(SubAlerts::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SubAlerts::Id).string().primary_key())
                    .col(
                        ColumnDef::new(SubAlerts::SubscriptionId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SubAlerts::DaysBefore).integer().not_null())
                    .col(ColumnDef::new(SubAlerts::SentAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(SubAlerts::Channel).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-sub_alerts-subscription_id")
                            .from(SubAlerts::Table, SubAlerts::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Purchases
        manager
            .create_table(
                Table::create()
                    .table(Purchases::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Purchases::Id).string().primary_key())
                    .col(ColumnDef::new(Purchases::TransactionId).string().not_null())
                    .col(ColumnDef::new(Purchases::Vendor).string().not_null())
                    .col(ColumnDef::new(Purchases::Total).decimal().not_null())
                    .col(ColumnDef::new(Purchases::OrderId).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-purchases-transaction_id")
                            .from(Purchases::Table, Purchases::TransactionId)
                            .to(Transactions::Table, Transactions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Purchase Items
        manager
            .create_table(
                Table::create()
                    .table(PurchaseItems::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PurchaseItems::Id).string().primary_key())
                    .col(
                        ColumnDef::new(PurchaseItems::PurchaseId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PurchaseItems::Name).string().not_null())
                    .col(ColumnDef::new(PurchaseItems::Quantity).integer().not_null())
                    .col(ColumnDef::new(PurchaseItems::Price).decimal().not_null())
                    .col(ColumnDef::new(PurchaseItems::Sku).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-purchase_items-purchase_id")
                            .from(PurchaseItems::Table, PurchaseItems::PurchaseId)
                            .to(Purchases::Table, Purchases::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Purchase Imports
        manager
            .create_table(
                Table::create()
                    .table(PurchaseImports::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PurchaseImports::Id).string().primary_key())
                    .col(ColumnDef::new(PurchaseImports::PdfUrl).string().not_null())
                    .col(ColumnDef::new(PurchaseImports::Vendor).string().not_null())
                    .col(ColumnDef::new(PurchaseImports::RawContent).text())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PurchaseImports::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PurchaseItems::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Purchases::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SubAlerts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SubscriptionCharges::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(StatementTxnMatches::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BankStatementRows::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(P2PRequests::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TxnParties::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TransactionSources::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(TransactionMetadata::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Transactions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ContactLinks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ContactIdentifiers::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Contacts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(UserUpiIds::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Verifications::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Accounts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Name,
    Email,
    EmailVerified,
    Image,
    Phone,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    Id,
    ExpiresAt,
    Token,
    CreatedAt,
    UpdatedAt,
    IpAddress,
    UserAgent,
    UserId,
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
    AccountId,
    ProviderId,
    UserId,
    AccessToken,
    RefreshToken,
    IdToken,
    AccessTokenExpiresAt,
    RefreshTokenExpiresAt,
    Scope,
    Password,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Verifications {
    Table,
    Id,
    Identifier,
    Value,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum UserUpiIds {
    Table,
    Id,
    UserId,
    UpiId,
    IsPrimary,
    Label,
}

#[derive(DeriveIden)]
enum Contacts {
    Table,
    Id,
    Name,
    Phone,
    IsPinned,
}

#[derive(DeriveIden)]
enum ContactIdentifiers {
    Table,
    Id,
    ContactId,
    Type,
    Value,
    LinkedUserId,
}

#[derive(DeriveIden)]
enum ContactLinks {
    Table,
    UserId,
    ContactId,
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    Id,
    UserId,
    Amount,
    Direction,
    Date,
    Source,
    Status,
    PurposeTag,
}

#[derive(DeriveIden)]
enum TransactionMetadata {
    Table,
    TransactionId,
    UpiTxnId,
    AppTxnId,
    AppName,
    ContactNumber,
}

#[derive(DeriveIden)]
enum TransactionSources {
    Table,
    Id,
    TransactionId,
    SourceType,
    R2FileUrl,
    RawMetadata,
}

#[derive(DeriveIden)]
enum TxnParties {
    Table,
    Id,
    TransactionId,
    UserId,
    ContactId,
    Role,
}

#[derive(DeriveIden)]
enum P2PRequests {
    Table,
    Id,
    SenderUserId,
    ReceiverEmail,
    TransactionData,
    Status,
    LinkedTxnId,
}

#[derive(DeriveIden)]
enum BankStatementRows {
    Table,
    Id,
    Date,
    Description,
    Debit,
    Credit,
    Balance,
}

#[derive(DeriveIden)]
enum StatementTxnMatches {
    Table,
    RowId,
    TransactionId,
    Confidence,
}

#[derive(DeriveIden)]
enum Subscriptions {
    Table,
    Id,
    UserId,
    Name,
    Amount,
    Cycle,
    StartDate,
    NextChargeDate,
    DetectionKeywords,
}

#[derive(DeriveIden)]
enum SubscriptionCharges {
    Table,
    Id,
    SubscriptionId,
    TransactionId,
    ChargedOn,
    Amount,
    Status,
}

#[derive(DeriveIden)]
enum SubAlerts {
    Table,
    Id,
    SubscriptionId,
    DaysBefore,
    SentAt,
    Channel,
}

#[derive(DeriveIden)]
enum Purchases {
    Table,
    Id,
    TransactionId,
    Vendor,
    Total,
    OrderId,
}

#[derive(DeriveIden)]
enum PurchaseItems {
    Table,
    Id,
    PurchaseId,
    Name,
    Quantity,
    Price,
    Sku,
}

#[derive(DeriveIden)]
enum PurchaseImports {
    Table,
    Id,
    PdfUrl,
    Vendor,
    RawContent,
}
