use sea_orm::DatabaseConnection;

pub mod account;
pub mod others;
pub mod session;
pub mod user;
pub mod verification;

#[derive(Clone)]
pub struct SqliteAdapter {
    pub db: DatabaseConnection,
}

impl SqliteAdapter {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

// Note: DatabaseAdapter is blanket-implemented by better-auth
// when all 10 persistence traits are implemented.
