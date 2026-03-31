use axum::{
    extract::{State, Json},
    routing::{post, get},
    Router,
    http::{HeaderMap, StatusCode},
};
use better_auth::{AuthConfig, AxumIntegration, AuthBuilder, AuthRequest, HttpMethod};
use better_auth::plugins::EmailPasswordPlugin;
use better_auth::adapters::SqlxAdapter;
use db::{SmartMerge, OcrResult};
use sea_orm::{DatabaseConnection, Database};
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize};

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
    auth: Arc<better_auth::BetterAuth<SqlxAdapter>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let db = Database::connect(&database_url).await?;
    let adapter = SqlxAdapter::new(&database_url).await?;
    
    let auth_instance = AuthBuilder::new(AuthConfig::new(auth_secret).base_url(base_url))
        .database(adapter)
        .plugin(EmailPasswordPlugin::new())
        .build()
        .await?;

    let auth = Arc::new(auth_instance);
    let state = AppState { db, auth: auth.clone() };

    let auth_router = auth.clone().axum_router().with_state(auth.clone());

    let api_router = Router::new()
        .route("/transactions", get(list_transactions_handler))
        .route("/p2p/pending", get(list_pending_p2p_handler))
        .route("/process-ocr", post(process_ocr_handler))
        .route("/p2p/create", post(create_p2p_handler))
        .route("/p2p/accept", post(accept_p2p_handler))
        .with_state(state);

    let app = Router::new()
        .nest("/api/auth", auth_router)
        .merge(api_router)
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(CorsLayer::permissive());

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".into());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;
    
    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Deserialize)]
struct SessionResponse {
    session: SessionInfo,
    user: UserInfo,
}

#[derive(Deserialize)]
struct SessionInfo {
    #[serde(rename = "userId")]
    user_id: String,
}

#[derive(Deserialize)]
struct UserInfo {
    email: String,
}

async fn get_user_data(auth: &better_auth::BetterAuth<SqlxAdapter>, headers: HeaderMap) -> Result<(String, String), (StatusCode, String)> {
    let mut mapped_headers = HashMap::new();
    for (name, value) in headers.iter() {
        if let Ok(val_str) = value.to_str() {
            mapped_headers.insert(name.as_str().to_string(), val_str.to_string());
        }
    }

    let auth_req = AuthRequest::from_parts(
        HttpMethod::Get,
        "/get-session".to_string(),
        mapped_headers,
        None,
        HashMap::new()
    );

    let response = auth.handle_request(auth_req).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if response.status != 200 {
        return Err((StatusCode::UNAUTHORIZED, "Unauthorized".to_string()));
    }

    let body_bytes = response.body;
    if body_bytes.is_empty() {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Empty response body".to_string()));
    }

    let session_data: SessionResponse = serde_json::from_slice(&body_bytes)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to parse session: {}", e)))?;
    
    Ok((session_data.session.user_id, session_data.user.email))
}

async fn list_transactions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<db::entities::transaction::Model>>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::list_transactions(&state.db, &user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

async fn list_pending_p2p_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<db::entities::p2p_request::Model>>, (StatusCode, String)> {
    let (_, email) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::list_pending_p2p_requests(&state.db, &email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

async fn process_ocr_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(ocr_data): Json<OcrResult>,
) -> Result<Json<db::entities::transaction::Model>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::process_ocr(&state.db, &user_id, ocr_data)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

#[derive(Deserialize)]
struct CreateP2PRequest {
    receiver_email: String,
    transaction_id: String,
}

async fn create_p2p_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateP2PRequest>,
) -> Result<Json<db::entities::p2p_request::Model>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::create_p2p_request(
        &state.db, 
        &user_id, 
        &payload.receiver_email, 
        &payload.transaction_id
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

#[derive(Deserialize)]
struct AcceptP2PRequest {
    request_id: String,
}

async fn accept_p2p_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AcceptP2PRequest>,
) -> Result<Json<db::entities::p2p_request::Model>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::accept_p2p_request(
        &state.db,
        &user_id,
        &payload.request_id
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}
