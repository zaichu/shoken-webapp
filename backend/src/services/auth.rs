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

    let session_token = upsert_user_and_rotate_session(pool, &user_info).await?;

    Ok(session_token)
}

/// Googleユーザー情報をupsertし、旧セッションを失効させた上で新セッションを発行する。
///
/// ユーザーupsertとセッションローテーション（旧削除+新規発行）を1つのトランザクションにまとめ、以下を保証する。
/// - 往復削減: 従来は upsert(1文・autocommit) + BEGIN + DELETE + INSERT + COMMIT の
///   5段階だったが、BEGIN + upsert + 旧セッション削除/新規発行CTE + COMMIT の
///   4段階・2文・1コネクションチェックアウトに削減する。
/// - 旧セッション失効: 同一トランザクション内で user 行の upsert が行ロックを保持するため、
///   同一ユーザーの同時ログインは直列化され、旧セッションが残らない。
/// - 原子性: session発行失敗時は user の upsert もロールバックされる。
async fn upsert_user_and_rotate_session(
    pool: &PgPool,
    user_info: &GoogleUserInfo,
) -> Result<String, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let user = upsert_user_in_tx(&mut tx, user_info).await?;
    let session_token = rotate_session_in_tx(&mut tx, user.id).await?;
    tx.commit().await?;

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

/// 旧セッションの削除と新セッションの発行を1文でアトミックに行う。
/// データ変更CTEは外部から参照されなくても必ず実行されるため、DELETE が省略されることはない。
async fn rotate_session_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_id: uuid::Uuid,
) -> Result<String, sqlx::Error> {
    let session_id: (uuid::Uuid,) = sqlx::query_as(
        r#"
        WITH deleted AS (
            DELETE FROM sessions WHERE user_id = $1
        )
        INSERT INTO sessions (user_id)
        VALUES ($1)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok(session_id.0.to_string())
}

async fn upsert_user_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_info: &GoogleUserInfo,
) -> Result<User, sqlx::Error> {
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
    .fetch_one(&mut **tx)
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

    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;

    fn test_google_user_info() -> GoogleUserInfo {
        GoogleUserInfo {
            sub: "test-google-id-804".to_string(),
            email: "test804@example.com".to_string(),
            name: Some("テストユーザー804".to_string()),
            picture: None,
        }
    }

    async fn start_auth_test_pool() -> (PgPool, impl Drop) {
        let node = Postgres::default().start().await.unwrap();
        let port = node.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);
        let mut last_error = None;
        let mut pool_opt = None;
        for _ in 0..20 {
            match tokio::time::timeout(
                Duration::from_secs(2),
                PgPoolOptions::new().connect(&database_url),
            )
            .await
            {
                Ok(Ok(pool)) => {
                    pool_opt = Some(pool);
                    break;
                }
                Ok(Err(err)) => last_error = Some(format!("{err:?}")),
                Err(_) => {}
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        let pool = pool_opt.unwrap_or_else(|| panic!("Postgres接続失敗: {last_error:?}"));
        crate::db::run_migrations(&pool)
            .await
            .expect("マイグレーション失敗");
        (pool, node)
    }

    async fn session_count(pool: &PgPool, user_id: uuid::Uuid) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .expect("セッション件数取得失敗")
    }

    #[tokio::test]
    #[ignore = "requires Docker to run Postgres container"]
    async fn test_relogin_invalidates_old_session() {
        let (pool, _node) = start_auth_test_pool().await;
        let user_info = test_google_user_info();

        let first_token = upsert_user_and_rotate_session(&pool, &user_info)
            .await
            .expect("初回ログイン失敗");
        let first_id: uuid::Uuid = first_token.parse().expect("初回トークンがUUIDでない");
        let user_id = select_user_id_by_session(&pool, first_id)
            .await
            .expect("初回セッション参照失敗")
            .expect("初回セッションが存在しない");

        let second_token = upsert_user_and_rotate_session(&pool, &user_info)
            .await
            .expect("再ログイン失敗");
        assert_ne!(
            first_token, second_token,
            "再ログインで新トークンが発行されること"
        );
        let second_id: uuid::Uuid = second_token.parse().expect("再発行トークンがUUIDでない");

        assert!(
            select_user_id_by_session(&pool, first_id)
                .await
                .expect("旧セッション参照失敗")
                .is_none(),
            "旧トークンは失効していること"
        );
        assert_eq!(
            select_user_id_by_session(&pool, second_id)
                .await
                .expect("新セッション参照失敗"),
            Some(user_id),
            "新トークンが有効であること"
        );
        assert_eq!(
            session_count(&pool, user_id).await,
            1,
            "セッションは1件のみ残ること"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 8)]
    #[ignore = "requires Docker to run Postgres container"]
    async fn test_concurrent_login_keeps_single_session() {
        let (pool, _node) = start_auth_test_pool().await;
        let user_info = test_google_user_info();

        let mut handles = Vec::new();
        for _ in 0..8 {
            let pool = pool.clone();
            let user_info = user_info.clone();
            handles.push(tokio::spawn(async move {
                upsert_user_and_rotate_session(&pool, &user_info).await
            }));
        }
        let mut tokens = Vec::new();
        for handle in handles {
            tokens.push(handle.await.expect("タスク失敗").expect("同時ログイン失敗"));
        }

        let mut live = 0;
        let mut live_user_id = None;
        for token in &tokens {
            let id: uuid::Uuid = token.parse().expect("発行トークンがUUIDでない");
            let user_id = select_user_id_by_session(&pool, id)
                .await
                .expect("セッション参照失敗");
            if user_id.is_some() {
                live += 1;
                live_user_id = user_id;
            }
        }
        assert_eq!(live, 1, "有効なトークンは1つのみであること");
        assert_eq!(
            session_count(&pool, live_user_id.expect("有効なセッションが存在しない")).await,
            1,
            "同時ログイン後もセッションは1件のみであること"
        );
    }
}
