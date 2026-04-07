use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // As evaluated in review, attempting to batch ADD COLUMN operations
        // via raw SQL string manipulation (to work around SeaORM's TableAlterStatement
        // overwriting operations) breaks Iden abstractions and introduces brittle code.
        // Since SQLite doesn't support multiple ADD COLUMN statements anyway, and
        // this is a one-time migration, we prioritize framework safety over this optimization.
        let columns = vec![
            ColumnDef::new(Users::Username).string().to_owned(),
            ColumnDef::new(Users::DisplayUsername).string().to_owned(),
            ColumnDef::new(Users::Role).string().to_owned(),
            ColumnDef::new(Users::Banned).boolean().to_owned(),
            ColumnDef::new(Users::BanReason).string().to_owned(),
            ColumnDef::new(Users::BanExpires)
                .timestamp_with_time_zone()
                .to_owned(),
            ColumnDef::new(Users::TwoFactorEnabled).boolean().to_owned(),
            ColumnDef::new(Users::PhoneNumber).string().to_owned(),
            ColumnDef::new(Users::PhoneNumberVerified)
                .boolean()
                .to_owned(),
            ColumnDef::new(Users::Metadata).json().to_owned(),
        ];

        for col in columns {
            manager
                .alter_table(
                    Table::alter()
                        .table(Users::Table)
                        .add_column(col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let columns = vec![
            Users::Username,
            Users::DisplayUsername,
            Users::Role,
            Users::Banned,
            Users::BanReason,
            Users::BanExpires,
            Users::TwoFactorEnabled,
            Users::PhoneNumber,
            Users::PhoneNumberVerified,
            Users::Metadata,
        ];

        for col in columns {
            manager
                .alter_table(
                    Table::alter()
                        .table(Users::Table)
                        .drop_column(col)
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Username,
    DisplayUsername,
    Role,
    Banned,
    BanReason,
    BanExpires,
    TwoFactorEnabled,
    PhoneNumber,
    PhoneNumberVerified,
    Metadata,
}
