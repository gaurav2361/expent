use chrono::Utc;
use db::AppError;
use db::entities;
use sea_orm::{DatabaseConnection, Set, Iden, EntityTrait, ActiveModelTrait, QueryFilter, ColumnTrait};

pub async fn create_ocr_job(
    db: &DatabaseConnection,
    user_id: &str,
    r2_key: &str,
) -> Result<entities::ocr_jobs::Model, AppError> {
    let job = entities::ocr_jobs::ActiveModel {
        id: Set(uuid::Uuid::now_v7().to_string()),
        user_id: Set(user_id.to_string()),
        status: Set("PENDING".to_string()),
        r2_key: Set(r2_key.to_string()),
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
) -> Result<entities::ocr_jobs::Model, AppError> {
    let mut job: entities::ocr_jobs::ActiveModel =
        entities::ocr_jobs::Entity::find_by_id(job_id.to_string())
            .one(db)
            .await?
            .ok_or_else(|| AppError::not_found("OCR Job not found"))?
            .into();

    job.status = Set(status.to_string());
    job.processed_data = Set(processed_data);
    job.error = Set(error);
    job.updated_at = Set(Utc::now().into());

    job.update(db).await.map_err(AppError::from)
}
