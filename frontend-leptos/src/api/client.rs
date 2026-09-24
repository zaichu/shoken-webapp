use gloo_timers::future::TimeoutFuture;
use serde::{de::DeserializeOwned, Serialize};
use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

const DEFAULT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_RETRY_DELAY_MS: u64 = 1_000;
const DEFAULT_RETRY_DELAY_MULTIPLIER: u64 = 2;

const READ_TIMEOUT_MS: u64 = 10_000;
const READ_MAX_RETRIES: u32 = 1;

const AUTH_TIMEOUT_MS: u64 = 5_000;
const AUTH_MAX_RETRIES: u32 = 1;
const AUTH_RETRY_DELAY_MS: u64 = 500;
const AUTH_RETRY_DELAY_MULTIPLIER: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiError {
    Network,
    Timeout,
    Http { status: u16 },
    Parse,
}

impl ApiError {
    pub fn status(&self) -> Option<u16> {
        match self {
            ApiError::Http { status } => Some(*status),
            _ => None,
        }
    }

    pub fn is_unauthorized(&self) -> bool {
        matches!(self, ApiError::Http { status: 401 })
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::Network | ApiError::Timeout => true,
            ApiError::Http { status } => *status >= 500,
            ApiError::Parse => false,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            ApiError::Network => "ネットワーク接続を確認してください".to_string(),
            ApiError::Timeout => {
                "リクエストがタイムアウトしました。もう一度お試しください".to_string()
            }
            ApiError::Parse => "応答の解析に失敗しました".to_string(),
            ApiError::Http { status } => match status {
                400 => "入力内容を確認してください".to_string(),
                401 => "認証が必要です".to_string(),
                403 => "このリソースへのアクセス権限がありません".to_string(),
                404 => "指定されたリソースが見つかりません".to_string(),
                500..=599 => {
                    "サーバーエラーが発生しました。しばらくしてから再度お試しください".to_string()
                }
                _ => format!("エラーが発生しました (ステータス: {status})"),
            },
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for ApiError {}

enum RequestBody {
    None,
    Json(String),
    Form(web_sys::FormData),
}

#[derive(Clone, Debug)]
pub struct ApiClient {
    base_url: String,
    timeout_ms: u64,
    max_retries: u32,
    retry_delay_ms: u64,
    retry_multiplier: u64,
}

impl ApiClient {
    pub fn new(
        base_url: String,
        timeout_ms: u64,
        max_retries: u32,
        retry_delay_ms: u64,
        retry_multiplier: u64,
    ) -> Self {
        ApiClient {
            base_url,
            timeout_ms,
            max_retries,
            retry_delay_ms,
            retry_multiplier,
        }
    }

    pub fn base_url() -> String {
        option_env!("SHOKEN_WEBAPI_URL").unwrap_or("").to_string()
    }

    pub fn default_client() -> Self {
        ApiClient::new(
            Self::base_url(),
            DEFAULT_TIMEOUT_MS,
            DEFAULT_MAX_RETRIES,
            DEFAULT_RETRY_DELAY_MS,
            DEFAULT_RETRY_DELAY_MULTIPLIER,
        )
    }

    pub fn read_client() -> Self {
        ApiClient::new(
            Self::base_url(),
            READ_TIMEOUT_MS,
            READ_MAX_RETRIES,
            DEFAULT_RETRY_DELAY_MS,
            DEFAULT_RETRY_DELAY_MULTIPLIER,
        )
    }

    pub fn auth_client() -> Self {
        ApiClient::new(
            Self::base_url(),
            AUTH_TIMEOUT_MS,
            AUTH_MAX_RETRIES,
            AUTH_RETRY_DELAY_MS,
            AUTH_RETRY_DELAY_MULTIPLIER,
        )
    }

    fn url(&self, path: &str, query: &[(&str, &str)]) -> String {
        let mut url = format!("{}{}", self.base_url, path);
        if !query.is_empty() {
            let encoded: Vec<String> = query
                .iter()
                .map(|(key, value)| {
                    format!(
                        "{}={}",
                        urlencoding::encode(key),
                        urlencoding::encode(value)
                    )
                })
                .collect();
            url.push(if url.contains('?') { '&' } else { '?' });
            url.push_str(&encoded.join("&"));
        }
        url
    }

    async fn send_once(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        body: &RequestBody,
    ) -> Result<String, ApiError> {
        let controller = web_sys::AbortController::new().map_err(|_| ApiError::Network)?;
        let signal = controller.signal();
        let timed_out = Rc::new(Cell::new(false));
        let timer_flag = Rc::clone(&timed_out);
        let timeout_ms = self.timeout_ms;
        leptos::task::spawn_local(async move {
            TimeoutFuture::new(timeout_ms as u32).await;
            timer_flag.set(true);
            controller.abort();
        });

        let url = self.url(path, query);
        let parsed_method: http::Method = method.parse().map_err(|_| ApiError::Network)?;
        let builder = gloo_net::http::RequestBuilder::new(&url)
            .method(parsed_method)
            .credentials(web_sys::RequestCredentials::Include)
            .abort_signal(Some(signal.as_ref()));
        let request = match body {
            RequestBody::None => builder.build().map_err(|_| ApiError::Network)?,
            RequestBody::Json(json) => builder
                .header("Content-Type", "application/json")
                .body(json)
                .map_err(|_| ApiError::Network)?,
            RequestBody::Form(form) => builder.body(form).map_err(|_| ApiError::Network)?,
        };
        let response = request.send().await.map_err(|_| {
            if timed_out.get() {
                ApiError::Timeout
            } else {
                ApiError::Network
            }
        })?;
        if !response.ok() {
            return Err(ApiError::Http {
                status: response.status(),
            });
        }
        response.text().await.map_err(|_| ApiError::Network)
    }

    async fn execute(
        &self,
        method: &str,
        path: &str,
        query: &[(&str, &str)],
        body: RequestBody,
    ) -> Result<String, ApiError> {
        let mut attempt = 0;
        loop {
            match self.send_once(method, path, query, &body).await {
                Ok(text) => return Ok(text),
                Err(error) if attempt < self.max_retries && error.is_retryable() => {
                    let delay = self
                        .retry_delay_ms
                        .saturating_mul(self.retry_multiplier.saturating_pow(attempt));
                    TimeoutFuture::new(delay as u32).await;
                    attempt += 1;
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> Result<T, ApiError> {
        let text = self.execute("GET", path, query, RequestBody::None).await?;
        serde_json::from_str(&text).map_err(|_| ApiError::Parse)
    }

    pub async fn delete_empty(&self, path: &str) -> Result<(), ApiError> {
        self.execute("DELETE", path, &[], RequestBody::None)
            .await
            .map(|_| ())
    }

    pub async fn post_json<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let json = serde_json::to_string(body).map_err(|_| ApiError::Parse)?;
        let text = self
            .execute("POST", path, &[], RequestBody::Json(json))
            .await?;
        serde_json::from_str(&text).map_err(|_| ApiError::Parse)
    }

    pub async fn post_multipart<T: DeserializeOwned>(
        &self,
        path: &str,
        form: &web_sys::FormData,
    ) -> Result<T, ApiError> {
        let text = self
            .execute("POST", path, &[], RequestBody::Form(form.clone()))
            .await?;
        serde_json::from_str(&text).map_err(|_| ApiError::Parse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retryable_errors() {
        assert!(ApiError::Network.is_retryable());
        assert!(ApiError::Timeout.is_retryable());
        assert!(ApiError::Http { status: 500 }.is_retryable());
        assert!(ApiError::Http { status: 503 }.is_retryable());
        assert!(!ApiError::Http { status: 400 }.is_retryable());
        assert!(!ApiError::Http { status: 401 }.is_retryable());
        assert!(!ApiError::Http { status: 403 }.is_retryable());
        assert!(!ApiError::Http { status: 404 }.is_retryable());
        assert!(!ApiError::Parse.is_retryable());
    }

    #[test]
    fn unauthorized_detection() {
        assert!(ApiError::Http { status: 401 }.is_unauthorized());
        assert!(!ApiError::Http { status: 403 }.is_unauthorized());
        assert!(!ApiError::Network.is_unauthorized());
    }

    #[test]
    fn unauthorized_message_matches_react() {
        assert_eq!(
            ApiError::Http { status: 401 }.user_message(),
            "認証が必要です"
        );
    }

    #[test]
    fn client_profiles_match_react_defaults() {
        let default = ApiClient::default_client();
        assert_eq!(default.timeout_ms, 30_000);
        assert_eq!(default.max_retries, 3);
        let read = ApiClient::read_client();
        assert_eq!(read.timeout_ms, 10_000);
        assert_eq!(read.max_retries, 1);
        let auth = ApiClient::auth_client();
        assert_eq!(auth.timeout_ms, 5_000);
        assert_eq!(auth.max_retries, 1);
        assert_eq!(auth.retry_delay_ms, 500);
        assert_eq!(auth.retry_multiplier, 1);
    }

    #[test]
    fn url_building() {
        let client = ApiClient::new(String::new(), 1, 0, 1, 1);
        assert_eq!(client.url("/api/v1/session", &[]), "/api/v1/session");
        assert_eq!(
            client.url("/api/v1/stocks", &[("query", "7974")]),
            "/api/v1/stocks?query=7974"
        );
        let prefixed = ApiClient::new("https://example.test".to_string(), 1, 0, 1, 1);
        assert_eq!(
            prefixed.url("/api/v1/session", &[]),
            "https://example.test/api/v1/session"
        );
    }
}
