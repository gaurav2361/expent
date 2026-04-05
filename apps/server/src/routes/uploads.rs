use axum::extract::{Multipart, State};
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/presigned", post(get_presigned_url_handler))
        .route("/", post(direct_upload_handler))
}

#[derive(Deserialize)]
pub struct PresignedUrlRequest {
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
}

#[derive(Serialize)]
pub struct PresignedUrlResponse {
    pub url: String,
    pub key: String,
}

pub async fn get_presigned_url_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<PresignedUrlRequest>,
) -> Result<Json<PresignedUrlResponse>, ApiError> {
    let (url, key) = state
        .upload_client
        .get_presigned_url(
            &session.user.id,
            &payload.file_name,
            &payload.content_type,
            Duration::from_secs(3600),
        )
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(Json(PresignedUrlResponse { url, key }))
}

pub async fn direct_upload_handler(
    State(state): State<AppState>,
    session: AuthSession,
    mut multipart: Multipart,
) -> Result<Json<PresignedUrlResponse>, ApiError> {
    tracing::debug!("📁 Received upload request for user: {}", session.user.id);
    let mut file_data = None;
    let mut file_name = String::new();
    let mut content_type = String::new();

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("❌ Multipart next_field error: {:?}", e);
        ApiError::BadRequest(e.to_string())
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
                ApiError::Internal(e.to_string())
            })?);
            break;
        }
    }

    let data = file_data.ok_or_else(|| {
        tracing::warn!("⚠️ No file data found in multipart request");
        ApiError::BadRequest("No file uploaded".to_string())
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
            ApiError::Internal(e.to_string())
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
