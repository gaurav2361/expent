use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let columns = vec![
            ColumnDef::new(OcrJobs::PHash).string().null().to_owned(),
            ColumnDef::new(OcrJobs::AutoConfirm)
                .boolean()
                .not_null()
                .default(false)
                .to_owned(),
            ColumnDef::new(OcrJobs::WalletId).string().null().to_owned(),
            ColumnDef::new(OcrJobs::CategoryId)
                .string()
                .null()
                .to_owned(),
            ColumnDef::new(OcrJobs::TransactionId)
                .string()
                .null()
                .to_owned(),
            ColumnDef::new(OcrJobs::StartedAt)
                .date_time()
                .null()
                .to_owned(),
        ];

        for col in columns {
            manager
                .alter_table(
                    Table::alter()
                        .table(OcrJobs::Table)
                        .add_column(col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let columns = vec![
            OcrJobs::PHash,
            OcrJobs::AutoConfirm,
            OcrJobs::WalletId,
            OcrJobs::CategoryId,
            OcrJobs::TransactionId,
            OcrJobs::StartedAt,
        ];

        for col in columns {
            manager
                .alter_table(
                    Table::alter()
                        .table(OcrJobs::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
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
