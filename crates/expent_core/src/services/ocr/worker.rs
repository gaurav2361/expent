use crate::ocr::{self, OcrUpdate};
use ::ocr::OcrService;
use chrono::Utc;
use db::entities;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::time::Duration;
use std::sync::Arc;
use upload::UploadClient;

pub async fn start_recovery_worker(db: DatabaseConnection) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
    loop {
        interval.tick().await;
        if let Err(e) = recover_stale_jobs(&db).await {
            tracing::error!("❌ Recovery worker failed: {}", e);
        }
    }
}

pub async fn start_processor_worker(
    db: DatabaseConnection,
    ocr_service: Arc<OcrService>,
    upload_client: UploadClient,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(10)); // Poll every 10 seconds
    loop {
        interval.tick().await;
        if let Err(e) = process_queued_jobs(&db, ocr_service.clone(), &upload_client, ocr_tx.clone()).await {
            tracing::error!("❌ Processor worker failed: {}", e);
        }
    }
}

async fn process_queued_jobs(
    db: &DatabaseConnection,
    ocr_service: Arc<OcrService>,
    upload_client: &UploadClient,
    ocr_tx: tokio::sync::broadcast::Sender<OcrUpdate>,
) -> Result<(), anyhow::Error> {
    // Find jobs in QUEUED status
    let queued_jobs = entities::ocr_jobs::Entity::find()
        .filter(entities::ocr_jobs::Column::Status.eq("QUEUED"))
        .all(db)
        .await?;

    for job in queued_jobs {
        let job_id = job.id.clone();
        tracing::info!("👷 Background worker picking up job: {}", job_id);
        
        // Process each job. We can do this concurrently or sequentially.
        // For simplicity and to respect rate limits, let's do it sequentially for now 
        // within this worker loop, or spawn if we want more throughput.
        let db_clone = db.clone();
        let ocr_service_clone = ocr_service.clone();
        let upload_client_clone = upload_client.clone();
        let ocr_tx_clone = ocr_tx.clone();
        
        tokio::spawn(async move {
            if let Err(e) = ocr::process_job(
                &db_clone,
                ocr_service_clone,
                &upload_client_clone,
                ocr_tx_clone,
                job_id
            ).await {
                tracing::error!("❌ Background job processing failed: {}", e);
            }
        });
    }

    Ok(())
}

async fn recover_stale_jobs(db: &DatabaseConnection) -> Result<(), anyhow::Error> {
    let ten_minutes_ago = Utc::now() - chrono::Duration::minutes(10);

    // Find jobs that have been PROCESSING for more than 10 minutes
    let stale_jobs = entities::ocr_jobs::Entity::find()
        .filter(entities::ocr_jobs::Column::Status.eq("PROCESSING"))
        .filter(entities::ocr_jobs::Column::StartedAt.lt(ten_minutes_ago))
        .all(db)
        .await?;

    for job in stale_jobs {
        tracing::warn!("🔄 Re-queuing stale OCR job: {}", job.id);

        let mut active_job: entities::ocr_jobs::ActiveModel = job.into();
        active_job.status = Set("QUEUED".to_string());
        active_job.started_at = Set(None);
        active_job.update(db).await?;
    }

    Ok(())
}
