use auth::adapter::SqliteAdapter;
use auth::{AuthSession, init_auth};
use axum::{
    Router,
    extract::{FromRef, Json, Multipart, Path, State},
    http::{HeaderName, HeaderValue, Method, StatusCode},
    routing::{delete, get, patch, post},
};
use better_auth::AxumIntegration;
use db::{SmartMerge, SplitDetail};
use ocr::OcrService;
use sea_orm::{Database, DatabaseConnection};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use upload::UploadClient;

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
    auth: Arc<better_auth::BetterAuth<SqliteAdapter>>,
    upload_client: UploadClient,
    ocr_service: Arc<OcrService>,
}

impl FromRef<AppState> for Arc<better_auth::BetterAuth<SqliteAdapter>> {
    fn from_ref(state: &AppState) -> Self {
        state.auth.clone()
    }
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info,server=debug,better_auth=debug,sqlx=warn".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(false)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false),
        )
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = Database::connect(&database_url).await?;
    let auth = init_auth(db.clone()).await?;
    let ocr_service = Arc::new(OcrService::new().await?);

    // S3/R2 Setup
    let mut endpoint = std::env::var("S3_ENDPOINT").expect("S3_ENDPOINT must be set");
    // Strip bucket suffix if present (e.g. /bucket-name)
    if let Some(pos) = endpoint.rfind(".com/") {
        endpoint.truncate(pos + 4);
    }

    let access_key_id = std::env::var("S3_ACCESS_KEY_ID").expect("S3_ACCESS_KEY_ID must be set");
    let secret_access_key =
        std::env::var("S3_SECRET_ACCESS_KEY").expect("S3_SECRET_ACCESS_KEY must be set");

    let s3_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .endpoint_url(endpoint)
        .region(aws_config::Region::new("auto"))
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            access_key_id,
            secret_access_key,
            None,
            None,
            "static",
        ))
        .load()
        .await;

    let s3_client_config = aws_sdk_s3::config::Builder::from(&s3_config)
        .force_path_style(true)
        .build();

    let s3_client = aws_sdk_s3::Client::from_conf(s3_client_config);
    let bucket_name = std::env::var("S3_BUCKET_NAME").expect("S3_BUCKET_NAME must be set");
    let upload_client = UploadClient::new(s3_client, bucket_name);

    let state = AppState {
        db,
        auth: auth.clone(),
        upload_client,
        ocr_service,
    };

    let auth_router = auth.clone().axum_router();

    let api_router = Router::new()
        .route("/transactions", get(list_transactions_handler))
        .route(
            "/transactions/manual",
            post(create_manual_transaction_handler),
        )
        .route("/transactions/:id", patch(update_transaction_handler))
        .route("/transactions/:id", delete(delete_transaction_handler))
        .route("/transactions/split", post(split_transaction_handler))
        .route("/p2p/pending", get(list_pending_p2p_handler))
        .route("/process-ocr", post(process_ocr_handler))
        .route("/p2p/create", post(create_p2p_handler))
        .route("/p2p/accept", post(accept_p2p_handler))
        .route("/groups", get(list_groups_handler))
        .route("/groups/create", post(create_group_handler))
        .route("/groups/invite", post(invite_to_group_handler))
        .route(
            "/groups/:id/transactions",
            get(list_group_transactions_handler),
        )
        .route("/subscriptions/detect", get(detect_subscriptions_handler))
        .route("/upload/presigned", post(get_presigned_url_handler))
        .route("/upload", post(direct_upload_handler))
        .route("/process-image-ocr", post(process_image_ocr_handler));

    let app = Router::new()
        .nest("/api/auth", auth_router.with_state(auth.clone()))
        .nest("/api", api_router)
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:3000".parse::<HeaderValue>().unwrap(),
                    "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),
                ])
                .allow_methods(tower_http::cors::Any)
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                    axum::http::header::COOKIE,
                    HeaderName::from_static("x-better-auth-origin"),
                ])
                .allow_credentials(true),
        )
        .with_state(state);

    let port = std::env::var("API_PORT").unwrap_or_else(|_| "8080".into());
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse()?;

    tracing::info!("Server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn list_transactions_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::transactions::Model>>, (StatusCode, String)> {
    let result = SmartMerge::list_transactions(&state.db, &session.user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

#[derive(Deserialize)]
struct UpdateTransactionRequest {
    amount: Option<rust_decimal::Decimal>,
    date: Option<chrono::DateTime<chrono::FixedOffset>>,
    purpose_tag: Option<String>,
    status: Option<db::entities::enums::TransactionStatus>,
}

async fn update_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
    Json(payload): Json<UpdateTransactionRequest>,
) -> Result<Json<db::entities::transactions::Model>, (StatusCode, String)> {
    let result = SmartMerge::update_transaction(
        &state.db,
        &session.user.id,
        &id,
        payload.amount,
        payload.date,
        payload.purpose_tag,
        payload.status,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

async fn delete_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    SmartMerge::delete_transaction(&state.db, &session.user.id, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct SplitTransactionRequest {
    transaction_id: String,
    splits: Vec<SplitDetail>,
}

async fn split_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<SplitTransactionRequest>,
) -> Result<Json<Vec<db::entities::p2p_requests::Model>>, (StatusCode, String)> {
    let result = SmartMerge::split_transaction(
        &state.db,
        &session.user.id,
        &payload.transaction_id,
        payload.splits,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

async fn list_pending_p2p_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::p2p_requests::Model>>, (StatusCode, String)> {
    let email = session
        .user
        .email
        .clone()
        .ok_or((StatusCode::BAD_REQUEST, "Email missing".to_string()))?;
    let result = SmartMerge::list_pending_p2p_requests(&state.db, &email)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

async fn process_ocr_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(processed_ocr): Json<db::ProcessedOcr>,
) -> Result<Json<db::entities::transactions::Model>, (StatusCode, String)> {
    let result = SmartMerge::process_ocr(&state.db, &session.user.id, processed_ocr)
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
    session: AuthSession,
    Json(payload): Json<CreateP2PRequest>,
) -> Result<Json<db::entities::p2p_requests::Model>, (StatusCode, String)> {
    let result = SmartMerge::create_p2p_request(
        &state.db,
        &session.user.id,
        &payload.receiver_email,
        &payload.transaction_id,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

async fn accept_p2p_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<AcceptP2PRequest>,
) -> Result<Json<db::entities::p2p_requests::Model>, (StatusCode, String)> {
    let result = SmartMerge::accept_p2p_request(&state.db, &session.user.id, &payload.request_id)
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
    session: AuthSession,
    Json(payload): Json<CreateGroupRequest>,
) -> Result<Json<db::entities::groups::Model>, (StatusCode, String)> {
    let result = SmartMerge::create_group(
        &state.db,
        &session.user.id,
        &payload.name,
        payload.description,
    )
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
    session: AuthSession,
    Json(payload): Json<InviteGroupRequest>,
) -> Result<Json<db::entities::p2p_requests::Model>, (StatusCode, String)> {
    let result = SmartMerge::invite_to_group(
        &state.db,
        &session.user.id,
        &payload.receiver_email,
        &payload.group_id,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

async fn list_groups_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::groups::Model>>, (StatusCode, String)> {
    let result = SmartMerge::list_groups(&state.db, &session.user.id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

async fn list_group_transactions_handler(
    State(state): State<AppState>,
    _session: AuthSession,
    Path(group_id): Path<String>,
) -> Result<Json<Vec<db::entities::transactions::Model>>, (StatusCode, String)> {
    let result = SmartMerge::list_group_transactions(&state.db, &group_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

async fn detect_subscriptions_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<Vec<db::entities::subscriptions::Model>>, (StatusCode, String)> {
    let result = SmartMerge::detect_subscriptions(&state.db, &session.user.id)
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
    session: AuthSession,
    Json(payload): Json<PresignedUrlRequest>,
) -> Result<Json<PresignedUrlResponse>, (StatusCode, String)> {
    let (url, key) = state
        .upload_client
        .get_presigned_url(
            &session.user.id,
            &payload.file_name,
            &payload.content_type,
            Duration::from_secs(3600),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PresignedUrlResponse { url, key }))
}

async fn direct_upload_handler(
    State(state): State<AppState>,
    session: AuthSession,
    mut multipart: Multipart,
) -> Result<Json<PresignedUrlResponse>, (StatusCode, String)> {
    tracing::debug!("📁 Received upload request for user: {}", session.user.id);
    let mut file_data = None;
    let mut file_name = String::new();
    let mut content_type = String::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("❌ Multipart next_field error: {:?}", e);
        (StatusCode::BAD_REQUEST, e.to_string())
    })? {
        let name = field.name().unwrap_or_default().to_string();
        tracing::debug!("📦 Processing multipart field: {}", name);
        if name == "file" {
            file_name = field.file_name().unwrap_or("unnamed").to_string();
            content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            tracing::debug!(
                "📎 Extracting bytes for file: {} ({})",
                file_name,
                content_type
            );
            file_data = Some(field.bytes().await.map_err(|e| {
                tracing::error!("❌ Multipart bytes extraction error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            })?);
            break;
        }
    }

    let data = file_data.ok_or_else(|| {
        tracing::warn!("⚠️ No file data found in multipart request");
        (StatusCode::BAD_REQUEST, "No file uploaded".to_string())
    })?;

    tracing::info!(
        "🚀 Starting direct upload for file: {} ({} bytes)",
        file_name,
        data.len()
    );
    let processed = state
        .upload_client
        .upload_direct(
            &session.user.id,
            data,
            Some(file_name),
            Some(content_type),
            true,
        )
        .await
        .map_err(|e| {
            tracing::error!("❌ UploadClient upload_direct failed: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    tracing::info!("✅ Upload successful, key: {}", processed.key);
    let bucket_name =
        std::env::var("S3_BUCKET_NAME").unwrap_or_else(|_| "expent-uploads".to_string());

    Ok(Json(PresignedUrlResponse {
        url: format!(
            "https://{}.r2.cloudflarestorage.com/{}",
            bucket_name, processed.key
        ),
        key: processed.key,
    }))
}

#[derive(Deserialize)]
struct ProcessImageOcrRequest {
    key: String,
}

async fn process_image_ocr_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<ProcessImageOcrRequest>,
) -> Result<Json<db::entities::transactions::Model>, (StatusCode, String)> {
    // Security check: Ensure the key starts with the user ID to prevent IDOR
    let user_id_prefix = format!("{}/", session.user.id);
    if !payload.key.starts_with(&user_id_prefix) {
        tracing::warn!(
            "🔒 Potential IDOR attempt by user {} for key {}",
            session.user.id,
            payload.key
        );
        return Err((
            StatusCode::FORBIDDEN,
            "You do not have permission to access this file".to_string(),
        ));
    }

    let bytes = state
        .upload_client
        .get_file(&payload.key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Determine filename and mime type from the key or payload
    let filename = payload.key.split("/").last().unwrap_or("upload");
    let mime_type = if filename.ends_with(".pdf") {
        "application/pdf"
    } else if filename.ends_with(".csv") {
        "text/csv"
    } else {
        "image/png" // Default for screenshots
    };

    let ocr_json = state
        .ocr_service
        .process_file(&bytes, filename, mime_type)
        .await
        .map_err(|e| {
            // Check if this is a reqwest error with a 429 status code
            if let Some(re) = e.downcast_ref::<reqwest::Error>() {
                if let Some(status) = re.status() {
                    if status == StatusCode::TOO_MANY_REQUESTS {
                        return (
                            StatusCode::TOO_MANY_REQUESTS,
                            "Gemini API quota exceeded. Please try again later.".to_string(),
                        );
                    }
                }
            }
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    let processed_ocr: db::ProcessedOcr = serde_json::from_value(ocr_json).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to parse OCR response: {}", e),
        )
    })?;

    let result = SmartMerge::process_ocr(&state.db, &session.user.id, processed_ocr)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

#[derive(Deserialize)]
struct CreateManualTransactionRequest {
    amount: rust_decimal::Decimal,
    purpose_tag: Option<String>,
    direction: db::entities::enums::TransactionDirection,
    date: chrono::DateTime<chrono::FixedOffset>,
}

async fn create_manual_transaction_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<CreateManualTransactionRequest>,
) -> Result<Json<db::entities::transactions::Model>, (StatusCode, String)> {
    let result = SmartMerge::create_transaction(
        &state.db,
        &session.user.id,
        payload.amount,
        payload.direction,
        payload.date,
        db::entities::enums::TransactionSource::Manual,
        payload.purpose_tag,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}
