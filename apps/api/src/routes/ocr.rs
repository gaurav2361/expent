use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, Sse};
use axum::routing::{get, post};
use axum::Router;
use expent_core::ocr;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use tokio_stream::StreamExt;

use crate::middleware::error::ApiError;
use crate::{AppState, AuthSession};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/process", post(process_image_ocr_handler))
        .route("/status/{job_id}", get(get_ocr_job_status_handler))
        .route("/stream", get(ocr_stream_handler))
        .route("/confirm/{job_id}", post(confirm_ocr_job_handler))
        .route("/resolve/{job_id}", post(resolve_ocr_job_handler))
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
    pub raw_key: Option<String>,
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
        payload.raw_key,
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

    // 2. Trigger processing immediately (the background worker will also pick it up if this fails)
    let state_clone = state.clone();
    let job_id_clone = job_id.clone();
    tokio::spawn(async move {
        if let Err(e) = ocr::process_job(
            &state_clone.core.db,
            state_clone.core.ocr_service.clone(),
            &state_clone.core.upload_client,
            state_clone.ocr_tx.clone(),
            job_id_clone,
        )
        .await
        {
            tracing::error!("❌ Background job processing failed: {}", e);
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

#[derive(Deserialize)]
pub struct ConfirmOcrRequest {
    pub manual_data: Option<db::ProcessedOcr>,
}

pub async fn confirm_ocr_job_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(job_id): Path<String>,
    Json(payload): Json<ConfirmOcrRequest>,
) -> Result<Json<db::OcrTransactionResponse>, ApiError> {
    let result = ocr::confirm_ocr_job(&state.core.db, &session.user.id, &job_id, payload.manual_data).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct ResolveContactRequest {
    pub contact_id: String,
}

pub async fn resolve_ocr_job_handler(
    State(state): State<AppState>,
    session: AuthSession,
    Path(job_id): Path<String>,
    Json(payload): Json<ResolveContactRequest>,
) -> Result<Json<db::OcrTransactionResponse>, ApiError> {
    let result = ocr::resolve_contact_collision(&state.core.db, &session.user.id, &job_id, &payload.contact_id).await?;
    Ok(Json(result))
}
