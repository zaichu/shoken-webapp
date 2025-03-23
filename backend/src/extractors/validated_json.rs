use crate::errors::ApiError;
use axum::{extract::FromRequest, http::Request, Json};
use serde::de::DeserializeOwned;
use validator::Validate;

#[derive(Debug)]
pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    Json<T>: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_| ApiError::JsonParseError)?;

        value.validate().map_err(|rejection| {
            ApiError::ValidationError(format!("{}", rejection).replace('\n', ", "))
        })?;

        Ok(ValidatedJson(value))
    }
}
