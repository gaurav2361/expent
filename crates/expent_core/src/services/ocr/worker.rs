use crate::ocr;
use chrono::Utc;
use db::entities;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::time::Duration;

pub async fn start_recovery_worker(db: DatabaseConnection) {
    let mut interval = tokio::time::interval(Duration::from_secs(300)); // Every 5 minutes
    loop {
        interval.tick().await;
        if let Err(e) = recover_stale_jobs(&db).await {
            tracing::error!("❌ Recovery worker failed: {}", e);
        }
    }
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

        // In a real system with a message queue, we'd re-publish to the queue here.
        // For now, we depend on the next time the user polls or we might need a
        // background consumer that processes the QUEUED jobs.
    }

    Ok(())
}
