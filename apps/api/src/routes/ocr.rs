use axum::Router;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use expent_core::ocr;
use serde::{Deserialize, Serialize};

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

use crate::OcrUpdate;
use axum::response::sse::{Event, Sse};
use futures::stream::Stream;
use std::convert::Infallible;
use tokio_stream::StreamExt;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/process", post(process_image_ocr_handler))
        .route("/status/{job_id}", get(get_ocr_job_status_handler))
        .route("/stream", get(ocr_stream_handler))
}

pub async fn ocr_stream_handler(
    State(state): State<AppState>,
    session: AuthSession,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.ocr_tx.subscribe();
    let user_id = session.user.id.clone();

    let stream = async_stream::stream! {
        loop {
            match rx.recv().await {
                Ok(update) => {
                    // Only send updates for the current user
                    if update.user_id == user_id {
                        if let Ok(event) = Event::default().json_data(update) {
                            yield Ok(event);
                        }
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
                Err(_) => break,
            }
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

#[derive(Deserialize)]
pub struct ProcessImageOcrRequest {
    pub key: String,
    pub p_hash: Option<String>,
    pub auto_confirm: Option<bool>,
    pub wallet_id: Option<String>,
    pub category_id: Option<String>,
}

#[derive(Serialize)]
pub struct OcrJobResponse {
    pub job_id: String,
    pub status: String,
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

    // 1. Create a record in ocr_jobs table (QUEUED)
    let auto_confirm = payload.auto_confirm.unwrap_or(false);
    let job = ocr::create_ocr_job(
        &state.core.db,
        &session.user.id,
        &payload.key,
        payload.p_hash,
        auto_confirm,
        payload.wallet_id.clone(),
        payload.category_id.clone(),
    )
    .await?;
    let job_id = job.id.clone();

    // If the job is already COMPLETED (from pHash match), return early
    if job.status == "COMPLETED" {
        return Ok((
            StatusCode::OK,
            Json(OcrJobResponse {
                job_id,
                status: job.status,
            }),
        ));
    }

    // 2. Spawn a background task
    let state_clone = state.clone();
    let key = payload.key.clone();
    let job_id_clone = job_id.clone();
    let user_id_clone = session.user.id.clone();

    tokio::spawn(async move {
        // Set status to PROCESSING
        let _ = ocr::update_ocr_job(
            &state_clone.core.db,
            &job_id_clone,
            "PROCESSING",
            None,
            None,
            None,
            Some(chrono::Utc::now()),
        )
        .await;

        let _ = state_clone.ocr_tx.send(OcrUpdate {
            user_id: user_id_clone.clone(),
            job_id: job_id_clone.clone(),
            status: "PROCESSING".to_string(),
        });

        let process_res = async {
            let bytes = state_clone.core.upload_client.get_file(&key).await?;

            // Determine filename and mime type from the key
            let filename = key.split("/").last().unwrap_or("upload");
            let mime_type = if filename.ends_with(".pdf") {
                "application/pdf"
            } else if filename.ends_with(".csv") {
                "text/csv"
            } else if filename.ends_with(".webp") {
                "image/webp"
            } else {
                "image/png"
            };

            let ocr_json = state_clone
                .core
                .ocr_service
                .process_file(&bytes, filename, mime_type)
                .await?;

            let mut processed_ocr: expent_core::ProcessedOcr = serde_json::from_value(ocr_json)?;
            processed_ocr.r2_key = Some(key.clone());

            // If auto_confirm is enabled, automatically create the transaction
            let mut transaction_id = None;
            let mut final_status = "COMPLETED";

            if auto_confirm {
                // Attach wallet and category if provided in the job
                if let Some(w_id) = payload.wallet_id {
                    match processed_ocr.doc_type.as_str() {
                        "GPAY" => {
                            if let Ok(mut gpay) = serde_json::from_value::<db::GPayExtraction>(
                                processed_ocr.data.0.clone(),
                            ) {
                                gpay.wallet_id = Some(w_id);
                                if let Some(c_id) = payload.category_id.clone() {
                                    gpay.category_id = Some(c_id);
                                }
                                processed_ocr.data.0 = serde_json::to_value(gpay).unwrap();
                            }
                        }
                        "GENERIC" => {
                            if let Ok(mut generic) = serde_json::from_value::<db::OcrResult>(
                                processed_ocr.data.0.clone(),
                            ) {
                                generic.wallet_id = Some(w_id);
                                if let Some(c_id) = payload.category_id.clone() {
                                    generic.category_id = Some(c_id);
                                }
                                processed_ocr.data.0 = serde_json::to_value(generic).unwrap();
                            }
                        }
                        _ => {}
                    }
                }

                match ocr::process_ocr(&state_clone.core.db, &user_id_clone, processed_ocr.clone())
                    .await
                {
                    Ok(res) => {
                        transaction_id = Some(res.transaction.id);
                    }
                    Err(e) => {
                        tracing::error!("❌ Auto-confirmation failed: {}", e);
                        final_status = "PENDING_REVIEW"; // Fallback to manual review if auto-confirm fails
                    }
                }
            } else {
                final_status = "PENDING_REVIEW";
            }

            Ok::<
                (expent_core::ProcessedOcr, String, Option<String>),
                Box<dyn std::error::Error + Send + Sync>,
            >((processed_ocr, final_status.to_string(), transaction_id))
        }
        .await;

        match process_res {
            Ok((processed, status, tx_id)) => {
                let _ = ocr::update_ocr_job(
                    &state_clone.core.db,
                    &job_id_clone,
                    &status,
                    Some(serde_json::to_value(processed).unwrap()),
                    None,
                    tx_id,
                    None,
                )
                .await;

                let _ = state_clone.ocr_tx.send(OcrUpdate {
                    user_id: user_id_clone.clone(),
                    job_id: job_id_clone.clone(),
                    status: status.to_string(),
                });
            }
            Err(e) => {
                tracing::error!("❌ OCR Background Job failed: {}", e);
                let _ = ocr::update_ocr_job(
                    &state_clone.core.db,
                    &job_id_clone,
                    "FAILED",
                    None,
                    Some(e.to_string()),
                    None,
                    None,
                )
                .await;

                let _ = state_clone.ocr_tx.send(OcrUpdate {
                    user_id: user_id_clone.clone(),
                    job_id: job_id_clone.clone(),
                    status: "FAILED".to_string(),
                });
            }
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(OcrJobResponse {
            job_id,
            status: "QUEUED".to_string(),
        }),
    ))
}

pub async fn get_ocr_job_status_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(job_id): Path<String>,
) -> Result<Json<db::entities::ocr_jobs::Model>, ApiError> {
    let job = ocr::get_ocr_job(&state.core.db, &session.user.id, &job_id).await?;
    Ok(Json(job))
}
