use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Wallets
        manager
            .create_table(
                Table::create()
                    .table(Wallets::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Wallets::Id).string().primary_key())
                    .col(ColumnDef::new(Wallets::UserId).string().not_null())
                    .col(ColumnDef::new(Wallets::Name).string().not_null())
                    .col(ColumnDef::new(Wallets::Type).string().not_null())
                    .col(
                        ColumnDef::new(Wallets::Balance)
                            .decimal()
                            .not_null()
                            .default(0.0),
                    )
                    .col(
                        ColumnDef::new(Wallets::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Wallets::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-wallets-user_id")
                            .from(Wallets::Table, Wallets::UserId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Ledger Tabs
        manager
            .create_table(
                Table::create()
                    .table(LedgerTabs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(LedgerTabs::Id).string().primary_key())
                    .col(ColumnDef::new(LedgerTabs::CreatorId).string().not_null())
                    .col(ColumnDef::new(LedgerTabs::CounterpartyId).string())
                    .col(ColumnDef::new(LedgerTabs::TabType).string().not_null())
                    .col(ColumnDef::new(LedgerTabs::Title).string().not_null())
                    .col(
                        ColumnDef::new(LedgerTabs::TargetAmount)
                            .decimal()
                            .not_null(),
                    )
                    .col(ColumnDef::new(LedgerTabs::Status).string().not_null())
                    .col(
                        ColumnDef::new(LedgerTabs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(LedgerTabs::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-ledger_tabs-creator_id")
                            .from(LedgerTabs::Table, LedgerTabs::CreatorId)
                            .to(Users::Table, Users::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Update Transactions
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(ColumnDef::new(Transactions::SourceWalletId).string())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(ColumnDef::new(Transactions::DestinationWalletId).string())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(ColumnDef::new(Transactions::LedgerTabId).string())
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(ColumnDef::new(Transactions::DeletedAt).timestamp_with_time_zone())
                    .to_owned(),
            )
            .await?;

        // P2P Transfers
        manager
            .create_table(
                Table::create()
                    .table(P2PTransfers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(P2PTransfers::Id).string().primary_key())
                    .col(
                        ColumnDef::new(P2PTransfers::TransactionId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(P2PTransfers::Direction).string().not_null())
                    .col(
                        ColumnDef::new(P2PTransfers::CounterpartyName)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(P2PTransfers::CounterpartyPhone).string())
                    .col(ColumnDef::new(P2PTransfers::CounterpartyUpiId).string())
                    .col(
                        ColumnDef::new(P2PTransfers::IsMerchant)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(P2PTransfers::UpiTransactionId).string())
                    .col(ColumnDef::new(P2PTransfers::GoogleTransactionId).string())
                    .col(ColumnDef::new(P2PTransfers::SourceBankAccount).string())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-p2p_transfers-transaction_id")
                            .from(P2PTransfers::Table, P2PTransfers::TransactionId)
                            .to(Transactions::Table, Transactions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        // Transaction Edits (Audit Log)
        manager
            .create_table(
                Table::create()
                    .table(TransactionEdits::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TransactionEdits::Id).string().primary_key())
                    .col(
                        ColumnDef::new(TransactionEdits::TransactionId)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransactionEdits::OldAmount)
                            .decimal()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransactionEdits::NewAmount)
                            .decimal()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransactionEdits::EditedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-transaction_edits-transaction_id")
                            .from(TransactionEdits::Table, TransactionEdits::TransactionId)
                            .to(Transactions::Table, Transactions::Id),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TransactionEdits::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(P2PTransfers::Table).to_owned())
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::SourceWalletId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::DestinationWalletId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::LedgerTabId)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::DeletedAt)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(LedgerTabs::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Wallets::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Wallets {
    Table,
    Id,
    UserId,
    Name,
    Type,
    Balance,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum LedgerTabs {
    Table,
    Id,
    CreatorId,
    CounterpartyId,
    TabType,
    Title,
    TargetAmount,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    Id,
    SourceWalletId,
    DestinationWalletId,
    LedgerTabId,
    DeletedAt,
}

#[derive(DeriveIden)]
enum P2PTransfers {
    Table,
    Id,
    TransactionId,
    Direction,
    CounterpartyName,
    CounterpartyPhone,
    CounterpartyUpiId,
    IsMerchant,
    UpiTransactionId,
    GoogleTransactionId,
    SourceBankAccount,
}

#[derive(DeriveIden)]
enum TransactionEdits {
    Table,
    Id,
    TransactionId,
    OldAmount,
    NewAmount,
    EditedAt,
}
