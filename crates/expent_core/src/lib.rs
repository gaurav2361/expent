pub mod services;

pub use db::AppError;
pub use db::GPayExtraction;
pub use db::LineItem;
pub use db::OcrResult;
pub use db::OcrTransactionResponse;
pub use db::P2PRequestWithSender;
pub use db::ProcessedOcr;
pub use db::SplitDetail;
pub use db::TransactionWithDetail;

pub use services::categories;
pub use services::contacts;
pub use services::groups;
pub use services::ocr;
pub use services::p2p;
pub use services::reconciliation;
pub use services::subscriptions;
pub use services::transactions;
pub use services::users;
pub use services::wallets;

// Re-export common crates so API doesn't need to depend on them directly
pub use auth;
pub use better_auth;
pub use ocr as ocr_engine;
pub use sea_orm;
pub use upload;

use ::ocr::OcrService;
use auth::adapter::PostgresAdapter;
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use upload::UploadClient;

#[derive(Clone)]
pub struct Core {
    pub db: DatabaseConnection,
    pub auth: Arc<better_auth::BetterAuth<PostgresAdapter>>,
    pub upload_client: UploadClient,
    pub ocr_service: Arc<OcrService>,
}

pub struct CoreConfig {
    pub database_url: String,
    pub s3_endpoint: String,
    pub s3_access_key_id: String,
    pub s3_secret_access_key: String,
    pub s3_bucket_name: String,
}

impl Core {
    pub async fn init(config: CoreConfig) -> Result<Self, anyhow::Error> {
        let db = Database::connect(&config.database_url).await?;

        // Initialize Auth
        let auth = auth::init_auth(db.clone())
            .await
            .map_err(|e| anyhow::anyhow!("Auth init failed: {e}"))?;

        // Initialize OCR
        let ocr_service = Arc::new(
            OcrService::new()
                .await
                .map_err(|e| anyhow::anyhow!("OCR init failed: {e}"))?,
        );

        // S3/R2 Setup
        let mut endpoint = config.s3_endpoint;
        if let Some(pos) = endpoint.rfind(".com/") {
            endpoint.truncate(pos + 4);
        }

        let s3_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(aws_config::Region::new("auto"))
            .credentials_provider(aws_sdk_s3::config::Credentials::new(
                config.s3_access_key_id,
                config.s3_secret_access_key,
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
        let upload_client = UploadClient::new(s3_client, config.s3_bucket_name);

        let core = Self {
            db,
            auth,
            upload_client,
            ocr_service,
        };

        // Ensure system categories exist
        if let Err(e) = categories::ensure_system_categories(&core.db).await {
            tracing::error!("Failed to ensure system categories: {:?}", e);
        }

        Ok(core)
    }
}
