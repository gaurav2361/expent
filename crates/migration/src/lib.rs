pub use sea_orm_migration::prelude::*;

pub mod m20220101_000001_create_table;
pub mod m20260331_092335_create_groups_table;
pub mod m20260331_181001_add_missing_user_fields;
pub mod m20260331_185523_add_associated_contact_id;
pub mod m20260401_000001_add_indexes;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20260331_092335_create_groups_table::Migration),
            Box::new(m20260331_181001_add_missing_user_fields::Migration),
            Box::new(m20260331_185523_add_associated_contact_id::Migration),
            Box::new(m20260401_000001_add_indexes::Migration),
        ]
    }
}
