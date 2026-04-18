use crate::OcrService;
use crate::OcrUpdate;
use chrono::Utc;
use db::AppError;
use db::entities;
use rand::Rng;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use upload::UploadClient;

pub const CURRENT_SCHEMA_VERSION: i32 = 1;

pub async fn create_ocr_job(
    db: &DatabaseConnection,
    user_id: &str,
    trace_id: Option<String>,
    r2_key: &str,
    raw_key: Option<String>,
    p_hash: Option<String>,
    auto_confirm: bool,
    wallet_id: Option<String>,
    category_id: Option<String>,
) -> Result<entities::ocr_jobs::Model, AppError> {
    // Check for duplicate pHash within the last hour to prevent re-processing
    if let Some(ref hash) = p_hash {
        let one_hour_ago = Utc::now() - chrono::Duration::hours(1);
        let existing = entities::ocr_jobs::Entity::find()
            .filter(entities::ocr_jobs::Column::UserId.eq(user_id))
            .filter(entities::ocr_jobs::Column::PHash.eq(hash))
            .filter(entities::ocr_jobs::Column::CreatedAt.gt(one_hour_ago))
            .one(db)
            .await?;

        if let Some(job) = existing {
            return Ok(job);
        }
    }

    let job = entities::ocr_jobs::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        trace_id: Set(trace_id),
        status: Set("QUEUED".to_string()),
        r2_key: Set(r2_key.to_string()),
        raw_key: Set(raw_key),
        p_hash: Set(p_hash),
        auto_confirm: Set(auto_confirm),
        wallet_id: Set(wallet_id),
        category_id: Set(category_id),
        transaction_id: Set(None),
        started_at: Set(None),
        scheduled_at: Set(None),
        retry_count: Set(0),
        is_high_res: Set(false),
        schema_version: Set(CURRENT_SCHEMA_VERSION),
        last_error: Set(None),
        resolution_candidates: Set(None),
        processed_data: Set(None),
        error: Set(None),
        created_at: Set(Utc::now().into()),
        updated_at: Set(Utc::now().into()),
    };

    job.insert(db).await.map_err(AppError::from)
}

pub async fn get_ocr_job(
    db: &DatabaseConnection,
    user_id: &str,
    job_id: &str,
) -> Result<entities::ocr_jobs::Model, AppError> {
    entities::ocr_jobs::Entity::find_by_id(job_id.to_string())
        .filter(entities::ocr_jobs::Column::UserId.eq(user_id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("OCR Job not found"))
}

pub async fn list_pending_ocr_jobs(
    db: &DatabaseConnection,
    user_id: &str,
) -> Result<Vec<entities::ocr_jobs::Model>, AppError> {
    entities::ocr_jobs::Entity::find()
        .filter(entities::ocr_jobs::Column::UserId.eq(user_id))
        .filter(entities::ocr_jobs::Column::Status.is_in(vec![
            "PENDING_REVIEW".to_string(),
            "CONTACT_COLLISION".to_string(),
        ]))
        .all(db)
        .await
        .map_err(AppError::from)
}

pub async fn update_ocr_job(
    db: &DatabaseConnection,
    job_id: &str,
    status: &str,
    processed_data: Option<serde_json::Value>,
    error: Option<String>,
    transaction_id: Option<String>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    scheduled_at: Option<chrono::DateTime<chrono::Utc>>,
    retry_count: Option<i32>,
    is_high_res: Option<bool>,
    schema_version: Option<i32>,
    resolution_candidates: Option<serde_json::Value>,
    last_error: Option<String>,
) -> Result<entities::ocr_jobs::Model, AppError> {
    let mut job: entities::ocr_jobs::ActiveModel =
        entities::ocr_jobs::Entity::find_by_id(job_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("OCR Job not found"))?
            .into();

    job.status = Set(status.to_string());
    if let Some(data) = processed_data {
        job.processed_data = Set(Some(data));
    }
    if error.is_some() {
        job.error = Set(error);
    }
    if let Some(t_id) = transaction_id {
        job.transaction_id = Set(Some(t_id));
    }
    if let Some(s_at) = started_at {
        job.started_at = Set(Some(s_at.into()));
    }
    if let Some(sch_at) = scheduled_at {
        job.scheduled_at = Set(Some(sch_at.into()));
    }
    if let Some(r_count) = retry_count {
        job.retry_count = Set(r_count);
    }
    if let Some(high_res) = is_high_res {
        job.is_high_res = Set(high_res);
    }
    if let Some(version) = schema_version {
        job.schema_version = Set(version);
    }
    if let Some(candidates) = resolution_candidates {
        job.resolution_candidates = Set(Some(candidates));
    }
    if let Some(l_err) = last_error {
        job.last_error = Set(Some(l_err));
    }
    job.updated_at = Set(Utc::now().into());

    job.update(db).await.map_err(AppError::from)
}

/// Helper function to bridge with expent_core for processing transactions.
/// This will be provided by a trait or callback to keep the ocr crate decoupled.
pub trait OcrProcessor: Send + Sync {
    fn process_ocr(
        &self,
        db: &DatabaseConnection,
        user_id: &str,
        processed: db::ProcessedOcr,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<db::OcrTransactionResponse, AppError>> + Send>,
    >;
}

pub async fn process_job(
    db: &DatabaseConnection,
    ocr_service: Arc<OcrService>,
    upload_client: &UploadClient,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
    processor: Arc<dyn OcrProcessor>,
    job_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let job = entities::ocr_jobs::Entity::find_by_id(job_id.clone())
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("OCR Job not found"))?;

    if job.status != "QUEUED" && job.status != "PENDING" {
        return Ok(());
    }

    let user_id = job.user_id.clone();
    let trace_id = job.trace_id.clone();

    // Use raw_key if it's already a high-res attempt or if we want to try high-res
    let key = if job.is_high_res && job.raw_key.is_some() {
        job.raw_key.clone().unwrap()
    } else {
        job.r2_key.clone()
    };

    // 1. Update status to PROCESSING
    update_ocr_job(
        db,
        &job_id,
        "PROCESSING",
        None,
        None,
        None,
        Some(chrono::Utc::now()),
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    let _ = ocr_tx.send(OcrUpdate {
        user_id: user_id.clone(),
        job_id: job_id.clone(),
        status: "PROCESSING".to_string(),
        trace_id: trace_id.clone(),
    });

    let process_res = async {
        let bytes = upload_client.get_file(&key).await?;

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

        let ocr_json = ocr_service
            .process_file(&bytes, filename, mime_type)
            .await?;

        let mut processed_ocr: db::ProcessedOcr = serde_json::from_value(ocr_json.clone())?;
        processed_ocr.r2_key = Some(job.r2_key.clone());
        processed_ocr.is_high_res = job.is_high_res;

        // Extract confidence from the inner data
        let confidence_score = match processed_ocr.doc_type.as_str() {
            "GPAY" => {
                let gpay: db::GPayExtraction =
                    serde_json::from_value(processed_ocr.data.0.clone())?;
                gpay.confidence_score
            }
            "GENERIC" => {
                let generic: db::OcrResult = serde_json::from_value(processed_ocr.data.0.clone())?;
                generic.confidence_score
            }
            _ => 1.0,
        };

        // 3. Progressive Quality Fallback
        if confidence_score < 0.8 && !job.is_high_res && job.raw_key.is_some() {
            tracing::info!(
                "⚠️ Low confidence ({}) for job {}, triggering high-res retry",
                confidence_score,
                job_id
            );
            return Ok::<
                (
                    db::ProcessedOcr,
                    String,
                    Option<String>,
                    Option<serde_json::Value>,
                ),
                Box<dyn std::error::Error + Send + Sync>,
            >((processed_ocr, "RETRY_HIGH_RES".to_string(), None, None));
        }

        let mut transaction_id = None;
        let mut final_status = "COMPLETED";
        let mut collision_data = None;

        if job.auto_confirm {
            if let Some(w_id) = job.wallet_id {
                match processed_ocr.doc_type.as_str() {
                    "GPAY" => {
                        if let Ok(mut gpay) = serde_json::from_value::<db::GPayExtraction>(
                            processed_ocr.data.0.clone(),
                        ) {
                            gpay.wallet_id = Some(w_id);
                            if let Some(c_id) = job.category_id.clone() {
                                gpay.category_id = Some(c_id);
                            }
                            processed_ocr.data.0 = serde_json::to_value(gpay).unwrap();
                        }
                    }
                    "GENERIC" => {
                        if let Ok(mut generic) =
                            serde_json::from_value::<db::OcrResult>(processed_ocr.data.0.clone())
                        {
                            generic.wallet_id = Some(w_id);
                            if let Some(c_id) = job.category_id.clone() {
                                generic.category_id = Some(c_id);
                            }
                            processed_ocr.data.0 = serde_json::to_value(generic).unwrap();
                        }
                    }
                    _ => {}
                }
            }

            match processor
                .process_ocr(db, &user_id, processed_ocr.clone())
                .await
            {
                Ok(res) => {
                    transaction_id = Some(res.transaction.id);
                }
                Err(e) => {
                    if let db::AppError::ContactCollision(candidates) = e {
                        tracing::warn!(
                            "⚠️ Contact collision for job {}, needs manual review",
                            job_id
                        );
                        final_status = "CONTACT_COLLISION";
                        collision_data = Some(candidates);
                    } else {
                        tracing::error!("❌ Auto-confirmation failed for job {}: {}", job_id, e);
                        final_status = "PENDING_REVIEW";
                    }
                }
            }
        } else {
            final_status = "PENDING_REVIEW";
        }

        Ok::<
            (
                db::ProcessedOcr,
                String,
                Option<String>,
                Option<serde_json::Value>,
            ),
            Box<dyn std::error::Error + Send + Sync>,
        >((
            processed_ocr,
            final_status.to_string(),
            transaction_id,
            collision_data,
        ))
    }
    .await;

    match process_res {
        Ok((processed, status, tx_id, candidates)) => {
            if status == "RETRY_HIGH_RES" {
                update_ocr_job(
                    db,
                    &job_id,
                    "QUEUED",
                    None,
                    None,
                    None,
                    None,
                    Some(chrono::Utc::now().into()),
                    None,
                    Some(true),
                    None,
                    None,
                    None,
                )
                .await?;

                let _ = ocr_tx.send(OcrUpdate {
                    user_id: user_id.clone(),
                    job_id: job_id.clone(),
                    status: "QUEUED".to_string(),
                    trace_id: trace_id.clone(),
                });
            } else {
                update_ocr_job(
                    db,
                    &job_id,
                    &status,
                    Some(serde_json::to_value(processed).unwrap()),
                    None,
                    tx_id,
                    None,
                    None,
                    None,
                    None,
                    None,
                    candidates,
                    None,
                )
                .await?;

                let _ = ocr_tx.send(OcrUpdate {
                    user_id,
                    job_id: job_id.clone(),
                    status: status.to_string(),
                    trace_id,
                });
            }
        }
        Err(e) => {
            tracing::error!("❌ OCR Background Job {} failed: {}", job_id, e);

            let new_retry_count = job.retry_count + 1;
            let max_retries = 5;

            if new_retry_count < max_retries {
                let base_delay = 10;
                let backoff_secs = base_delay * (2_i64.pow(new_retry_count as u32));
                let jitter = rand::thread_rng().gen_range(0..5);
                let next_run =
                    chrono::Utc::now() + chrono::Duration::seconds(backoff_secs + jitter);

                update_ocr_job(
                    db,
                    &job_id,
                    "QUEUED",
                    None,
                    None,
                    None,
                    None,
                    Some(next_run.into()),
                    Some(new_retry_count),
                    None,
                    None,
                    None,
                    Some(e.to_string()),
                )
                .await?;

                let _ = ocr_tx.send(OcrUpdate {
                    user_id: user_id.clone(),
                    job_id: job_id.clone(),
                    status: "QUEUED".to_string(),
                    trace_id,
                });
            } else {
                update_ocr_job(
                    db,
                    &job_id,
                    "DEAD_LETTER",
                    None,
                    Some(e.to_string()),
                    None,
                    None,
                    None,
                    Some(new_retry_count),
                    None,
                    None,
                    None,
                    Some(e.to_string()),
                )
                .await?;

                let _ = ocr_tx.send(OcrUpdate {
                    user_id,
                    job_id: job_id.clone(),
                    status: "DEAD_LETTER".to_string(),
                    trace_id,
                });
            }
        }
    }

    Ok(())
}

async fn log_ocr_edits(
    db: &DatabaseConnection,
    user_id: &str,
    job_id: &str,
    original: &serde_json::Value,
    corrected: &serde_json::Value,
) -> Result<(), AppError> {
    if let (Some(orig_obj), Some(corr_obj)) = (original.as_object(), corrected.as_object()) {
        for (key, corr_val) in corr_obj {
            let orig_val = orig_obj.get(key).unwrap_or(&serde_json::Value::Null);
            if orig_val != corr_val && !corr_val.is_object() && !corr_val.is_array() {
                let edit = entities::ocr_job_edits::ActiveModel {
                    id: Set(uuid::Uuid::now_v7().to_string()),
                    ocr_job_id: Set(job_id.to_string()),
                    user_id: Set(user_id.to_string()),
                    field_name: Set(key.clone()),
                    original_value: Set(Some(orig_val.to_string())),
                    corrected_value: Set(Some(corr_val.to_string())),
                    created_at: Set(Utc::now().into()),
                };
                edit.insert(db).await?;
            }
        }
    }
    Ok(())
}

pub async fn confirm_ocr_job(
    db: &DatabaseConnection,
    processor: Arc<dyn OcrProcessor>,
    user_id: &str,
    job_id: &str,
    manual_data: Option<db::ProcessedOcr>,
) -> Result<db::OcrTransactionResponse, AppError> {
    let job = get_ocr_job(db, user_id, job_id).await?;

    let ocr_data = if let Some(data) = manual_data {
        if let Some(orig_data) = job.processed_data {
            let corrected_json = serde_json::to_value(&data).unwrap();
            log_ocr_edits(db, user_id, job_id, &orig_data, &corrected_json).await?;
        }
        data
    } else {
        serde_json::from_value(
            job.processed_data
                .ok_or_else(|| AppError::validation("Job has no processed data"))?,
        )
        .map_err(|e| AppError::Ocr(format!("Failed to parse job data: {}", e)))?
    };

    let result = processor.process_ocr(db, user_id, ocr_data).await?;

    update_ocr_job(
        db,
        job_id,
        "COMPLETED",
        None,
        None,
        Some(result.transaction.id.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    Ok(result)
}

pub async fn resolve_contact_collision(
    db: &DatabaseConnection,
    processor: Arc<dyn OcrProcessor>,
    user_id: &str,
    job_id: &str,
    contact_id: &str,
) -> Result<db::OcrTransactionResponse, AppError> {
    let job = get_ocr_job(db, user_id, job_id).await?;

    let mut ocr_data: db::ProcessedOcr = serde_json::from_value(
        job.processed_data
            .ok_or_else(|| AppError::validation("Job has no processed data"))?,
    )
    .map_err(|e| AppError::Ocr(format!("Failed to parse job data: {}", e)))?;

    match ocr_data.doc_type.as_str() {
        "GPAY" => {
            let mut gpay: db::GPayExtraction = serde_json::from_value(ocr_data.data.0.clone())
                .map_err(|e| AppError::Ocr(format!("Failed to parse GPAY data: {}", e)))?;
            gpay.contact_id = Some(contact_id.to_string());
            ocr_data.data.0 = serde_json::to_value(gpay).unwrap();
        }
        "GENERIC" => {
            let mut generic: db::OcrResult = serde_json::from_value(ocr_data.data.0.clone())
                .map_err(|e| AppError::Ocr(format!("Failed to parse GENERIC data: {}", e)))?;
            generic.contact_id = Some(contact_id.to_string());
            ocr_data.data.0 = serde_json::to_value(generic).unwrap();
        }
        _ => return Err(AppError::validation("Unknown document type")),
    }

    confirm_ocr_job(db, processor, user_id, job_id, Some(ocr_data)).await
}
