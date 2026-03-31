use anyhow::{Context, Result};
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;
use std::fs;
use std::path::Path;

pub struct OcrService {
    engine: OcrEngine,
}

impl OcrService {
    pub async fn new() -> Result<Self> {
        let models_dir = Path::new("models");
        if !models_dir.exists() {
            fs::create_dir_all(models_dir).context("Failed to create models directory")?;
        }

        let detection_model_path = models_dir.join("text-detection.rten");
        let recognition_model_path = models_dir.join("text-recognition.rten");

        if !detection_model_path.exists() {
            tracing::info!("📥 Downloading text-detection model...");
            Self::download_file(
                "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten",
                &detection_model_path,
            )
            .await?;
            tracing::info!("✅ Text-detection model downloaded.");
        }

        if !recognition_model_path.exists() {
            tracing::info!("📥 Downloading text-recognition model...");
            Self::download_file(
                "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten",
                &recognition_model_path,
            )
            .await?;
            tracing::info!("✅ Text-recognition model downloaded.");
        }

        tracing::info!("🧠 Loading OCR models into memory...");
        let detection_model = Model::load_file(&detection_model_path)
            .with_context(|| format!("Failed to load detection model from {:?}", detection_model_path))?;
        let recognition_model = Model::load_file(&recognition_model_path)
            .with_context(|| format!("Failed to load recognition model from {:?}", recognition_model_path))?;

        let engine = OcrEngine::new(OcrEngineParams {
            detection_model: Some(detection_model),
            recognition_model: Some(recognition_model),
            ..Default::default()
        })
        .map_err(|e| anyhow::anyhow!("Failed to initialize OcrEngine: {}", e))?;

        tracing::info!("🚀 OCR Engine initialized successfully.");
        Ok(Self { engine })
    }

    async fn download_file(url: &str, path: &Path) -> Result<()> {
        let response = reqwest::get(url).await.context("Failed to download model")?;
        let bytes = response.bytes().await.context("Failed to read model bytes")?;
        fs::write(path, bytes).context("Failed to save model to disk")?;
        Ok(())
    }

    pub fn process_image(&self, image_bytes: &[u8]) -> Result<String> {
        tracing::info!("🖼️ Received image for OCR processing ({} bytes)", image_bytes.len());
        
        let img = image::load_from_memory(image_bytes)
            .context("Failed to load image from memory")?
            .into_rgb8();
        let (width, height) = img.dimensions();
        tracing::info!("📏 Image dimensions: {}x{}", width, height);
        
        let img_source = ImageSource::from_bytes(img.as_raw(), (width, height))
            .map_err(|e| anyhow::anyhow!("Failed to create ImageSource: {}", e))?;
        
        tracing::info!("⚙️ Preparing input for OCR engine...");
        let input = self.engine.prepare_input(img_source)
            .map_err(|e| anyhow::anyhow!("Failed to prepare input: {}", e))?;
        
        tracing::info!("🔍 Running OCR engine (this may take a few seconds)...");
        let ocr_output = self.engine.get_text(&input)
            .map_err(|e| anyhow::anyhow!("Failed to run OCR: {}", e))?;
        
        let mut text = String::new();
        let mut line_count = 0;
        for line in ocr_output.lines() {
            text.push_str(&line.to_string());
            text.push('\n');
            line_count += 1;
        }
        
        tracing::info!("📝 OCR completed. Extracted {} lines of text.", line_count);
        Ok(text)
    }
}
