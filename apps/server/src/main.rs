pub use auth::AuthSession;
use auth::adapter::SqliteAdapter;
use auth::init_auth;
use axum::{
    Router,
    extract::FromRef,
    http::{HeaderValue, Method},
    routing::get,
};
use better_auth::AxumIntegration;
use ocr::OcrService;
use sea_orm::{Database, DatabaseConnection};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use upload::UploadClient;

pub mod extractors;
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
    dotenvy::dotenv().ok();

    let rust_log = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "info,server=debug,better_auth=info".into());

    let filter_string = if rust_log.contains("sqlx=") {
        rust_log
    } else {
        format!("{},sqlx=error,sea_orm=warn,tower_http=debug", rust_log)
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter_string))
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_target(false)
                // Disable thread IDs in logs for cleaner output
                .with_thread_ids(false)
                // Disable file and line numbers in logs to reduce log verbosity
                .with_file(false)
                .with_line_number(false),
        )
        .init();

    // Custom error type for environment variable issues
    #[derive(Debug)]
    struct EnvVarError(String);

    impl std::fmt::Display for EnvVarError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Environment variable error: {}", self.0)
        }
    }

    impl std::error::Error for EnvVarError {}

    let get_env_var = |name: &str| -> Result<String, EnvVarError> {
        std::env::var(name).map_err(|_| EnvVarError(format!("{} must be set", name)))
    };

    let database_url = get_env_var("DATABASE_URL")?;
    let s3_endpoint = get_env_var("S3_ENDPOINT")?;
    let access_key_id = get_env_var("S3_ACCESS_KEY_ID")?;
    let secret_access_key = get_env_var("S3_SECRET_ACCESS_KEY")?;
    let bucket_name = get_env_var("S3_BUCKET_NAME")?;

    let db = Database::connect(&database_url).await?;
    let auth = init_auth(db.clone()).await?;
    let ocr_service = Arc::new(OcrService::new().await?);

    // S3/R2 Setup
    let mut endpoint = s3_endpoint;
    // Strip bucket suffix if present (e.g. /bucket-name)
    if let Some(pos) = endpoint.rfind(".com/") {
        endpoint.truncate(pos + 4);
    }

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
    let upload_client = UploadClient::new(s3_client, bucket_name);

    let state = AppState {
        db,
        auth: auth.clone(),
        upload_client,
        ocr_service,
    };

    let auth_router = auth.clone().axum_router();

    // Rate limiting config: ~1 request per second per IP, burst of 10
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(10)
            .finish()
            // This unwrap is safe because the default configuration with positive per_second and burst_size values is always valid.
            .unwrap(),
    );

    let api_router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest("/transactions", routes::transactions::router())
        .nest("/p2p", routes::p2p::router())
        .nest("/groups", routes::groups::router())
        .nest("/contacts", routes::contacts::router())
        .nest("/wallets", routes::wallets::router())
        .nest("/users", routes::users::router())
        .nest("/categories", routes::categories::router())
        .nest("/subscriptions", routes::subscriptions::router())
        .nest("/reconciliation", routes::reconciliation::router())
        .nest("/upload", routes::uploads::router())
        .nest("/ocr", routes::ocr::router())
        .layer(GovernorLayer::new(governor_conf));

    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://127.0.0.1:3000".to_string())
        .split(',')
        .map(|s| s.parse::<HeaderValue>().unwrap())
        .collect::<Vec<_>>();

    let app = Router::new()
        .nest("/api/auth", auth_router.with_state(auth.clone()))
        .nest("/api", api_router)
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::DefaultBodyLimit::max(10 * 1024 * 1024))
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origins)
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
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
