use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Groups
        manager
            .create_table(
                Table::create()
                    .table(Groups::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Groups::Id).string().primary_key())
                    .col(ColumnDef::new(Groups::Name).string().not_null())
                    .col(ColumnDef::new(Groups::Description).string())
                    .col(ColumnDef::new(Groups::CreatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // User Groups (Many-to-Many)
        manager
            .create_table(
                Table::create()
                    .table(UserGroups::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserGroups::UserId).string().not_null())
                    .col(ColumnDef::new(UserGroups::GroupId).string().not_null())
                    .col(ColumnDef::new(UserGroups::Role).string().not_null().default("MEMBER"))
                    .primary_key(Index::create().col(UserGroups::UserId).col(UserGroups::GroupId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_groups-user_id")
                            .from(UserGroups::Table, UserGroups::UserId)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-user_groups-group_id")
                            .from(UserGroups::Table, UserGroups::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add group_id to transactions (optional, for direct group transactions)
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .add_column(ColumnDef::new(Transactions::GroupId).string())
                    .to_owned()
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(UserGroups::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Groups::Table).to_owned()).await?;
        
        manager
            .alter_table(
                Table::alter()
                    .table(Transactions::Table)
                    .drop_column(Transactions::GroupId)
                    .to_owned()
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Groups {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
}

#[derive(DeriveIden)]
enum UserGroups {
    Table,
    UserId,
    GroupId,
    Role,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Transactions {
    Table,
    GroupId,
}
