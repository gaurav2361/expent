use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OcrJobs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OcrJobs::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OcrJobs::UserId).string().not_null())
                    .col(ColumnDef::new(OcrJobs::Status).string().not_null())
                    .col(ColumnDef::new(OcrJobs::R2Key).string().not_null())
                    .col(ColumnDef::new(OcrJobs::ProcessedData).json())
                    .col(ColumnDef::new(OcrJobs::Error).string())
                    .col(ColumnDef::new(OcrJobs::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(OcrJobs::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OcrJobs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum OcrJobs {
    Table,
    Id,
    UserId,
    Status,
    R2Key,
    ProcessedData,
    Error,
    CreatedAt,
    UpdatedAt,
}
