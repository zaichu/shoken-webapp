use crate::errors::ApiError;
use crate::models::user::{GoogleUserInfo, User, UserResponse};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json, Redirect, Response},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
const SESSION_COOKIE_NAME: &str = "session_token";

/// OAuth認証開始時のレスポンス
#[derive(Debug, Serialize)]
pub struct AuthUrlResponse {
    pub auth_url: String,
}

/// コールバック時のクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    #[allow(dead_code)]
    pub state: Option<String>,
}

/// OAuthクライアントを作成
fn create_oauth_client(state: &AppState) -> Result<BasicClient, ApiError> {
    let client_id = state
        .secrets
        .google_client_id
        .clone()
        .ok_or_else(|| ApiError::ApiError("GOOGLE_CLIENT_ID が設定されていません".to_string()))?;

    let client_secret = state
        .secrets
        .google_client_secret
        .clone()
        .ok_or_else(|| ApiError::ApiError("GOOGLE_CLIENT_SECRET が設定されていません".to_string()))?;

    let redirect_url = format!("{}/auth/google/callback", get_backend_url());

    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(GOOGLE_AUTH_URL.to_string())
            .map_err(|e| ApiError::ApiError(format!("認証URL解析エラー: {}", e)))?,
        Some(
            TokenUrl::new(GOOGLE_TOKEN_URL.to_string())
                .map_err(|e| ApiError::ApiError(format!("トークンURL解析エラー: {}", e)))?,
        ),
    )
    .set_redirect_uri(
        RedirectUrl::new(redirect_url)
            .map_err(|e| ApiError::ApiError(format!("リダイレクトURL解析エラー: {}", e)))?,
    );

    Ok(client)
}

/// バックエンドURLを取得
fn get_backend_url() -> String {
    std::env::var("BACKEND_URL").unwrap_or_else(|_| {
        let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
        format!("http://localhost:{}", port)
    })
}

/// Google OAuth認証を開始
pub async fn google_auth(State(state): State<AppState>) -> Result<Json<AuthUrlResponse>, ApiError> {
    let client = create_oauth_client(&state)?;

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Ok(Json(AuthUrlResponse {
        auth_url: auth_url.to_string(),
    }))
}

/// Google OAuthコールバックを処理
pub async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<AuthCallbackQuery>,
    jar: CookieJar,
) -> Result<Response, ApiError> {
    let client = create_oauth_client(&state)?;

    // 認証コードをトークンに交換
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| ApiError::ApiError(format!("トークン交換エラー: {:?}", e)))?;

    let access_token = token_result.access_token().secret();

    // Googleユーザー情報を取得
    let user_info: GoogleUserInfo = state
        .client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| ApiError::NetworkError(format!("ユーザー情報取得エラー: {}", e)))?
        .json()
        .await
        .map_err(|e| ApiError::NetworkError(format!("ユーザー情報解析エラー: {}", e)))?;

    // ユーザーをデータベースに登録または更新
    let user = upsert_user(&state, &user_info).await?;

    // セッショントークンを生成（シンプルな実装ではユーザーIDを使用）
    let session_token = user.id.to_string();

    // Cookieを設定
    let cookie = Cookie::build((SESSION_COOKIE_NAME, session_token))
        .path("/")
        .http_only(true)
        .secure(std::env::var("BACKEND_URL").is_ok()) // 本番環境ではsecure
        .max_age(time::Duration::days(7))
        .build();

    let jar = jar.add(cookie);

    // フロントエンドにリダイレクト
    let frontend_url = &state.secrets.frontend_url;
    let redirect_url = format!("{}?login=success", frontend_url);

    Ok((jar, Redirect::to(&redirect_url)).into_response())
}

/// ユーザーを登録または更新
async fn upsert_user(state: &AppState, user_info: &GoogleUserInfo) -> Result<User, ApiError> {
    let user = sqlx::query_as::<_, User>(
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
    .fetch_one(&state.pool)
    .await?;

    Ok(user)
}

/// 現在ログイン中のユーザー情報を取得
pub async fn get_current_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<UserResponse>, ApiError> {
    let session_token = jar
        .get(SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| ApiError::Unauthorized("ログインが必要です".to_string()))?;

    let user_id: uuid::Uuid = session_token
        .parse()
        .map_err(|_| ApiError::Unauthorized("無効なセッショントークンです".to_string()))?;

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, google_id, email, name, picture_url, created_at, updated_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::Unauthorized("ユーザーが見つかりません".to_string()))?;

    Ok(Json(user.into()))
}

/// ログアウト処理
pub async fn logout(jar: CookieJar) -> impl IntoResponse {
    let cookie = Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(0))
        .build();

    let jar = jar.remove(cookie);

    (jar, Json(serde_json::json!({"message": "ログアウトしました"})))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        // モジュールが正常にコンパイルされることを確認
        assert!(true);
    }
}
