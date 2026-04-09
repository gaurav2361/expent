use anyhow::Result;
use reqwest::multipart;
use serde_json::Value;
use tracing::info;

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
