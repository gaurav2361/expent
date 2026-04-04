use auth::adapter::SqliteAdapter;
pub use auth::AuthSession;
use auth::init_auth;
use axum::{
    extract::FromRef,
    http::{HeaderValue, Method},
    routing::{delete, get, patch, post, put},
    Router,
};
use better_auth::AxumIntegration;
use ocr::OcrService;
use sea_orm::{Database, DatabaseConnection};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use upload::UploadClient;

pub mod middleware;
pub mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub auth: Arc<better_auth::BetterAuth<SqliteAdapter>>,
    pub upload_client: UploadClient,
    pub ocr_service: Arc<OcrService>,
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
        .route(
            "/transactions",
            get(routes::transactions::list_transactions_handler),
        )
        .route(
            "/transactions/manual",
            post(routes::transactions::create_manual_transaction_handler),
        )
        .route(
            "/transactions/{id}",
            patch(routes::transactions::update_transaction_handler),
        )
        .route(
            "/transactions/{id}",
            delete(routes::transactions::delete_transaction_handler),
        )
        .route(
            "/transactions/split",
            post(routes::transactions::split_transaction_handler),
        )
        .route("/p2p/pending", get(routes::p2p::list_pending_p2p_handler))
        .route("/process-ocr", post(routes::ocr::process_ocr_handler))
        .route("/p2p/create", post(routes::p2p::create_p2p_handler))
        .route("/p2p/accept", post(routes::p2p::accept_p2p_handler))
        .route("/groups", get(routes::groups::list_groups_handler))
        .route("/groups/create", post(routes::groups::create_group_handler))
        .route("/groups/invite", post(routes::groups::invite_to_group_handler))
        .route(
            "/groups/{id}/transactions",
            get(routes::groups::list_group_transactions_handler),
        )
        .route("/contacts", get(routes::contacts::list_contacts_handler))
        .route("/contacts", post(routes::contacts::create_contact_handler))
        .route("/contacts/{id}", put(routes::contacts::update_contact_handler))
        .route(
            "/contacts/{id}",
            delete(routes::contacts::delete_contact_handler),
        )
        .route(
            "/contacts/{id}",
            get(routes::contacts::get_contact_detail_handler),
        )
        .route(
            "/contacts/{id}/identifiers",
            post(routes::contacts::add_contact_identifier_handler),
        )
        .route("/wallets", get(routes::wallets::list_wallets_handler))
        .route("/wallets", post(routes::wallets::create_wallet_handler))
        .route("/wallets/{id}", put(routes::wallets::update_wallet_handler))
        .route(
            "/subscriptions/detect",
            get(routes::subscriptions::detect_subscriptions_handler),
        )
        .route(
            "/upload/presigned",
            post(routes::uploads::get_presigned_url_handler),
        )
        .route("/upload", post(routes::uploads::direct_upload_handler))
        .route(
            "/process-image-ocr",
            post(routes::ocr::process_image_ocr_handler),
        );

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
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::PATCH,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::ACCEPT,
                ])
                .allow_credentials(true),
        )
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("🚀 Server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
