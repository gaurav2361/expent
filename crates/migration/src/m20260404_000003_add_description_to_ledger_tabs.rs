use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(LedgerTabs::Table)
                    .add_column(ColumnDef::new(LedgerTabs::Description).string())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(LedgerTabs::Table)
                    .drop_column(LedgerTabs::Description)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum LedgerTabs {
    Table,
    Description,
}
