use axum::Router;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::routing::post;
use serde::Deserialize;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new().route("/process", post(process_image_ocr_handler))
}

#[derive(Deserialize)]
pub struct ProcessImageOcrRequest {
    pub key: String,
}

pub async fn process_image_ocr_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<ProcessImageOcrRequest>,
) -> Result<Json<db::ProcessedOcr>, ApiError> {
    // Security check: Ensure the key starts with the user ID to prevent IDOR
    let user_id_prefix = format!("{}/", session.user.id);
    if !payload.key.starts_with(&user_id_prefix) {
        tracing::warn!(
            "🔒 Potential IDOR attempt by user {} for key {}",
            session.user.id,
            payload.key
        );
        return Err(ApiError::Unauthorized(
            "You do not have permission to access this file".to_string(),
        ));
    }

    let bytes = state
        .upload_client
        .get_file(&payload.key)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    // Determine filename and mime type from the key or payload
    let filename = payload.key.split("/").last().unwrap_or("upload");
    let mime_type = if filename.ends_with(".pdf") {
        "application/pdf"
    } else if filename.ends_with(".csv") {
        "text/csv"
    } else {
        "image/png" // Default for screenshots
    };

    let ocr_json = state
        .ocr_service
        .process_file(&bytes, filename, mime_type)
        .await
        .map_err(|e| {
            // Check if this is a reqwest error with a 429 status code
            if let Some(re) = e.downcast_ref::<reqwest::Error>() {
                if let Some(status) = re.status() {
                    if status == StatusCode::TOO_MANY_REQUESTS {
                        return ApiError::Internal(
                            "Gemini API quota exceeded. Please try again later.".to_string(),
                        );
                    }
                }
            }
            ApiError::Internal(e.to_string())
        })?;

    let mut processed_ocr: db::ProcessedOcr = serde_json::from_value(ocr_json)
        .map_err(|e| ApiError::Internal(format!("Failed to parse OCR response: {}", e)))?;

    processed_ocr.r2_key = Some(payload.key);

    Ok(Json(processed_ocr))
}
