use image::ImageError;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OcrError {
    #[error("Failed to create models directory: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to download model from {url}: {source}")]
    Download { url: String, source: reqwest::Error },

    #[error("Failed to read model bytes: {0}")]
    ReadBytes(#[from] reqwest::Error),

    #[error("Failed to load model from {path:?}: {reason}")]
    ModelLoad { path: PathBuf, reason: String },

    #[error("Failed to initialize OcrEngine: {0}")]
    EngineInit(String),

    #[error("Failed to load image from memory: {0}")]
    ImageLoad(#[from] ImageError),

    #[error("Failed to create ImageSource: {0}")]
    ImageSource(String),

    #[error("Failed to prepare input: {0}")]
    PrepareInput(String),

    #[error("Failed to run OCR: {0}")]
    OcrRun(String),
}

pub struct OcrService {
    engine: OcrEngine,
}

impl OcrService {
    pub async fn new() -> Result<Self, OcrError> {
        let models_dir = Path::new("models");
        if !models_dir.exists() {
            fs::create_dir_all(models_dir)?;
        }

        let detection_model_path = models_dir.join("text-detection.rten");
        let recognition_model_path = models_dir.join("text-recognition.rten");

        if !detection_model_path.exists() {
            tracing::info!("📥 Downloading text-detection model...");
            let url = "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten";
            Self::download_file(url, &detection_model_path).await?;
            tracing::info!("✅ Text-detection model downloaded.");
        }

        if !recognition_model_path.exists() {
            tracing::info!("📥 Downloading text-recognition model...");
            let url = "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten";
            Self::download_file(url, &recognition_model_path).await?;
            tracing::info!("✅ Text-recognition model downloaded.");
        }

        tracing::info!("🧠 Loading OCR models into memory...");
        let detection_model =
            Model::load_file(&detection_model_path).map_err(|e| OcrError::ModelLoad {
                path: detection_model_path.to_path_buf(),
                reason: e.to_string(),
            })?;
        let recognition_model =
            Model::load_file(&recognition_model_path).map_err(|e| OcrError::ModelLoad {
                path: recognition_model_path.to_path_buf(),
                reason: e.to_string(),
            })?;

        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            ..Default::default()
        })
        .map_err(|e| OcrError::EngineInit(e.to_string()))?;

        tracing::info!("🚀 OCR Engine initialized successfully.");
        Ok(Self { engine })
    }

    async fn download_file(url: &str, path: &Path) -> Result<(), OcrError> {
        let response = reqwest::get(url).await.map_err(|e| OcrError::Download {
            url: url.to_string(),
            source: e,
        })?;
        let bytes = response.bytes().await?;
        fs::write(path, bytes)?;
        Ok(())
    }

    pub fn process_image(&self, image_bytes: &[u8]) -> Result<String, OcrError> {
        tracing::info!(
            "🖼️ Received image for OCR processing ({} bytes)",
            image_bytes.len()
        );

        let img = image::load_from_memory(image_bytes)?.into_rgb8();
        let (width, height) = img.dimensions();
        tracing::info!("📏 Image dimensions: {}x{}", width, height);

        let img_source = ImageSource::from_bytes(img.as_raw(), (width, height))
            .map_err(|e| OcrError::ImageSource(e.to_string()))?;

        tracing::info!("⚙️ Preparing input for OCR engine...");
        let input = self
            .engine
            .prepare_input(img_source)
            .map_err(|e| OcrError::PrepareInput(e.to_string()))?;

        tracing::info!("🔍 Running OCR engine (this may take a few seconds)...");
        let ocr_output = self
            .engine
            .get_text(&input)
            .map_err(|e| OcrError::OcrRun(e.to_string()))?;

        let mut text = String::new();
        let mut line_count = 0;
        for line in ocr_output.lines() {
            text.push_str(line);
            text.push('\n');
            line_count += 1;
        }

        tracing::info!("📝 OCR completed. Extracted {} lines of text.", line_count);
        Ok(text)
    }
}
