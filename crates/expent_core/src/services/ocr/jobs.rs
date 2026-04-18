use chrono::Utc;
use db::AppError;
use db::entities;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, Iden, QueryFilter, Set,
};

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
