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
                    .add_column(ColumnDef::new(OcrJobs::PHash).string().null())
                    .add_column(
                        ColumnDef::new(OcrJobs::AutoConfirm)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(ColumnDef::new(OcrJobs::WalletId).string().null())
                    .add_column(ColumnDef::new(OcrJobs::CategoryId).string().null())
                    .add_column(ColumnDef::new(OcrJobs::TransactionId).string().null())
                    .add_column(ColumnDef::new(OcrJobs::StartedAt).date_time().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .drop_column(OcrJobs::PHash)
                    .drop_column(OcrJobs::AutoConfirm)
                    .drop_column(OcrJobs::WalletId)
                    .drop_column(OcrJobs::CategoryId)
                    .drop_column(OcrJobs::TransactionId)
                    .drop_column(OcrJobs::StartedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum OcrJobs {
    Table,
    PHash,
    AutoConfirm,
    WalletId,
    CategoryId,
    TransactionId,
    StartedAt,
}
