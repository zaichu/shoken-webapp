use axum::{http::StatusCode, response::IntoResponse, Json};
use oauth2::{
    basic::BasicErrorResponseType, url::ParseError, RequestTokenError, StandardErrorResponse,
};
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;

/// 本番環境かどうかを判定
/// BACKEND_URL が設定されている、または RUST_ENV=production の場合に true
fn is_production() -> bool {
    env::var("RUST_ENV")
        .map(|v| v == "production")
        .unwrap_or(false)
        || env::var("BACKEND_URL").is_ok()
}

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
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Serde JSON error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_details) = match self {
            ApiError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorDetails {
                    code: "VALIDATION_ERROR".to_string(),
                    message: msg,
                    details: None,
                },
            ),
            ApiError::JsonParseError => (
                StatusCode::BAD_REQUEST,
                ErrorDetails {
                    code: "JSON_PARSE_ERROR".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
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
                // ログにエラー詳細を出力（本番・開発両方）
                // 注: 個人情報やトークンは含まれないことを確認済み
                tracing::error!("Database error [{}]: {}", code, e);
                (
                    if code == "NOT_FOUND" {
                        StatusCode::NOT_FOUND
                    } else {
                        StatusCode::INTERNAL_SERVER_ERROR
                    },
                    ErrorDetails {
                        code: code.to_string(),
                        message: message.to_string(),
                        // 本番環境では詳細を含めない
                        details: if is_production() {
                            None
                        } else {
                            Some(e.to_string())
                        },
                    },
                )
            }
            ApiError::NotFound => (
                StatusCode::NOT_FOUND,
                ErrorDetails {
                    code: "NOT_FOUND".to_string(),
                    message: self.to_string(),
                    details: None,
                },
            ),
            ApiError::EnvVarError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetails {
                    code: "CONFIGURATION_ERROR".to_string(),
                    message: "Configuration error".to_string(),
                    details: None,
                },
            ),
            ApiError::UrlParseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetails {
                    code: "URL_PARSE_ERROR".to_string(),
                    message: "Invalid URL".to_string(),
                    details: None,
                },
            ),
            ApiError::OAuthError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorDetails {
                    code: "OAUTH_ERROR".to_string(),
                    message: "Authentication error".to_string(),
                    details: None,
                },
            ),
            ApiError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                ErrorDetails {
                    code: "UNAUTHORIZED".to_string(),
                    message: msg,
                    details: None,
                },
            ),
            ApiError::NetworkError(msg) => (
                StatusCode::BAD_GATEWAY,
                ErrorDetails {
                    code: "NETWORK_ERROR".to_string(),
                    message: msg,
                    details: None,
                },
            ),
            ApiError::ApiError(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorDetails {
                    code: "API_ERROR".to_string(),
                    message: msg,
                    details: None,
                },
            ),
            ApiError::SerdeJsonError(ref e) => {
                tracing::error!("JSON processing error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorDetails {
                        code: "JSON_ERROR".to_string(),
                        message: "JSON processing error".to_string(),
                        // 本番環境では詳細を含めない
                        details: if is_production() {
                            None
                        } else {
                            Some(e.to_string())
                        },
                    },
                )
            }
        };

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
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
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
