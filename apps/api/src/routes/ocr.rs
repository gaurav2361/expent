use axum::Router;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use expent_core::ocr;
use serde::{Deserialize, Serialize};

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/process", post(process_image_ocr_handler))
        .route("/status/{job_id}", get(get_ocr_job_status_handler))
}

#[derive(Deserialize)]
pub struct ProcessImageOcrRequest {
    pub key: String,
}

#[derive(Serialize)]
pub struct OcrJobResponse {
    pub job_id: String,
}

pub async fn process_image_ocr_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(payload): Json<ProcessImageOcrRequest>,
) -> Result<(StatusCode, Json<OcrJobResponse>), ApiError> {
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

    // 1. Create a record in ocr_jobs table (PENDING)
    let job = ocr::create_ocr_job(&state.core.db, &session.user.id, &payload.key).await?;
    let job_id = job.id.clone();

    // 2. Spawn a background task
    let state_clone = state.clone();
    let key = payload.key.clone();
    let job_id_clone = job_id.clone();

    tokio::spawn(async move {
        let process_res = async {
            let bytes = state_clone.core.upload_client.get_file(&key).await?;

            // Determine filename and mime type from the key
            let filename = key.split("/").last().unwrap_or("upload");
            let mime_type = if filename.ends_with(".pdf") {
                "application/pdf"
            } else if filename.ends_with(".csv") {
                "text/csv"
            } else {
                "image/png" // Default for screenshots
            };

            let ocr_json = state_clone
                .core
                .ocr_service
                .process_file(&bytes, filename, mime_type)
                .await?;

            let mut processed_ocr: expent_core::ProcessedOcr = serde_json::from_value(ocr_json)?;
            processed_ocr.r2_key = Some(key);

            Ok::<expent_core::ProcessedOcr, Box<dyn std::error::Error + Send + Sync>>(processed_ocr)
        }
        .await;

        match process_res {
            Ok(processed) => {
                let _ = ocr::update_ocr_job(
                    &state_clone.core.db,
                    &job_id_clone,
                    "COMPLETED",
                    Some(serde_json::to_value(processed).unwrap()),
                    None,
                )
                .await;
            }
            Err(e) => {
                tracing::error!("❌ OCR Background Job failed: {}", e);
                let _ = ocr::update_ocr_job(
                    &state_clone.core.db,
                    &job_id_clone,
                    "FAILED",
                    None,
                    Some(e.to_string()),
                )
                .await;
            }
        }
    });

    Ok((StatusCode::ACCEPTED, Json(OcrJobResponse { job_id })))
}

pub async fn get_ocr_job_status_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(job_id): Path<String>,
) -> Result<Json<db::entities::ocr_jobs::Model>, ApiError> {
    let job = ocr::get_ocr_job(&state.core.db, &session.user.id, &job_id).await?;
    Ok(Json(job))
}
