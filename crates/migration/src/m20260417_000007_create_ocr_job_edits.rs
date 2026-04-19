use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(OcrJobEdits::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(OcrJobEdits::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(OcrJobEdits::OcrJobId).string().not_null())
                    .col(ColumnDef::new(OcrJobEdits::UserId).string().not_null())
                    .col(ColumnDef::new(OcrJobEdits::FieldName).string().not_null())
                    .col(ColumnDef::new(OcrJobEdits::OriginalValue).string().null())
                    .col(ColumnDef::new(OcrJobEdits::CorrectedValue).string().null())
                    .col(
                        ColumnDef::new(OcrJobEdits::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(OcrJobEdits::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum OcrJobEdits {
    Table,
    Id,
    OcrJobId,
    UserId,
    FieldName,
    OriginalValue,
    CorrectedValue,
    CreatedAt,
}
