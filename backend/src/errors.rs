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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use oauth2::url::ParseError;
    use sqlx::Error as SqlxError;
    use std::env::VarError;

    #[test]
    fn test_validation_error_into_response() {
        let error = ApiError::ValidationError("必須フィールドが不足しています".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_json_parse_error_into_response() {
        let error = ApiError::JsonParseError;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_database_error_into_response() {
        let sql_error = SqlxError::RowNotFound;
        let error = ApiError::DatabaseError(sql_error);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_not_found_into_response() {
        let error = ApiError::NotFound;
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_env_var_error_into_response() {
        let env_error = VarError::NotPresent;
        let error = ApiError::EnvVarError(env_error);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_url_parse_error_into_response() {
        let parse_error = ParseError::EmptyHost;
        let error = ApiError::UrlParseError(parse_error);
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_oauth_error_into_response() {
        let error = ApiError::OAuthError("認証エラー".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
