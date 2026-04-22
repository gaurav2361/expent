use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Budgets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Budgets::Id)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Budgets::UserId).string().not_null())
                    .col(ColumnDef::new(Budgets::CategoryId).string())
                    .col(
                        ColumnDef::new(Budgets::Amount)
                            .decimal_len(16, 4)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Budgets::Period)
                            .string()
                            .not_null()
                            .default("MONTHLY"),
                    )
                    .col(
                        ColumnDef::new(Budgets::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Budgets::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-budgets-user_id")
                            .from(Budgets::Table, Budgets::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-budgets-category_id")
                            .from(Budgets::Table, Budgets::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Budgets::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Budgets {
    Table,
    Id,
    UserId,
    CategoryId,
    Amount,
    Period,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
}
