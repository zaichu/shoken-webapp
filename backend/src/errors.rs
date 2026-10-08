use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use oauth2::url::ParseError;
use std::sync::OnceLock;
use thiserror::Error;

use crate::config::RuntimeEnv;

pub use shared::error::{ErrorDetails, ErrorResponse};

/// CSV アップロードの形式・読み込みエラー
#[derive(Error, Debug)]
pub enum CsvError {
    #[error("CSVが空です")]
    Empty,
    #[error("CSVヘッダーの読み込みに失敗しました: {0}")]
    HeaderRead(String),
    #[error("CSV行の読み込みに失敗しました: {0}")]
    RowRead(String),
    #[error("マルチパートの読み込みに失敗しました: {0}")]
    MultipartRead(String),
    #[error("CSVファイル（.csv）のみアップロードできます")]
    InvalidFileType,
    #[error("ファイルの読み込みに失敗しました: {0}")]
    FileRead(String),
    #[error("fileフィールドが見つかりません")]
    MissingFile,
}

/// 上流サービス(J-Quants / Google OAuth)との通信エラー
#[derive(Error, Debug)]
pub enum UpstreamError {
    #[error("上流サービスへの接続に失敗しました: {0}")]
    Transport(#[source] reqwest::Error),
    #[error("上流サービスがステータス {status} を返しました: {body}")]
    Http { status: u16, body: String },
    #[error("上流サービスのレート制限に達しました")]
    RateLimited,
    #[error("上流サービスの応答を解析できませんでした: {0}")]
    Decode(#[source] serde_json::Error),
    /// 認証サービス呼び出しの失敗。詳細を露出しないため呼び出し名だけ持つ
    #[error("{0}")]
    OAuth(&'static str),
}

impl UpstreamError {
    fn status(&self) -> StatusCode {
        match self {
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::BAD_GATEWAY,
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::RateLimited => "RATE_LIMIT_EXCEEDED",
            _ => "UPSTREAM_ERROR",
        }
    }
}

/// 設定の不備
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("{0}")]
    Missing(&'static str),
    #[error("URL parse error: {0}")]
    UrlParse(#[from] ParseError),
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Validation error: {0}")]
    Validation(String),
    /// serde の失敗位置(フィールドの path)を含むメッセージ
    #[error("JSON parse error: {0}")]
    JsonParse(String),
    #[error("{0}")]
    Csv(#[from] CsvError),
    #[error("Not found")]
    NotFound,
    #[error("{0}")]
    Unauthorized(&'static str),
    #[error("OAuth error: {0}")]
    OAuth(String),
    #[error("{0}")]
    Upstream(#[from] UpstreamError),
    #[error("Database error: {0}")]
    Database(#[from] crate::db::DbError),
    #[error("{0}")]
    Config(#[from] ConfigError),
    #[error("Internal error: {0}")]
    Internal(&'static str),
}

impl ApiError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::Validation(_) | Self::JsonParse(_) | Self::Csv(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::OAuth(_) | Self::Config(_) | Self::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::Upstream(e) => e.status(),
            Self::Database(e) => {
                if matches!(e, crate::db::DbError::RowNotFound) {
                    StatusCode::NOT_FOUND
                } else if e.is_unique_violation() {
                    StatusCode::CONFLICT
                } else if e.is_foreign_key_violation() {
                    StatusCode::BAD_REQUEST
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        }
    }

    /// 応答の error.code
    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::JsonParse(_) => "JSON_PARSE_ERROR",
            Self::Csv(_) => "CSV_ERROR",
            Self::NotFound => "NOT_FOUND",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::OAuth(_) => "OAUTH_ERROR",
            Self::Upstream(e) => e.code(),
            Self::Database(e) => {
                if matches!(e, crate::db::DbError::RowNotFound) {
                    "NOT_FOUND"
                } else if e.is_unique_violation() {
                    "DUPLICATE_ENTRY"
                } else if e.is_foreign_key_violation() {
                    "FOREIGN_KEY_VIOLATION"
                } else {
                    "DATABASE_ERROR"
                }
            }
            Self::Config(_) => "CONFIGURATION_ERROR",
            Self::Internal(_) => "INTERNAL_ERROR",
        }
    }

    /// クライアントへ返すメッセージ（内部情報は出さない）
    fn response_message(&self) -> String {
        match self {
            Self::Validation(msg) => msg.clone(),
            Self::JsonParse(_) | Self::NotFound | Self::Unauthorized(_) => self.to_string(),
            Self::Csv(e) => e.to_string(),
            Self::OAuth(_) => "Authentication error".to_string(),
            Self::Upstream(e) => match e {
                UpstreamError::RateLimited => {
                    "上流サービスのレート制限に達しました。しばらくしてから再試行してください"
                        .to_string()
                }
                _ => "外部サービスとの通信に失敗しました".to_string(),
            },
            Self::Database(e) => {
                if matches!(e, crate::db::DbError::RowNotFound) {
                    "Resource not found".to_string()
                } else if e.is_unique_violation() {
                    "Duplicate entry".to_string()
                } else if e.is_foreign_key_violation() {
                    "Foreign key constraint violation".to_string()
                } else {
                    "Database error occurred".to_string()
                }
            }
            Self::Config(_) => "Configuration error".to_string(),
            Self::Internal(_) => "Internal server error".to_string(),
        }
    }

    /// 非本番の応答 details にだけ載せる内部情報
    fn debug_detail(&self) -> Option<String> {
        match self {
            Self::Database(e) => Some(e.to_string()),
            Self::Config(e) => Some(e.to_string()),
            Self::Upstream(e) => Some(e.to_string()),
            Self::Internal(msg) => Some((*msg).to_string()),
            _ => None,
        }
    }
}

/// 起動時に一度だけ設定する実行環境。未設定（テスト等）のままなら
/// Production 扱いにして内部詳細を露出しない（fail-safe）
static RUNTIME_ENV: OnceLock<RuntimeEnv> = OnceLock::new();

pub fn init_runtime_env(env: RuntimeEnv) {
    let _ = RUNTIME_ENV.set(env);
}

fn is_production_env() -> bool {
    RUNTIME_ENV
        .get()
        .copied()
        .unwrap_or_default()
        .is_production()
}

/// `ErrorDetails` を生成する唯一のコンストラクタ
///
/// 本番環境では内部エラー詳細を露出させないため、`debug_detail` があっても
/// `details` は `None` になる（ログ・機密情報ルール対応）。
pub fn error_details_with_debug(
    code: &str,
    message: String,
    debug_detail: Option<String>,
) -> ErrorDetails {
    ErrorDetails {
        code: code.to_string(),
        message,
        details: if is_production_env() {
            None
        } else {
            debug_detail
        },
    }
}

/// `details` なしの定型エラーレスポンスを直接生成する（middleware 用）
pub fn simple_error_response(status: StatusCode, code: &str, message: String) -> Response {
    let error_response = ErrorResponse {
        error: error_details_with_debug(code, message, None),
    };
    (status, Json(error_response)).into_response()
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match &self {
            Self::Database(e) => {
                if is_production_env() {
                    tracing::error!("Database error [{}]", self.code());
                } else {
                    tracing::error!("Database error [{}]: {}", self.code(), e);
                }
            }
            Self::Config(e) => tracing::error!("Config error: {e}"),
            Self::Internal(msg) => tracing::error!("Internal error: {msg}"),
            Self::Upstream(e) => {
                if is_production_env() {
                    match e {
                        UpstreamError::Http { status, .. } => {
                            tracing::error!("Upstream error: status {status}")
                        }
                        _ => tracing::error!("Upstream error [{}]", e.code()),
                    }
                } else {
                    tracing::error!("Upstream error: {e}");
                }
            }
            _ => {}
        }
        let status = self.status();
        let details =
            error_details_with_debug(self.code(), self.response_message(), self.debug_detail());
        (status, Json(ErrorResponse { error: details })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::db::DbError, crate::db::SqlState, axum::http::StatusCode};

    fn check(error: ApiError, expected_status: StatusCode, expected_code: &str) {
        assert_eq!(error.status(), expected_status);
        assert_eq!(error.code(), expected_code);
        assert_eq!(error.into_response().status(), expected_status);
    }

    #[test]
    fn test_all_error_status_codes() {
        let serde_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        for (error, status, code) in [
            (
                ApiError::Validation("必須フィールドが不足しています".to_string()),
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
            ),
            (
                ApiError::JsonParse("invalid json".to_string()),
                StatusCode::BAD_REQUEST,
                "JSON_PARSE_ERROR",
            ),
            (
                ApiError::Csv(CsvError::Empty),
                StatusCode::BAD_REQUEST,
                "CSV_ERROR",
            ),
            (
                ApiError::Database(DbError::RowNotFound),
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
            ),
            (
                ApiError::Database(DbError::Other("test_column".to_string())),
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
            ),
            (ApiError::NotFound, StatusCode::NOT_FOUND, "NOT_FOUND"),
            (
                ApiError::Unauthorized("認証が必要です"),
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
            ),
            (
                ApiError::OAuth("認証エラー".to_string()),
                StatusCode::INTERNAL_SERVER_ERROR,
                "OAUTH_ERROR",
            ),
            (
                ApiError::Upstream(UpstreamError::RateLimited),
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
            ),
            (
                ApiError::Upstream(UpstreamError::Http {
                    status: 500,
                    body: "upstream failed".to_string(),
                }),
                StatusCode::BAD_GATEWAY,
                "UPSTREAM_ERROR",
            ),
            (
                ApiError::Upstream(UpstreamError::Decode(serde_err)),
                StatusCode::BAD_GATEWAY,
                "UPSTREAM_ERROR",
            ),
            (
                ApiError::Config(ConfigError::Missing("TEST_VAR")),
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIGURATION_ERROR",
            ),
            (
                ApiError::Config(ConfigError::UrlParse(ParseError::EmptyHost)),
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIGURATION_ERROR",
            ),
            (
                ApiError::Internal("内部失敗"),
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
            ),
        ] {
            check(error, status, code);
        }
    }

    #[test]
    fn test_upstream_response_messages() {
        assert_eq!(
            ApiError::Upstream(UpstreamError::RateLimited).response_message(),
            "上流サービスのレート制限に達しました。しばらくしてから再試行してください"
        );
        for e in [
            UpstreamError::Http {
                status: 500,
                body: "raw provider detail".to_string(),
            },
            UpstreamError::OAuth("Googleトークン交換エラー"),
            UpstreamError::Decode(serde_json::from_str::<serde_json::Value>("bad").unwrap_err()),
        ] {
            assert_eq!(
                ApiError::Upstream(e).response_message(),
                "外部サービスとの通信に失敗しました"
            );
        }
    }

    #[test]
    fn test_error_details_with_debug() {
        let details = error_details_with_debug(
            "TEST_CODE",
            "test message".to_string(),
            Some("internal detail".to_string()),
        );
        // テスト環境では RUNTIME_ENV 未初期化 → Production 扱いで details は隠れる
        assert_eq!(
            (
                details.code.as_str(),
                details.message.as_str(),
                details.details
            ),
            ("TEST_CODE", "test message", None)
        );
    }

    #[test]
    fn test_database_error_kind_maps_to_status_and_code() {
        for (db_error, expected) in [
            (
                DbError::State {
                    state: SqlState::UNIQUE_VIOLATION,
                    message: "db error".to_string(),
                },
                (StatusCode::CONFLICT, "DUPLICATE_ENTRY"),
            ),
            (
                DbError::State {
                    state: SqlState::FOREIGN_KEY_VIOLATION,
                    message: "db error".to_string(),
                },
                (StatusCode::BAD_REQUEST, "FOREIGN_KEY_VIOLATION"),
            ),
            (
                DbError::Other("db error".to_string()),
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            ),
        ] {
            let error = ApiError::Database(db_error);
            assert_eq!((error.status(), error.code()), (expected.0, expected.1));
        }
    }
}
