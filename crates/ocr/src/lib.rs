use anyhow::Result;
use reqwest::multipart;
use serde_json::Value;
use tracing::info;

pub struct OcrService {
    worker_url: String,
    client: reqwest::Client,
}

impl OcrService {
    pub async fn new() -> Result<Self> {
        let worker_url = std::env::var("OCR_WORKER_URL")
            .unwrap_or_else(|_| "http://localhost:8090/ocr".to_string());
        let client = reqwest::Client::new();
        info!(
            "🚀 OCR Service (Proxy) initialized with worker at: {}",
            worker_url
        );
        Ok(Self { worker_url, client })
    }

    pub async fn process_image(&self, image_bytes: &[u8]) -> Result<Value> {
        info!(
            "🖼️ Forwarding image for OCR processing ({} bytes)",
            image_bytes.len()
        );

        let part = multipart::Part::bytes(image_bytes.to_vec())
            .file_name("upload.png")
            .mime_str("image/png")?;

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

        info!("📝 OCR completed successfully via Python worker.");
        Ok(res)
    }
}
