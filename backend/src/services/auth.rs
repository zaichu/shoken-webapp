use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, EndpointNotSet,
    EndpointSet, RedirectUrl, TokenResponse, TokenUrl,
};
use sqlx::PgPool;
use std::sync::OnceLock;

use crate::config;
use crate::errors::ApiError;
use crate::models::user::{GoogleUserInfo, User};

pub const SESSION_COOKIE_NAME: &str = "session_token";
pub const OAUTH_STATE_COOKIE_NAME: &str = "oauth_state";

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

type GoogleOAuthClient = oauth2::Client<
    oauth2::basic::BasicErrorResponse,
    oauth2::basic::BasicTokenResponse,
    oauth2::basic::BasicTokenIntrospectionResponse,
    oauth2::StandardRevocableToken,
    oauth2::basic::BasicRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

pub fn create_oauth_client(
    client_id: &str,
    client_secret: &str,
) -> Result<GoogleOAuthClient, ApiError> {
    let redirect_url = format!("{}/api/v1/oauth/google/callback", config::backend_url());

    let client = BasicClient::new(ClientId::new(client_id.to_string()))
        .set_client_secret(ClientSecret::new(client_secret.to_string()))
        .set_auth_uri(
            AuthUrl::new(GOOGLE_AUTH_URL.to_string())
                .map_err(|e| ApiError::ApiError(format!("認証URL解析エラー: {}", e)))?,
        )
        .set_token_uri(
            TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
                .map_err(|e| ApiError::ApiError(format!("トークンURL解析エラー: {}", e)))?,
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_url)
                .map_err(|e| ApiError::ApiError(format!("リダイレクトURL解析エラー: {}", e)))?,
        );

    Ok(client)
}

/// トークン交換用の共有HTTPクライアント（起動時に1つ構築し、TCP/TLSコネクションを使い回す）。
///
/// `oauth2::reqwest` は oauth2 クレートが依存する reqwest 0.12 の再エクスポートであり、
/// アプリ側の `reqwest` 0.13（`AppState.client`）とは型が異なるため、専用の static で保持する。
static OAUTH_HTTP_CLIENT: OnceLock<oauth2::reqwest::Client> = OnceLock::new();

/// OAuth用HTTPクライアントを構築する（10秒タイムアウト・リダイレクト3回制限）。
pub fn build_oauth_http_client() -> Result<oauth2::reqwest::Client, ApiError> {
    oauth2::reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(oauth2::reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|e| ApiError::OAuthError(format!("OAuth HTTPクライアント構築エラー: {}", e)))
}

/// 起動時に共有OAuthクライアントを初期化する（`main.rs` の起動処理から呼ぶ）。
pub fn init_oauth_http_client() -> Result<(), ApiError> {
    if OAUTH_HTTP_CLIENT.get().is_none() {
        let client = build_oauth_http_client()?;
        // 並行して初期化が進んでいた場合は先勝ちした側を共有する。
        if OAUTH_HTTP_CLIENT.set(client).is_err() {
            tracing::debug!("OAuth HTTPクライアントは既に初期化されています");
        }
    }
    Ok(())
}

/// 共有OAuthクライアントを取得する。起動時初期化漏れ時はその場で構築する。
fn shared_oauth_http_client() -> Result<&'static oauth2::reqwest::Client, ApiError> {
    if let Some(client) = OAUTH_HTTP_CLIENT.get() {
        return Ok(client);
    }
    init_oauth_http_client()?;
    OAUTH_HTTP_CLIENT.get().ok_or_else(|| {
        ApiError::OAuthError("OAuth HTTPクライアントが初期化されていません".to_string())
    })
}

/// Google OAuth コードをトークンに交換し、ユーザーを upsert してセッショントークンを返す
pub async fn authenticate_with_google_code(
    pool: &PgPool,
    http_client: &reqwest::Client,
    oauth_client: &GoogleOAuthClient,
    code: String,
) -> Result<String, ApiError> {
    let oauth_http_client = shared_oauth_http_client()?;
    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(code))
        .request_async(oauth_http_client)
        .await
        .map_err(|e| {
            tracing::error!("OAuth token exchange error: {}", e);
            ApiError::OAuthError(e.to_string())
        })?;

    let access_token = token_result.access_token().secret().to_string();

    let user_info: GoogleUserInfo = http_client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(format!("ユーザー情報取得エラー: {}", e)))?
        .json()
        .await
        .map_err(|e| ApiError::NetworkError(format!("ユーザー情報解析エラー: {}", e)))?;

    let user = upsert_user(pool, &user_info).await?;
    let session_token = create_session(pool, user.id).await?;

    Ok(session_token)
}

pub async fn select_user_by_session(
    pool: &PgPool,
    session_id: uuid::Uuid,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT u.id, u.google_id, u.email, u.name, u.picture_url, u.created_at, u.updated_at
        FROM users u
        INNER JOIN sessions s ON u.id = s.user_id
        WHERE s.id = $1 AND s.expires_at > NOW()
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
}

pub async fn select_user_id_by_session(
    pool: &PgPool,
    session_id: uuid::Uuid,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    let user_id: Option<(uuid::Uuid,)> = sqlx::query_as(
        r#"
        SELECT user_id FROM sessions
        WHERE id = $1 AND expires_at > NOW()
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(user_id.map(|record| record.0))
}

pub async fn create_session(pool: &PgPool, user_id: uuid::Uuid) -> Result<String, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM sessions WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    let session_id: (uuid::Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO sessions (user_id)
        VALUES ($1)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(session_id.0.to_string())
}

pub async fn upsert_user(pool: &PgPool, user_info: &GoogleUserInfo) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (google_id, email, name, picture_url)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (google_id) DO UPDATE SET
            email = EXCLUDED.email,
            name = EXCLUDED.name,
            picture_url = EXCLUDED.picture_url,
            updated_at = NOW()
        RETURNING id, google_id, email, name, picture_url, created_at, updated_at
        "#,
    )
    .bind(&user_info.sub)
    .bind(&user_info.email)
    .bind(&user_info.name)
    .bind(&user_info.picture)
    .fetch_one(pool)
    .await
}

pub async fn delete_session(pool: &PgPool, session_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// ユーザーを削除（CASCADE により関連データも削除）
pub async fn delete_account(pool: &PgPool, user_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}
#[cfg(test)]
mod tests {
    use {
        super::*,
        crate::test_env::{EnvGuard, ENV_MUTEX},
    };

    #[test]
    fn test_create_oauth_client() {
        assert!(create_oauth_client("client-id", "client-secret").is_ok());
    }

    #[test]
    fn test_oauth_http_client_is_shared() {
        init_oauth_http_client().expect("共有OAuthクライアントの初期化");
        let first = shared_oauth_http_client().expect("共有クライアント取得") as *const _;
        init_oauth_http_client().expect("再初期化は冪等");
        let second = shared_oauth_http_client().expect("共有クライアント再取得") as *const _;
        assert_eq!(
            first, second,
            "OAuth用HTTPクライアントは同一インスタンスを共有すること"
        );
    }

    #[tokio::test]
    async fn test_oauth_redirect_uri_is_v1_path() {
        let _lock = ENV_MUTEX.lock().await;
        let _env = EnvGuard::set("BACKEND_URL", Some("https://shoken-backend.fly.dev"));
        let client = create_oauth_client("client-id", "client-secret").unwrap();
        let (auth_url, _csrf) = client.authorize_url(oauth2::CsrfToken::new_random).url();
        let url_str = auth_url.to_string();
        assert!(
            url_str.contains("redirect_uri=https%3A%2F%2Fshoken-backend.fly.dev%2Fapi%2Fv1%2Foauth%2Fgoogle%2Fcallback"),
            "redirect_uri が /api/v1/oauth/google/callback でない: {}",
            url_str
        );
    }
}
