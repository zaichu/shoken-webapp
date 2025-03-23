use std::env;

use axum::{http::StatusCode, response::IntoResponse, Json};
use oauth2::{
    basic::BasicErrorResponseType, url::ParseError, RequestTokenError, StandardErrorResponse,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    ValidationError(String),
    #[error("JSON parse error")]
    JsonParseError,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Not found")]
    NotFound,
    #[error("Environment variable error: {0}")]
    EnvVarError(#[from] env::VarError),
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] ParseError),
    #[error("OAuth error: {0}")]
    OAuthError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::JsonParseError => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ApiError::EnvVarError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::UrlParseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ApiError::OAuthError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(serde_json::json!({ "error": error_message }))).into_response()
    }
}

impl<T> From<RequestTokenError<T, StandardErrorResponse<BasicErrorResponseType>>> for ApiError
where
    T: std::error::Error + Send + Sync + 'static,
{
    fn from(err: RequestTokenError<T, StandardErrorResponse<BasicErrorResponseType>>) -> Self {
        ApiError::OAuthError(err.to_string())
    }
}
