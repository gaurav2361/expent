use axum::{
    extract::{State, Json, Path},
    routing::{post, get},
    Router,
    http::{HeaderMap, StatusCode},
};
use better_auth::{AuthConfig, AxumIntegration, AuthBuilder, AuthRequest, HttpMethod};
use better_auth::plugins::EmailPasswordPlugin;
use better_auth::adapters::SqlxAdapter;
use db::{SmartMerge, OcrResult, SplitDetail};
use sea_orm::{DatabaseConnection, Database};
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde::{Deserialize};
use aws_sdk_s3::presigning::PresigningConfig;
use std::time::Duration;

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
    auth: Arc<better_auth::BetterAuth<SqlxAdapter>>,
    s3_client: aws_sdk_s3::Client,
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

    // S3/R2 Setup
    let endpoint = std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT must be set");
    let access_key_id = std::env::var("S3_ACCESS_KEY_ID").expect("S3_ACCESS_KEY_ID must be set");
    let secret_access_key = std::env::var("S3_SECRET_ACCESS_KEY").expect("S3_SECRET_ACCESS_KEY must be set");
    
    let s3_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(endpoint)
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "static",
        ))
        .load()
        .await;
    let s3_client = aws_sdk_s3::Client::new(&s3_config);

    let auth = Arc::new(auth_instance);
    let state = AppState { db, auth: auth.clone(), s3_client };

    let auth_router = auth.clone().axum_router().with_state(auth.clone());

    let api_router = Router::new()
        .route("/transactions", get(list_transactions_handler))
        .route("/transactions/split", post(split_transaction_handler))
        .route("/p2p/pending", get(list_pending_p2p_handler))
        .route("/process-ocr", post(process_ocr_handler))
        .route("/p2p/create", post(create_p2p_handler))
        .route("/p2p/accept", post(accept_p2p_handler))
        .route("/groups", get(list_groups_handler))
        .route("/groups/create", post(create_group_handler))
        .route("/groups/invite", post(invite_to_group_handler))
        .route("/groups/{id}/transactions", get(list_group_transactions_handler))
        .route("/subscriptions/detect", get(detect_subscriptions_handler))
        .route("/upload/presigned", post(get_presigned_url_handler))
        .with_state(state);

    let app = Router::new()
        .nest("/api/auth", auth_router)
        .merge(api_router)
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<axum::http::HeaderValue>().unwrap())
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PUT,
                    axum::http::Method::DELETE,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                    axum::http::header::COOKIE,
                ])
                .allow_credentials(true),
        );

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

#[derive(Deserialize)]
struct SplitTransactionRequest {
    transaction_id: String,
    splits: Vec<SplitDetail>,
}

async fn split_transaction_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SplitTransactionRequest>,
) -> Result<Json<Vec<db::entities::p2p_request::Model>>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::split_transaction(
        &state.db, 
        &user_id, 
        &payload.transaction_id, 
        payload.splits
    )
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

#[derive(Deserialize)]
struct AcceptP2PRequest {
    request_id: String,
}

#[derive(Deserialize)]
struct CreateGroupRequest {
    name: String,
    description: Option<String>,
}

async fn create_group_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<Json<db::entities::group::Model>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::create_group(&state.db, &user_id, &payload.name, payload.description)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

#[derive(Deserialize)]
struct InviteGroupRequest {
    group_id: String,
    receiver_email: String,
}

async fn invite_to_group_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<InviteGroupRequest>,
) -> Result<Json<db::entities::p2p_request::Model>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::invite_to_group(&state.db, &user_id, &payload.receiver_email, &payload.group_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

async fn list_groups_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<db::entities::group::Model>>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::list_groups(&state.db, &user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

async fn list_group_transactions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(group_id): Path<String>,
) -> Result<Json<Vec<db::entities::transaction::Model>>, (StatusCode, String)> {
    let _ = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::list_group_transactions(&state.db, &group_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

async fn detect_subscriptions_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<db::entities::subscription::Model>>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    let result = SmartMerge::detect_subscriptions(&state.db, &user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

#[derive(Deserialize)]
struct PresignedUrlRequest {
    #[serde(rename = "contentType")]
    content_type: String,
    #[serde(rename = "fileName")]
    file_name: String,
}

#[derive(serde::Serialize)]
struct PresignedUrlResponse {
    url: String,
    key: String,
}

async fn get_presigned_url_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<PresignedUrlRequest>,
) -> Result<Json<PresignedUrlResponse>, (StatusCode, String)> {
    let (user_id, _) = get_user_data(&state.auth, headers).await?;
    
    let bucket_name = std::env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set");
    let key = format!("{}/{}-{}", user_id, uuid::Uuid::new_v4(), payload.file_name);

    let presigning_config = PresigningConfig::expires_in(Duration::from_secs(3600))
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let presigned_request = state.s3_client
        .put_object()
        .bucket(bucket_name)
        .key(&key)
        .content_type(payload.content_type)
        .presigned(presigning_config)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PresignedUrlResponse {
        url: presigned_request.uri().to_string(),
        key,
    }))
}
