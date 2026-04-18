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
                    .add_column(ColumnDef::new(OcrJobs::SchemaVersion).integer().not_null().default(1))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .drop_column(OcrJobs::SchemaVersion)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum OcrJobs {
    Table,
    SchemaVersion,
}
