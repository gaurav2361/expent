use axum::Router;
use better_auth::{AuthConfig, AxumIntegration, AuthBuilder};
use better_auth::plugins::EmailPasswordPlugin;
use better_auth::adapters::SqlxAdapter;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let auth_secret = std::env::var("AUTH_SECRET").expect("AUTH_SECRET must be set");
    let base_url = std::env::var("AUTH_BASE_URL").unwrap_or_else(|_| "http://localhost:3001".into());

    // 1. Define Auth Configuration
    let config = AuthConfig::new(auth_secret)
        .base_url(base_url);

    // 2. Build Better Auth instance
    let adapter = SqlxAdapter::new(&database_url).await?;
    
    let auth = AuthBuilder::new(config)
        .database(adapter)
        .plugin(EmailPasswordPlugin::new())
        .build()
        .await?;

    // 3. Wrap in Arc for Axum
    let auth = Arc::new(auth);

    // 4. Build Axum Router
    let app = Router::new()
        .nest("/api/auth", auth.clone().axum_router())
        .with_state(auth)
        .layer(CorsLayer::permissive());

    // 5. Start Server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".into());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    tracing::info!("Auth service listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
