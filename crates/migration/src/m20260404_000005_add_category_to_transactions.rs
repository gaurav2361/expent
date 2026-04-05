use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DbBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(ColumnDef::new(Transactions::CategoryId).string().null())
                    .to_owned(),
            )
            .await?;

        // Sqlite does not support adding foreign keys via ALTER TABLE
        if manager.get_database_backend() != DbBackend::Sqlite {
            manager
                .create_foreign_key(
                    ForeignKey::create()
                        .name("fk-transactions-category_id")
                        .from(Transactions::Table, Transactions::CategoryId)
                        .to(Categories::Table, Categories::Id)
                        .on_delete(ForeignKeyAction::SetNull)
                        .to_owned(),
                )
                .await?;
        }

        // Add index on category_id
        manager
            .create_index(
                Index::create()
                    .name("idx-transactions-category_id")
                    .table(Transactions::Table)
                    .col(Transactions::CategoryId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-transactions-category_id")
                    .table(Transactions::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::CategoryId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    CategoryId,
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
}
