use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add user_id to bank_statement_rows
        manager
            .alter_table(
                Table::alter()
                    .table(BankStatementRows::Table)
                    .add_column(
                        ColumnDef::new(BankStatementRows::UserId)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        // Add is_matched to bank_statement_rows
        manager
            .alter_table(
                Table::alter()
                    .table(BankStatementRows::Table)
                    .add_column(
                        ColumnDef::new(BankStatementRows::IsMatched)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // Add matched_at to statement_txn_matches
        manager
            .alter_table(
                Table::alter()
                    .table(StatementTxnMatches::Table)
                    .add_column(
                        ColumnDef::new(StatementTxnMatches::MatchedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(StatementTxnMatches::Table)
                    .drop_column(StatementTxnMatches::MatchedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BankStatementRows::Table)
                    .drop_column(BankStatementRows::IsMatched)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(BankStatementRows::Table)
                    .drop_column(BankStatementRows::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum BankStatementRows {
    Table,
    UserId,
    IsMatched,
}

#[derive(DeriveIden)]
enum StatementTxnMatches {
    Table,
    MatchedAt,
}
