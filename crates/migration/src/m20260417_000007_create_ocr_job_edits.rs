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
                    .add_column(
                        ColumnDef::new(OcrJobEdits::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .add_column(ColumnDef::new(OcrJobEdits::OcrJobId).string().not_null())
                    .add_column(ColumnDef::new(OcrJobEdits::UserId).string().not_null())
                    .add_column(ColumnDef::new(OcrJobEdits::FieldName).string().not_null())
                    .add_column(ColumnDef::new(OcrJobEdits::OriginalValue).string().null())
                    .add_column(ColumnDef::new(OcrJobEdits::CorrectedValue).string().null())
                    .add_column(
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
