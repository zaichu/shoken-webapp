use axum::{http::StatusCode, response::IntoResponse, Json};
use oauth2::{
    basic::BasicErrorResponseType, url::ParseError, RequestTokenError, StandardErrorResponse,
};
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use utoipa::ToSchema;

use crate::config::is_production_env;

#[derive(Error, Debug)]
#[allow(clippy::enum_variant_names)]
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
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),
    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

// --- ヘルパー関数 ---

/// details なしのシンプルなエラーレスポンスパーツを生成
fn simple_error(status: StatusCode, code: &str, message: String) -> (StatusCode, ErrorDetails) {
    (
        status,
        ErrorDetails {
            code: code.to_string(),
            message,
            details: None,
        },
    )
}

/// ApiError を HTTP ステータスと ErrorDetails に変換
fn into_http(err: ApiError) -> (StatusCode, ErrorDetails) {
    match err {
        ApiError::ValidationError(msg) => {
            simple_error(StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg)
        }
        ApiError::JsonParseError => {
            simple_error(StatusCode::BAD_REQUEST, "JSON_PARSE_ERROR", err.to_string())
        }
        ApiError::DatabaseError(ref e) => {
            let (code, message) = match e {
                sqlx::Error::RowNotFound => ("NOT_FOUND", "Resource not found"),
                sqlx::Error::Database(db_err) => {
                    if db_err.is_unique_violation() {
                        ("DUPLICATE_ENTRY", "Duplicate entry")
                    } else if db_err.is_foreign_key_violation() {
                        ("FOREIGN_KEY_VIOLATION", "Foreign key constraint violation")
                    } else {
                        ("DATABASE_ERROR", "Database error occurred")
                    }
                }
                _ => ("DATABASE_ERROR", "Database error occurred"),
            };
            if is_production_env() {
                tracing::error!("Database error [{}]", code);
            } else {
                tracing::error!("Database error [{}]: {}", code, e);
            }
            (
                if code == "NOT_FOUND" {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                },
                ErrorDetails {
                    code: code.to_string(),
                    message: message.to_string(),
                    details: if is_production_env() {
                        None
                    } else {
                        Some(e.to_string())
                    },
                },
            )
        }
        ApiError::NotFound => simple_error(StatusCode::NOT_FOUND, "NOT_FOUND", err.to_string()),
        ApiError::EnvVarError(_) => simple_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CONFIGURATION_ERROR",
            "Configuration error".to_string(),
        ),
        ApiError::UrlParseError(_) => simple_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "URL_PARSE_ERROR",
            "Invalid URL".to_string(),
        ),
        ApiError::OAuthError(_) => simple_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "OAUTH_ERROR",
            "Authentication error".to_string(),
        ),
        ApiError::Unauthorized(msg) => simple_error(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg),
        ApiError::NetworkError(msg) => simple_error(StatusCode::BAD_GATEWAY, "NETWORK_ERROR", msg),
        ApiError::ApiError(msg) => simple_error(StatusCode::BAD_REQUEST, "API_ERROR", msg),
        ApiError::RateLimitError(msg) => {
            simple_error(StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED", msg)
        }
        ApiError::SerdeJsonError(ref e) => {
            tracing::error!("JSON processing error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetails {
                    code: "JSON_ERROR".to_string(),
                    message: "JSON processing error".to_string(),
                    details: if is_production_env() {
                        None
                    } else {
                        Some(e.to_string())
                    },
                },
            )
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_details) = into_http(self);
        (
            status,
            Json(ErrorResponse {
                error: error_details,
            }),
        )
            .into_response()
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

    fn check_status(error: ApiError, expected: StatusCode) {
        assert_eq!(error.into_response().status(), expected);
    }

    #[test]
    fn test_all_error_status_codes() {
        check_status(
            ApiError::ValidationError("必須フィールドが不足しています".to_string()),
            StatusCode::BAD_REQUEST,
        );
        check_status(ApiError::JsonParseError, StatusCode::BAD_REQUEST);
        check_status(
            ApiError::DatabaseError(SqlxError::RowNotFound),
            StatusCode::NOT_FOUND,
        );
        check_status(
            ApiError::DatabaseError(SqlxError::ColumnNotFound("test_column".to_string())),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
        check_status(ApiError::NotFound, StatusCode::NOT_FOUND);
        check_status(
            ApiError::EnvVarError(VarError::NotPresent),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
        check_status(
            ApiError::RateLimitError("Rate limit exceeded".to_string()),
            StatusCode::TOO_MANY_REQUESTS,
        );
        check_status(
            ApiError::UrlParseError(ParseError::EmptyHost),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
        check_status(
            ApiError::OAuthError("認証エラー".to_string()),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
        let serde_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        check_status(
            ApiError::Unauthorized("認証が必要です".to_string()),
            StatusCode::UNAUTHORIZED,
        );
        check_status(
            ApiError::NetworkError("接続エラー".to_string()),
            StatusCode::BAD_GATEWAY,
        );
        check_status(
            ApiError::ApiError("API エラー".to_string()),
            StatusCode::BAD_REQUEST,
        );
        check_status(
            ApiError::SerdeJsonError(serde_err),
            StatusCode::INTERNAL_SERVER_ERROR,
        );
    }

    #[test]
    fn test_simple_error_helper() {
        let (status, details) = simple_error(
            StatusCode::BAD_REQUEST,
            "TEST_CODE",
            "test message".to_string(),
        );
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(details.code, "TEST_CODE");
        assert_eq!(details.message, "test message");
        assert!(details.details.is_none());
    }
}
