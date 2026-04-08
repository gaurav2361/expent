use crate::middleware::error::ApiError;
use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
};
use serde::de::DeserializeOwned;
use validator::Validate;

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + Validate + 'static,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|rejection| match rejection {
                JsonRejection::JsonDataError(err) => ApiError::BadRequest(err.to_string()),
                JsonRejection::JsonSyntaxError(err) => ApiError::BadRequest(err.to_string()),
                _ => ApiError::BadRequest("Invalid JSON".to_string()),
            })?;

        value
            .validate()
            .map_err(|err| ApiError::BadRequest(err.to_string()))?;

        Ok(ValidatedJson(value))
    }
}
