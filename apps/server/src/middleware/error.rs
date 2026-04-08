use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Internal Server Error: {0}")]
    Internal(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Database error: {0}")]
    App(#[from] db::AppError),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::App(app_err) => match app_err {
                db::AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                db::AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
                db::AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
                db::AppError::Ocr(msg) => (StatusCode::BAD_REQUEST, msg),
                _ => {
                    tracing::error!("Internal App Error: {:?}", app_err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "An internal error occurred".into(),
                    )
                }
            },
        };

        let body = Json(json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

use sea_orm::DbErr;

impl From<DbErr> for ApiError {
    fn from(err: DbErr) -> Self {
        ApiError::App(db::AppError::Db(err))
    }
}
