use sea_orm::DatabaseConnection;

pub mod account;
pub mod others;
pub mod session;
pub mod user;
pub mod verification;

#[derive(Clone)]
pub struct PostgresAdapter {
    pub db: DatabaseConnection,
}

impl PostgresAdapter {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}
