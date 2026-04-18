use anyhow::Result;
use reqwest::multipart;
use serde_json::Value;
use std::sync::Arc;
use tracing::info;
use sea_orm::DatabaseConnection;

pub mod jobs;
pub mod worker;

pub use jobs::*;
pub use worker::*;

#[derive(Clone, serde::Serialize)]
pub struct OcrUpdate {
    pub user_id: String,
    pub job_id: String,
    pub status: String,
    pub trace_id: Option<String>,
}

#[derive(Debug)]
pub struct OcrService {
    worker_url: String,
    client: reqwest::Client,
}

impl OcrService {
    pub async fn new() -> Result<Self> {
        let mut url =
            std::env::var("OCR_WORKER_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());

        // Intelligently determine if we need to append /extract
        if !url.contains("/extract") && !url.contains("/ocr") && !url.contains("/process") {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("extract");
        }

        let client = reqwest::Client::new();
        info!("🚀 OCR Service (Proxy) initialized with worker at: {}", url);
        Ok(Self {
            worker_url: url,
            client,
        })
    }

    pub async fn process_file(
        &self,
        file_bytes: &[u8],
        filename: &str,
        mime_type: &str,
    ) -> Result<Value> {
        info!(
            "📄 Forwarding file '{}' ({}) for extraction ({} bytes)",
            filename,
            mime_type,
            file_bytes.len()
        );

        let part = multipart::Part::bytes(file_bytes.to_vec())
            .file_name(filename.to_string())
            .mime_str(mime_type)?;

        let form = multipart::Form::new().part("file", part);

        let res = self
            .client
            .post(&self.worker_url)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;

        info!("📝 Extraction completed successfully via Python worker.");
        Ok(res)
    }

    pub async fn process_image(&self, image_bytes: &[u8]) -> Result<Value> {
        self.process_file(image_bytes, "upload.png", "image/png")
            .await
    }
}

/// Central manager for the OCR lifecycle.
#[derive(Clone)]
pub struct OcrManager {
    pub service: Arc<OcrService>,
    pub db: DatabaseConnection,
    pub upload: upload::UploadClient,
    pub ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
}

impl OcrManager {
    pub fn new(
        service: Arc<OcrService>,
        db: DatabaseConnection,
        upload: upload::UploadClient,
        ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    ) -> Self {
        Self {
            service,
            db,
            upload,
            ocr_tx,
        }
    }

    pub async fn start_job(
        &self,
        user_id: &str,
        trace_id: Option<String>,
        key: &str,
        raw_key: Option<String>,
        p_hash: Option<String>,
        auto_confirm: bool,
        wallet_id: Option<String>,
        category_id: Option<String>,
    ) -> Result<db::entities::ocr_jobs::Model, db::AppError> {
        create_ocr_job(
            &self.db,
            user_id,
            trace_id,
            key,
            raw_key,
            p_hash,
            auto_confirm,
            wallet_id,
            category_id,
        )
        .await
    }

    pub async fn process_immediately(
        &self,
        processor: Arc<dyn OcrProcessor>,
        job_id: String,
    ) {
        let db = self.db.clone();
        let service = self.service.clone();
        let upload = self.upload.clone();
        let ocr_tx = self.ocr_tx.clone();

        tokio::spawn(async move {
            if let Err(e) = process_job(&db, service, &upload, ocr_tx, processor, job_id).await {
                tracing::error!("❌ Immediate OCR processing failed: {}", e);
            }
        });
    }

    pub fn spawn_workers(&self, processor: Arc<dyn OcrProcessor>) {
        tokio::spawn(start_recovery_worker(self.db.clone()));
        tokio::spawn(start_processor_worker(
            self.db.clone(),
            self.service.clone(),
            self.upload.clone(),
            self.ocr_tx.clone(),
            processor,
        ));
    }
}
