use better_auth::plugins::{EmailPasswordPlugin, SessionManagementPlugin};
use better_auth::{AuthBuilder, AuthConfig, BetterAuth, AuthRequest, HttpMethod};
use axum::{
    extract::{FromRequestParts, FromRef},
    http::{request::Parts, StatusCode},
};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use sea_orm::DatabaseConnection;

pub mod adapter;
use crate::auth::adapter::SqliteAdapter;

pub struct AuthSession {
    pub user: better_auth::types_mod::User,
    pub session: better_auth::types_mod::Session,
}

impl<S> FromRequestParts<S> for AuthSession
where
    S: Send + Sync,
    Arc<BetterAuth<SqliteAdapter>>: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth = Arc::from_ref(state);

        let mut mapped_headers = HashMap::new();
        for (name, value) in parts.headers.iter() {
            if let Ok(val_str) = value.to_str() {
                mapped_headers.insert(name.as_str().to_string(), val_str.to_string());
            }
        }

        let auth_req = AuthRequest::from_parts(
            HttpMethod::Get,
            "/get-session".to_string(),
            mapped_headers,
            None,
            HashMap::new(),
        );

        let response = auth.handle_request(auth_req).await.map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

        if response.status != 200 {
            return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
        }

        #[derive(serde::Deserialize)]
        struct SessionResponse {
            session: better_auth::types_mod::Session,
            user: better_auth::types_mod::User,
        }

        let session_data: SessionResponse = serde_json::from_slice(&response.body).map_err(|e| {
            (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse session: {}", e))
        })?;

        tracing::info!("🔑 Auth Session extracted for user: {}", session_data.user.email.as_deref().unwrap_or("unknown"));

        Ok(AuthSession {
            session: session_data.session,
            user: session_data.user,
        })
    }
}

pub async fn init_auth(db: DatabaseConnection) -> Result<Arc<BetterAuth<SqliteAdapter>>, Box<dyn std::error::Error>> {
    let auth_secret = env::var("BETTER_AUTH_SECRET")
        .or_else(|_| env::var("BETTERAUTH_SECRET"))
        .expect("BETTER_AUTH_SECRET must be set");
    
    let base_url = env::var("BETTER_AUTH_BASE_URL")
        .or_else(|_| env::var("BASE_URL"))
        .unwrap_or_else(|_| "http://localhost:8080".into());

    let cors_origin = env::var("CORS_ORIGIN").unwrap_or_default();
    
    let mut trusted_origins = vec![
        "http://localhost:3000".to_string(),
        "http://127.0.0.1:3000".to_string(),
        base_url.clone(),
    ];

    if !cors_origin.is_empty() {
        trusted_origins.extend(cors_origin.split(',').map(|s| s.trim().to_string()));
    }

    trusted_origins.sort();
    trusted_origins.dedup();

    let adapter = SqliteAdapter::new(db);

    let enable_signup = env::var("ENABLE_SIGNUP")
        .map(|v| v != "false")
        .unwrap_or(true);

    let require_email_verification = env::var("REQUIRE_EMAIL_VERIFICATION")
        .map(|v| v != "false")
        .unwrap_or(false);

    let auth_instance = AuthBuilder::new(
        AuthConfig::new(auth_secret)
            .base_url(base_url)
            .trusted_origins(trusted_origins),
    )
    .database(adapter)
    .plugin(
        EmailPasswordPlugin::new()
            .enable_signup(enable_signup)
            .require_email_verification(require_email_verification),
    )
    .plugin(SessionManagementPlugin::new())
    .build()
    .await?;

    Ok(Arc::new(auth_instance))
}
