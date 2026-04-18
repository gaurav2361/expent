use crate::ocr::OcrUpdate;
use ::ocr::OcrService;
use chrono::Utc;
use db::AppError;
use db::entities;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Iden, QueryFilter, Set,
};
use std::sync::Arc;
use upload::UploadClient;

pub async fn create_ocr_job(
    db: &DatabaseConnection,
    user_id: &str,
    r2_key: &str,
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
        status: Set("QUEUED".to_string()),
        r2_key: Set(r2_key.to_string()),
        p_hash: Set(p_hash),
        auto_confirm: Set(auto_confirm),
        wallet_id: Set(wallet_id),
        category_id: Set(category_id),
        transaction_id: Set(None),
        started_at: Set(None),
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

pub async fn update_ocr_job(
    db: &DatabaseConnection,
    job_id: &str,
    status: &str,
    processed_data: Option<serde_json::Value>,
    error: Option<String>,
    transaction_id: Option<String>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
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
    job.updated_at = Set(Utc::now().into());

    job.update(db).await.map_err(AppError::from)
}

pub async fn process_job(
    db: &DatabaseConnection,
    ocr_service: Arc<OcrService>,
    upload_client: &UploadClient,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
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
    let key = job.r2_key.clone();

    // 1. Update status to PROCESSING
    update_ocr_job(
        db,
        &job_id,
        "PROCESSING",
        None,
        None,
        None,
        Some(chrono::Utc::now()),
    )
    .await?;

    let _ = ocr_tx.send(OcrUpdate {
        user_id: user_id.clone(),
        job_id: job_id.clone(),
        status: "PROCESSING".to_string(),
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

        let ocr_json = ocr_service.process_file(&bytes, filename, mime_type).await?;

        let mut processed_ocr: db::ProcessedOcr = serde_json::from_value(ocr_json)?;
        processed_ocr.r2_key = Some(key.clone());

        let mut transaction_id = None;
        let mut final_status = "COMPLETED";

        if job.auto_confirm {
            // Attach wallet and category if provided in the job
            if let Some(w_id) = job.wallet_id {
                match processed_ocr.doc_type.as_str() {
                    "GPAY" => {
                        if let Ok(mut gpay) =
                            serde_json::from_value::<db::GPayExtraction>(processed_ocr.data.0.clone())
                        {
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

            match crate::services::ocr::process_ocr(db, &user_id, processed_ocr.clone()).await {
                Ok(res) => {
                    transaction_id = Some(res.transaction.id);
                }
                Err(e) => {
                    tracing::error!("❌ Auto-confirmation failed for job {}: {}", job_id, e);
                    final_status = "PENDING_REVIEW";
                }
            }
        } else {
            final_status = "PENDING_REVIEW";
        }

        Ok::<(db::ProcessedOcr, String, Option<String>), Box<dyn std::error::Error + Send + Sync>>(
            (processed_ocr, final_status.to_string(), transaction_id),
        )
    }
    .await;

    match process_res {
        Ok((processed, status, tx_id)) => {
            update_ocr_job(
                db,
                &job_id,
                &status,
                Some(serde_json::to_value(processed).unwrap()),
                None,
                tx_id,
                None,
            )
            .await?;

            let _ = ocr_tx.send(OcrUpdate {
                user_id,
                job_id: job_id.clone(),
                status: status.to_string(),
            });
        }
        Err(e) => {
            tracing::error!("❌ OCR Background Job {} failed: {}", job_id, e);
            update_ocr_job(
                db,
                &job_id,
                "FAILED",
                None,
                Some(e.to_string()),
                None,
                None,
            )
            .await?;

            let _ = ocr_tx.send(OcrUpdate {
                user_id,
                job_id: job_id.clone(),
                status: "FAILED".to_string(),
            });
        }
    }

    Ok(())
}
