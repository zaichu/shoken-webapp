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
    Http {
        status: u16,
        // 400 ではバリデーション理由が入る。取れない場合は既定文に倒す
        server_message: Option<String>,
    },
    Parse,
}

impl ApiError {
    #[cfg(test)]
    pub fn http(status: u16) -> Self {
        ApiError::Http {
            status,
            server_message: None,
        }
    }

    pub fn is_unauthorized(&self) -> bool {
        matches!(self, ApiError::Http { status: 401, .. })
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::Network | ApiError::Timeout => true,
            ApiError::Http { status, .. } => *status >= 500,
            ApiError::Parse => false,
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            ApiError::Network => "ネットワーク接続を確認してください".to_string(),
            ApiError::Timeout => {
                "リクエストがタイムアウトしました。もう一度お試しください".to_string()
            }
            ApiError::Parse => {
                "銘柄情報の取得に失敗しました。時間をおいて再度お試しください。".to_string()
            }
            ApiError::Http { status, .. } => match status {
                400 => "入力内容を確認してください".to_string(),
                401 => "ログインが必要です".to_string(),
                403 => "このリソースへのアクセス権限がありません".to_string(),
                404 => "指定されたリソースが見つかりません".to_string(),
                500 | 502..=504 => {
                    "サーバーエラーが発生しました。しばらくしてから再度お試しください".to_string()
                }
                _ => format!("エラーが発生しました (ステータス: {status})"),
            },
        }
    }

    // エラーバッジ用の生文言。user_message() の案内文とは別系統で、HTTP 既定文に直す
    pub fn message(&self) -> String {
        match self {
            ApiError::Network => "ネットワークエラーが発生しました".to_string(),
            ApiError::Timeout => "リクエストがタイムアウトしました".to_string(),
            ApiError::Parse => "応答の解析に失敗しました".to_string(),
            ApiError::Http {
                status,
                server_message,
            } => match status {
                400 => server_message
                    .clone()
                    .unwrap_or_else(|| "リクエストが不正です".to_string()),
                401 => "認証が必要です".to_string(),
                403 => "アクセスが拒否されました".to_string(),
                404 => "リソースが見つかりません".to_string(),
                500 | 502..=504 => "サーバーエラーが発生しました".to_string(),
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

    // API 接続先の正本は index.html の shoken-api-origin meta(session-probe.js と共通)。
    // 本番は prepare-vercel-dist.mjs が vercel.json の CSP connect-src の値を埋め、
    // ローカル開発では空のまま = 同一オリジン(trunk のプロキシが /api を受ける)
    #[cfg(target_arch = "wasm32")]
    pub fn base_url() -> String {
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| {
                document
                    .query_selector("meta[name=\"shoken-api-origin\"]")
                    .ok()
                    .flatten()
            })
            .and_then(|meta| meta.get_attribute("content"))
            .unwrap_or_default()
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    pub fn with_max_retries(&self, max_retries: u32) -> Self {
        ApiClient {
            max_retries,
            ..self.clone()
        }
    }

    pub fn with_timeout_ms(&self, timeout_ms: u64) -> Self {
        ApiClient {
            timeout_ms,
            ..self.clone()
        }
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
            let status = response.status();
            let server_message = response
                .text()
                .await
                .ok()
                .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
                .and_then(|body| {
                    body.get("error")?
                        .get("message")?
                        .as_str()
                        .map(str::to_string)
                });
            return Err(ApiError::Http {
                status,
                server_message,
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
mod tests;
