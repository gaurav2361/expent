use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Wallets::Table)
                    .add_column(ColumnDef::new(Wallets::BankName).string().null())
                    .add_column(ColumnDef::new(Wallets::AccountNumber).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Wallets::Table)
                    .drop_column(Wallets::BankName)
                    .drop_column(Wallets::AccountNumber)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Wallets {
    Table,
    BankName,
    AccountNumber,
}
