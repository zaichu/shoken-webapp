#[cfg(target_arch = "wasm32")]
mod google_worker;

#[doc(hidden)]
pub mod google_common;

use sha2::{Digest, Sha256};
use shared::value::UserId;
use std::{fmt, str::FromStr};

use crate::db::{Bind, Db, DbError, Tx};
use crate::errors::ApiError;
use crate::models::user::{GoogleUserInfo, User};

pub use google_common::{GoogleOAuthClient, create_oauth_client};

/// セッション Cookie の値。内部は UUID v4。
/// Cookie の文字列表現と DB の `token_hash`(BYTEA) はいずれも
/// UUID の正準形（小文字ハイフン付き）文字列から導かれる
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionToken(uuid::Uuid);

impl SessionToken {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// `sessions.token_hash` と突き合わせるための SHA-256(正準形文字列)
    pub fn hash(self) -> Vec<u8> {
        Sha256::digest(self.0.to_string().as_bytes()).to_vec()
    }
}

impl Default for SessionToken {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SessionToken {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        uuid::Uuid::parse_str(s).map(Self)
    }
}

pub const SESSION_COOKIE_NAME: &str = "session_token";
pub const OAUTH_STATE_COOKIE_NAME: &str = "oauth_state";
pub const OAUTH_PKCE_VERIFIER_COOKIE_NAME: &str = "oauth_pkce_verifier";
pub const OAUTH_NONCE_COOKIE_NAME: &str = "oauth_nonce";

/// `/api/v1/oauth/google/authorize` が返す認可リダイレクト一式。
/// state/nonce/PKCE verifier は Cookie に保存し、callback で検証する
pub struct GoogleAuthFlow {
    pub authorize_url: String,
    pub state: String,
    pub nonce: String,
    pub pkce_verifier: String,
}

/// Google 認可 URL と state/nonce/PKCE を生成する
pub fn begin_google_auth(client: &GoogleOAuthClient) -> GoogleAuthFlow {
    google_common::begin_auth(client)
}

/// Google OAuth コードを検証し、ユーザーを upsert してセッショントークンを返す。
/// ID トークン検証は tokeninfo エンドポイントに委譲する
pub async fn authenticate_with_google_code(
    pool: &Db,
    oauth_client: &GoogleOAuthClient,
    code: String,
    pkce_verifier: String,
    nonce: &str,
) -> Result<SessionToken, ApiError> {
    let user_info = verify_code_with_google(oauth_client, &code, &pkce_verifier, nonce).await?;
    Ok(upsert_user_and_rotate_session(pool, &user_info).await?)
}

#[cfg(target_arch = "wasm32")]
async fn verify_code_with_google(
    oauth_client: &GoogleOAuthClient,
    code: &str,
    pkce_verifier: &str,
    nonce: &str,
) -> Result<GoogleUserInfo, ApiError> {
    google_worker::verify_code_with_google(oauth_client, code, pkce_verifier, nonce).await
}

// openapi.json はホストで生成されるためコールバックハンドラ経由で本関数もホストで
// コンパイルされる必要がある。Google への問い合わせ経路は Worker にしか存在しない
#[cfg(not(target_arch = "wasm32"))]
async fn verify_code_with_google(
    _oauth_client: &GoogleOAuthClient,
    _code: &str,
    _pkce_verifier: &str,
    _nonce: &str,
) -> Result<GoogleUserInfo, ApiError> {
    Err(ApiError::Internal(
        "Google OAuth のコード検証は Workers 環境でのみ利用できます",
    ))
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
// tests/db_integration.rs からの検証用に公開しているため docs には出さない
#[doc(hidden)]
pub async fn upsert_user_and_rotate_session(
    pool: &Db,
    user_info: &GoogleUserInfo,
) -> Result<SessionToken, DbError> {
    let mut tx = pool.begin().await?;
    let user = upsert_user_in_tx(&mut tx, user_info).await?;
    let session_token = rotate_session_in_tx(&mut tx, user.id).await?;
    tx.commit().await?;

    Ok(session_token)
}

pub async fn select_user_by_session(
    pool: &Db,
    token: SessionToken,
) -> Result<Option<User>, DbError> {
    crate::db::query_as::<User>(
        r#"
        SELECT u.id, u.google_id, u.email, u.name, u.picture_url, u.created_at, u.updated_at
        FROM users u
        INNER JOIN sessions s ON u.id = s.user_id
        WHERE s.token_hash = $1 AND s.expires_at > NOW()
        "#,
        vec![Bind::Bytes(token.hash())],
    )
    .fetch_optional(pool)
    .await
}

pub async fn select_user_id_by_session(
    pool: &Db,
    token: SessionToken,
) -> Result<Option<UserId>, DbError> {
    crate::db::query_scalar::<UserId>(
        r#"
        SELECT user_id FROM sessions
        WHERE token_hash = $1 AND expires_at > NOW()
        "#,
        vec![Bind::Bytes(token.hash())],
    )
    .fetch_optional(pool)
    .await
}

/// 旧セッションと期限切れセッションの削除、新セッションの発行を1文でアトミックに行う。
/// データ変更CTEは外部から参照されなくても必ず実行されるため、DELETE が省略されることはない。
async fn rotate_session_in_tx(tx: &mut Tx, user_id: UserId) -> Result<SessionToken, DbError> {
    let token = SessionToken::new();
    crate::db::query(
        r#"
        WITH expired AS (
            SELECT id FROM sessions WHERE expires_at <= NOW() FOR UPDATE SKIP LOCKED
        ),
        deleted AS (
            DELETE FROM sessions WHERE user_id = $1 OR id IN (SELECT id FROM expired)
        )
        INSERT INTO sessions (user_id, token_hash)
        VALUES ($1, $2)
        "#,
        vec![Bind::from(user_id), Bind::Bytes(token.hash())],
    )
    .execute(&mut *tx)
    .await?;

    Ok(token)
}

async fn upsert_user_in_tx(tx: &mut Tx, user_info: &GoogleUserInfo) -> Result<User, DbError> {
    crate::db::query_as::<User>(
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
        vec![
            Bind::from(user_info.sub.as_str()),
            Bind::from(user_info.email.as_str()),
            Bind::from(user_info.name.as_deref()),
            Bind::from(user_info.picture.as_deref()),
        ],
    )
    .fetch_one(&mut *tx)
    .await
}

pub async fn delete_session(pool: &Db, token: SessionToken) -> Result<(), DbError> {
    crate::db::query(
        "DELETE FROM sessions WHERE token_hash = $1",
        vec![Bind::Bytes(token.hash())],
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// ユーザーを削除（CASCADE により関連データも削除）
pub async fn delete_account(pool: &Db, user_id: UserId) -> Result<(), DbError> {
    crate::db::query("DELETE FROM users WHERE id = $1", vec![Bind::from(user_id)])
        .execute(pool)
        .await?;

    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_session_token_matches_canonical_uuid_digest() {
        let token: SessionToken = "550e8400-e29b-41d4-a716-446655440000".parse().unwrap();
        // SQL の sha256(convert_to(id::text, 'UTF8')) と同じ、
        // 小文字ハイフン付き正規形文字列への SHA-256 の固定値
        let digest_hex: String = token.hash().iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            digest_hex,
            "a3a9e1ed9732cab28868127be00f1ce921acaefdd5c3b23a6e9e0072bd9c1a34"
        );
    }

    fn test_google_user_info() -> GoogleUserInfo {
        GoogleUserInfo {
            sub: "test-google-id-804".to_string(),
            email: "test804@example.com".to_string(),
            name: Some("テストユーザー804".to_string()),
            picture: None,
        }
    }

    async fn session_count(pool: &Db, user_id: UserId) -> i64 {
        crate::db::query_scalar::<i64>(
            "SELECT COUNT(*) FROM sessions WHERE user_id = $1",
            vec![Bind::from(user_id)],
        )
        .fetch_one(pool)
        .await
        .expect("セッション件数取得失敗")
    }

    #[tokio::test]
    #[ignore = "requires Docker to run Postgres container"]
    async fn test_relogin_invalidates_old_session() {
        let (pool, _node) = crate::test_db::start_test_pool().await;
        let user_info = test_google_user_info();

        let first_token = upsert_user_and_rotate_session(&pool, &user_info)
            .await
            .expect("初回ログイン失敗");
        let user_id = select_user_id_by_session(&pool, first_token)
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

        assert!(
            select_user_id_by_session(&pool, first_token)
                .await
                .expect("旧セッション参照失敗")
                .is_none(),
            "旧トークンは失効していること"
        );
        assert_eq!(
            select_user_id_by_session(&pool, second_token)
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
        let (pool, _node) = crate::test_db::start_test_pool().await;
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
            let user_id = select_user_id_by_session(&pool, *token)
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
