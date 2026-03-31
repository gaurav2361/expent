use axum::{
    extract::{FromRequestParts, FromRef},
    http::{request::Parts, StatusCode},
};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};

// We'll use the entities from our own db crate
use db::entities::{user, session};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: String,
    pub email: String,
    pub name: String,
}

pub struct AuthSession {
    pub user: AuthUser,
}

impl<S> FromRequestParts<S> for AuthSession
where
    S: Send + Sync,
    DatabaseConnection: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = DatabaseConnection::from_ref(state);

        // Get token from cookie or Authorization header
        let token = parts.headers.get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .or_else(|| {
                parts.headers.get("cookie")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| {
                        s.split(';')
                            .find(|p| p.trim().starts_with("better-auth.session-token="))
                            .map(|p| p.trim().split('=').nth(1).unwrap_or(""))
                    })
            });

        let token = match token {
            Some(t) => t,
            None => return Err((StatusCode::UNAUTHORIZED, "Missing session token".to_string())),
        };

        // Query session and user
        let session_user = session::Entity::find()
            .filter(session::Column::Token.eq(token))
            .find_also_related(user::Entity)
            .one(&db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        match session_user {
            Some((_sess, Some(u))) => {
                Ok(AuthSession {
                    user: AuthUser {
                        id: u.id,
                        email: u.email,
                        name: u.name,
                    },
                })
            }
            _ => Err((StatusCode::UNAUTHORIZED, "Invalid or expired session".to_string())),
        }
    }
}
