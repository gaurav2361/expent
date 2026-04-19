use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .add_column(
                        ColumnDef::new(OcrJobs::RetryCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .add_column(ColumnDef::new(OcrJobs::LastError).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .drop_column(OcrJobs::RetryCount)
                    .drop_column(OcrJobs::LastError)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum OcrJobs {
    Table,
    RetryCount,
    LastError,
}
