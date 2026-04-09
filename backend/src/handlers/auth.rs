use crate::config;
use crate::errors::{ApiError, ErrorResponse};
use crate::models::common::MessageResponse;
use crate::models::user::{GoogleUserInfo, UserResponse};
use crate::services::auth::{self as auth_service, create_oauth_client};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json, Redirect, Response},
    routing::{delete, get, post},
    Router,
};
use axum_extra::extract::CookieJar;
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;

const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";

pub fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/google", get(google_auth))
        .route("/auth/google/callback", get(google_callback))
        .route("/auth/me", get(get_current_user))
        .route("/auth/logout", post(logout))
        .route("/auth/delete-account", delete(delete_account))
}
/// コールバック時のクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String,
}

/// Google OAuth認証を開始（直接リダイレクト）
pub async fn google_auth(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    let client = create_oauth_client(&state)?;

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    // CSRF トークンを Cookie に保存（10分間有効）
    let is_secure = config::is_secure_cookie();
    let state_cookie = auth_service::build_state_cookie(csrf_token.secret(), is_secure);

    let jar = jar.add(state_cookie);

    Ok((jar, Redirect::to(auth_url.as_str())))
}

/// Google OAuthコールバックを処理
pub async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<AuthCallbackQuery>,
    jar: CookieJar,
) -> Result<Response, ApiError> {
    // CSRF トークンを検証
    let stored_state = jar
        .get(auth_service::OAUTH_STATE_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| ApiError::Unauthorized("OAuth state が見つかりません".to_string()))?;

    if query.state != stored_state {
        return Err(ApiError::Unauthorized(
            "OAuth state が一致しません".to_string(),
        ));
    }

    // state Cookie を削除
    let is_secure = config::is_secure_cookie();
    let jar = jar.remove(auth_service::clear_state_cookie(is_secure));

    let client = create_oauth_client(&state)?;

    // 認証コードをトークンに交換（oauth2 5.0.0 の新しい HTTP クライアント API）
    let http_client = oauth2::reqwest::Client::new();
    let token_result = client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await
        .map_err(|e| {
            tracing::error!("OAuth token exchange error: {}", e);
            ApiError::OAuthError(e.to_string())
        })?;

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
    let user = auth_service::upsert_user(&state.pool, &user_info).await?;

    // ランダムなセッショントークンを生成してデータベースに保存
    let session_token = auth_service::create_session(&state.pool, user.id).await?;

    // Cookieを設定
    // クロスオリジン（フロントエンド: GitHub Pages, バックエンド: Fly.io）で
    // Cookieを送受信するには SameSite=None + Secure が必要
    let is_secure = config::is_secure_cookie();
    let cookie = auth_service::build_session_cookie(&session_token, is_secure);

    let jar = jar.add(cookie);

    // フロントエンドにリダイレクト
    let frontend_url = &state.secrets.frontend_url;
    let redirect_url = format!("{}?login=success", frontend_url);

    Ok((jar, Redirect::to(&redirect_url)).into_response())
}

/// 現在ログイン中のユーザー情報を取得
#[utoipa::path(
    get,
    path = "/auth/me",
    operation_id = "auth_me",
    responses(
        (status = 200, body = UserResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn get_current_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<UserResponse>, ApiError> {
    let session_id = auth_service::get_session_id_from_jar(&jar)?;

    // セッションテーブルからユーザーを取得（期限切れでないセッションのみ）
    let user = auth_service::select_user_by_session(&state.pool, session_id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("セッションが無効または期限切れです".to_string()))?;

    Ok(Json(user.into()))
}

/// ログアウト処理
#[utoipa::path(
    post,
    path = "/auth/logout",
    operation_id = "auth_logout",
    responses(
        (status = 200, body = MessageResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    // セッションをデータベースから削除
    if let Some(session_token) = jar
        .get(auth_service::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
    {
        if let Ok(session_id) = session_token.parse::<uuid::Uuid>() {
            let _ = auth_service::delete_session(&state.pool, session_id).await;
        }
    }

    let is_secure = config::is_secure_cookie();
    let cookie = auth_service::clear_session_cookie(is_secure);

    let jar = jar.remove(cookie);

    (
        jar,
        Json(serde_json::json!({"message": "ログアウトしました"})),
    )
}

/// アカウント削除処理
/// ユーザーとすべての関連データ（sessions, dividends, domestic_stocks, mutualfunds, asset_balances）を削除
#[utoipa::path(
    delete,
    path = "/auth/delete-account",
    operation_id = "auth_delete_account",
    responses(
        (status = 200, body = MessageResponse),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_account(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = auth_service::get_session_id_from_jar(&jar)?;

    // セッションからユーザーIDを取得
    let user_id = auth_service::select_user_id_by_session(&state.pool, session_id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("セッションが無効または期限切れです".to_string()))?;

    // ユーザーを削除（CASCADE により関連データも削除）
    auth_service::delete_account(&state.pool, user_id).await?;

    // セッションCookieを削除
    let is_secure = config::is_secure_cookie();
    let cookie = auth_service::clear_session_cookie(is_secure);

    let jar = jar.remove(cookie);

    Ok((
        jar,
        Json(serde_json::json!({"message": "アカウントを削除しました"})),
    ))
}

#[cfg(test)]
mod tests {
    #[rustfmt::skip]
    use crate::{config::Config, db::connect_pool_lazy, errors::ErrorResponse, models::common::MessageResponse, routes::app_router, state::{AppState, Secrets}};
    #[rustfmt::skip]
    use axum::{body::{to_bytes, Body}, http::{Request, StatusCode}, Router};
    use reqwest::Client;
    use serde::de::DeserializeOwned;
    use std::sync::Arc;
    use tower::ServiceExt;

    const BODY_LIMIT: usize = 1024 * 1024;

    #[rustfmt::skip]
    fn make_test_state() -> AppState { let database_url = "postgresql://user:password@localhost/test_db"; let pool = connect_pool_lazy(database_url, 1).expect("pool"); let secrets = Arc::new(Secrets { database_url: database_url.to_string(), jquants_api_key: None, google_client_id: None, google_client_secret: None, frontend_url: "http://localhost:8080".to_string() }); AppState { pool, secrets, client: Client::new(), dividend_cache: crate::state::DividendCacheState::default() } }

    #[rustfmt::skip]
    fn test_app() -> Router { let config = Config { auth_rate_limit_rps: 0, jquants_rate_limit_rps: 0, ..Config::default() }; app_router(make_test_state(), &config) }

    #[rustfmt::skip]
    async fn read_json_response<T: DeserializeOwned>(response: axum::response::Response) -> T { let body = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap(); serde_json::from_slice(&body).unwrap() }

    #[rustfmt::skip]
    async fn request_json<T: DeserializeOwned>(method: &str, uri: &str) -> (StatusCode, T) { let response = test_app().oneshot(Request::builder().method(method).uri(uri).body(Body::empty()).unwrap()).await.unwrap(); let status = response.status(); (status, read_json_response(response).await) }

    #[tokio::test]
    #[rustfmt::skip]
    async fn test_auth_endpoints_without_cookie() {
        let (status, error) = request_json::<ErrorResponse>("GET", "/auth/me").await; assert_eq!((status, error.error.code.as_str(), error.error.message.contains("ログインが必要")), (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", true));
        let (status, message) = request_json::<MessageResponse>("POST", "/auth/logout").await; assert_eq!((status, message.message.as_str()), (StatusCode::OK, "ログアウトしました"));
    }
}
