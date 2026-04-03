use aws_sdk_s3::Client as S3Client;
use aws_sdk_s3::presigning::PresigningConfig;
use bytes::Bytes;
use image::ImageFormat;
use std::path::Path;
use std::time::Duration;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum UploadError {
    #[error("Failed to process image: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Unknown file type")]
    UnknownFileType,
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("S3 error: {0}")]
    S3Error(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileCategory {
    Pdf,
    Image,
    Csv,
    Unknown,
}

#[derive(Debug, serde::Serialize)]
pub struct ProcessedFile {
    pub id: Uuid,
    pub original_name: Option<String>,
    pub category: FileCategory,
    pub content_type: String,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
    pub key: String,
}

#[derive(Clone)]
pub struct UploadClient {
    s3_client: S3Client,
    bucket_name: String,
}

impl UploadClient {
    pub fn new(s3_client: S3Client, bucket_name: String) -> Self {
        Self {
            s3_client,
            bucket_name,
        }
    }

    pub async fn get_presigned_url(
        &self,
        user_id: &str,
        file_name: &str,
        content_type: &str,
        expires_in: Duration,
    ) -> Result<(String, String), UploadError> {
        let sanitized_name = Path::new(file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");
        let key = format!("{}/{}-{}", user_id, Uuid::new_v4(), sanitized_name);

        let presigning_config = PresigningConfig::expires_in(expires_in)
            .map_err(|e| UploadError::Internal(e.to_string()))?;

        let presigned_request = self
            .s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| UploadError::S3Error(format!("{:#?}", e)))?;

        Ok((presigned_request.uri().to_string(), key))
    }

    pub async fn upload_direct(
        &self,
        user_id: &str,
        data: Bytes,
        original_name: Option<String>,
        content_type: Option<String>,
        normalize_images: bool,
    ) -> Result<ProcessedFile, UploadError> {
        let processed = UploadProcessor::process(
            data,
            original_name.clone(),
            content_type.clone(),
            normalize_images,
        )?;

        let sanitized_name = original_name
            .as_deref()
            .and_then(|name| Path::new(name).file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");
        let key = format!("{}/{}-{}", user_id, processed.id, sanitized_name);

        self.s3_client
            .put_object()
            .bucket(&self.bucket_name)
            .key(&key)
            .content_type(&processed.content_type)
            .body(processed.data.clone().into())
            .send()
            .await
            .map_err(|e| UploadError::S3Error(format!("{:#?}", e)))?;

        Ok(ProcessedFile {
            id: processed.id,
            original_name: processed.original_name,
            category: processed.category,
            content_type: processed.content_type,
            data: processed.data.to_vec(),
            key,
        })
    }

    pub async fn get_file(&self, key: &str) -> Result<Bytes, UploadError> {
        let response = self
            .s3_client
            .get_object()
            .bucket(&self.bucket_name)
            .key(key)
            .send()
            .await
            .map_err(|e| UploadError::S3Error(format!("{:#?}", e)))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| UploadError::Internal(e.to_string()))?
            .into_bytes();

        Ok(data)
    }
}

pub struct UploadProcessor;

impl UploadProcessor {
    pub fn process(
        data: Bytes,
        original_name: Option<String>,
        content_type: Option<String>,
        normalize_images: bool,
    ) -> Result<RawProcessedFile, UploadError> {
        let category =
            Self::determine_category(&data, original_name.as_deref(), content_type.as_deref());

        let id = Uuid::new_v4();

        // Perform category-specific processing
        match category {
            FileCategory::Image => {
                let (final_data, final_content_type) = if normalize_images {
                    let png_data = Self::convert_to_png(&data)?;
                    (png_data, "image/png".to_string())
                } else {
                    (
                        data,
                        content_type.unwrap_or_else(|| "image/png".to_string()),
                    )
                };

                // Validate it's a valid image (already validated if converted, but let's be sure)
                if !normalize_images {
                    let _img = image::load_from_memory(&final_data)?;
                }

                Ok(RawProcessedFile {
                    id,
                    original_name,
                    category,
                    content_type: final_content_type,
                    data: final_data,
                })
            }
            FileCategory::Pdf => Ok(RawProcessedFile {
                id,
                original_name,
                category,
                content_type: "application/pdf".to_string(),
                data,
            }),
            FileCategory::Csv => Ok(RawProcessedFile {
                id,
                original_name,
                category,
                content_type: "text/csv".to_string(),
                data,
            }),
            FileCategory::Unknown => Ok(RawProcessedFile {
                id,
                original_name,
                category,
                content_type: content_type
                    .unwrap_or_else(|| "application/octet-stream".to_string()),
                data,
            }),
        }
    }

    fn determine_category(
        data: &[u8],
        filename: Option<&str>,
        content_type: Option<&str>,
    ) -> FileCategory {
        // 1. Try by content-type
        if let Some(ct) = content_type {
            if ct == "application/pdf" {
                return FileCategory::Pdf;
            }
            if ct.starts_with("image/") {
                return FileCategory::Image;
            }
            if ct == "text/csv" || ct == "application/csv" {
                return FileCategory::Csv;
            }
        }

        // 2. Try by extension
        if let Some(name) = filename {
            let path = Path::new(name);
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                match ext.to_lowercase().as_str() {
                    "pdf" => return FileCategory::Pdf,
                    "csv" => return FileCategory::Csv,
                    "jpg" | "jpeg" | "png" | "gif" | "webp" | "heic" | "heif" => {
                        return FileCategory::Image;
                    }
                    _ => {}
                }
            }
        }

        // 3. Try by magic bytes
        if let Some(kind) = infer::get(data) {
            match kind.mime_type() {
                "application/pdf" => return FileCategory::Pdf,
                "text/csv" | "application/csv" => return FileCategory::Csv,
                m if m.starts_with("image/") => return FileCategory::Image,
                _ => {}
            }
        }

        FileCategory::Unknown
    }

    /// Convert image to a standard format (e.g. PNG)
    pub fn convert_to_png(data: &[u8]) -> Result<Bytes, UploadError> {
        let img = image::load_from_memory(data)?;
        let mut buffer = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageFormat::Png)?;
        Ok(Bytes::from(buffer.into_inner()))
    }
}

pub struct RawProcessedFile {
    pub id: Uuid,
    pub original_name: Option<String>,
    pub category: FileCategory,
    pub content_type: String,
    pub data: Bytes,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_category_pdf() {
        let data = b"%PDF-1.4";
        assert_eq!(
            UploadProcessor::determine_category(data, None, None),
            FileCategory::Pdf
        );
    }

    #[test]
    fn test_determine_category_image() {
        // PNG magic bytes
        let data = b"\x89PNG\r\n\x1a\n";
        assert_eq!(
            UploadProcessor::determine_category(data, None, None),
            FileCategory::Image
        );
    }

    #[test]
    fn test_determine_category_csv() {
        assert_eq!(
            UploadProcessor::determine_category(b"col1,col2\nval1,val2", Some("test.csv"), None),
            FileCategory::Csv
        );
        assert_eq!(
            UploadProcessor::determine_category(b"col1,col2\nval1,val2", None, Some("text/csv")),
            FileCategory::Csv
        );
    }

    #[test]
    fn test_determine_category_unknown() {
        assert_eq!(
            UploadProcessor::determine_category(b"unknown data", None, None),
            FileCategory::Unknown
        );
    }

    #[test]
    fn test_s3_key_path_traversal_mitigated() {
        let user_id = "user123";
        let file_name = "../dangerous.txt";
        let id = Uuid::new_v4();

        // Simulate the fixed key generation
        let sanitized_name = Path::new(file_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unnamed");
        let key = format!("{}/{}-{}", user_id, id, sanitized_name);

        // The key should NO LONGER contain "../"
        assert!(!key.contains("../"));
        assert_eq!(key, format!("user123/{}-dangerous.txt", id));
    }
}
