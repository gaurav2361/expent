use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Create contact_staging table
        manager
            .create_table(
                Table::create()
                    .table(ContactStaging::Table)
                    .if_not_exists()
                    .add_column(
                        ColumnDef::new(ContactStaging::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .add_column(ColumnDef::new(ContactStaging::UserId).string().not_null())
                    .add_column(ColumnDef::new(ContactStaging::OcrJobId).string().not_null())
                    .add_column(ColumnDef::new(ContactStaging::Name).string().not_null())
                    .add_column(ColumnDef::new(ContactStaging::Phone).string().null())
                    .add_column(ColumnDef::new(ContactStaging::Email).string().null())
                    .add_column(ColumnDef::new(ContactStaging::UpiId).string().null())
                    .add_column(
                        ColumnDef::new(ContactStaging::Status)
                            .string()
                            .not_null()
                            .default("PENDING"),
                    )
                    .add_column(ColumnDef::new(ContactStaging::Candidates).json().null())
                    .add_column(
                        ColumnDef::new(ContactStaging::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .add_column(
                        ColumnDef::new(ContactStaging::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. Add resolution_candidates to ocr_jobs
        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .add_column(ColumnDef::new(OcrJobs::ResolutionCandidates).json().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(OcrJobs::Table)
                    .drop_column(OcrJobs::ResolutionCandidates)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(ContactStaging::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ContactStaging {
    Table,
    Id,
    UserId,
    OcrJobId,
    Name,
    Phone,
    Email,
    UpiId,
    Status,
    Candidates,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum OcrJobs {
    Table,
    ResolutionCandidates,
}
