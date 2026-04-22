use db::AppError;
use db::entities;
use sea_orm::DatabaseConnection;

pub mod ops;

#[derive(Debug, Clone)]
pub struct UsersManager {
    db: DatabaseConnection,
}

impl UsersManager {
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn update_profile(
        &self,
        user_id: &str,
        name: Option<String>,
        username: Option<String>,
        image: Option<String>,
    ) -> Result<entities::users::Model, AppError> {
        ops::update_profile(&self.db, user_id, name, username, image).await
    }

    pub async fn list_upi(
        &self,
        user_id: &str,
    ) -> Result<Vec<entities::user_upi_ids::Model>, AppError> {
        ops::list_user_upi(&self.db, user_id).await
    }

    pub async fn add_upi(
        &self,
        user_id: &str,
        upi_id: String,
        label: Option<String>,
    ) -> Result<entities::user_upi_ids::Model, AppError> {
        ops::add_user_upi(&self.db, user_id, upi_id, label).await
    }

    pub async fn make_primary_upi(&self, user_id: &str, upi_id: &str) -> Result<(), AppError> {
        ops::make_primary_upi(&self.db, user_id, upi_id).await
    }
}
