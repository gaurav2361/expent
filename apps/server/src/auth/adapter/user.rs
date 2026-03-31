use crate::auth::adapter::SqliteAdapter;
use async_trait::async_trait;
use better_auth::types_mod::{
    AuthError, AuthResult, CreateUser, ListUsersParams, UpdateUser, User, UserOps,
};
use chrono::{DateTime, Utc, FixedOffset};
use sea_orm::{EntityTrait, Set, ActiveModelTrait, QueryOrder, QueryFilter, ColumnTrait};
use db::entities::user;

#[async_trait]
impl UserOps for SqliteAdapter {
    type User = User;

    async fn create_user(&self, data: CreateUser) -> AuthResult<Self::User> {
        let id = data.id.clone().unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let now: DateTime<FixedOffset> = Utc::now().into();

        let active_model = user::ActiveModel {
            id: Set(id.clone()),
            name: Set(data.name.unwrap_or_default()),
            email: Set(data.email.unwrap_or_default()),
            email_verified: Set(data.email_verified.unwrap_or(false)),
            image: Set(data.image),
            phone: Set(None),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
        };

        active_model.insert(&self.db).await
            .map_err(|e| AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string())))?;

        self.get_user_by_id(&id).await?.ok_or(AuthError::UserNotFound)
    }

    async fn get_user_by_id(&self, id: &str) -> AuthResult<Option<Self::User>> {
        let model = user::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string())))?;

        Ok(model.map(map_model_to_user))
    }

    async fn get_user_by_email(&self, email: &str) -> AuthResult<Option<Self::User>> {
        let model = user::Entity::find()
            .filter(user::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|e| AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string())))?;

        Ok(model.map(map_model_to_user))
    }

    async fn get_user_by_username(&self, _username: &str) -> AuthResult<Option<Self::User>> {
        Ok(None)
    }

    async fn update_user(&self, id: &str, update: UpdateUser) -> AuthResult<Self::User> {
        let model = user::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|e| AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string())))?
            .ok_or(AuthError::UserNotFound)?;

        let mut active_model: user::ActiveModel = model.into();
        active_model.updated_at = Set(Utc::now().into());

        if let Some(name) = update.name { active_model.name = Set(name); }
        if let Some(email) = update.email { active_model.email = Set(email); }
        if let Some(ev) = update.email_verified { active_model.email_verified = Set(ev); }
        if let Some(image) = update.image { active_model.image = Set(Some(image)); }

        active_model.update(&self.db).await
            .map_err(|e| AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string())))?;

        self.get_user_by_id(id).await?.ok_or(AuthError::UserNotFound)
    }

    async fn delete_user(&self, id: &str) -> AuthResult<()> {
        user::Entity::delete_by_id(id.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string())))?;
        Ok(())
    }

    async fn list_users(&self, _params: ListUsersParams) -> AuthResult<(Vec<Self::User>, usize)> {
        let models = user::Entity::find()
            .order_by_desc(user::Column::CreatedAt)
            .all(&self.db)
            .await
            .map_err(|e| AuthError::Database(better_auth::types_mod::DatabaseError::Query(e.to_string())))?;

        let count = models.len();
        let users = models.into_iter().map(map_model_to_user).collect();
        Ok((users, count))
    }
}

fn map_model_to_user(m: user::Model) -> User {
    User {
        id: m.id,
        name: Some(m.name),
        email: Some(m.email),
        email_verified: m.email_verified,
        image: m.image,
        created_at: m.created_at.into(),
        updated_at: m.updated_at.into(),
        username: None,
        display_username: None,
        two_factor_enabled: false,
        role: Some("user".to_string()),
        banned: false,
        ban_reason: None,
        ban_expires: None,
        metadata: serde_json::Value::Null,
    }
}
