use crate::config;
use crate::errors::{ApiError, ErrorResponse};
use crate::models::common::MessageResponse;
use crate::models::user::{GoogleUserInfo, UserResponse};
use crate::services::auth as auth_service;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json, Redirect, Response},
};
use axum_extra::extract::CookieJar;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EndpointNotSet, EndpointSet, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use serde::Deserialize;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v3/userinfo";
/// コールバック時のクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String,
}

/// OAuthクライアントの型エイリアス（oauth2 5.0.0 の新しい型システム対応）
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

/// OAuthクライアントを作成
fn create_oauth_client(state: &AppState) -> Result<GoogleOAuthClient, ApiError> {
    let client_id =
        state.secrets.google_client_id.clone().ok_or_else(|| {
            ApiError::ApiError("GOOGLE_CLIENT_ID が設定されていません".to_string())
        })?;

    let client_secret = state.secrets.google_client_secret.clone().ok_or_else(|| {
        ApiError::ApiError("GOOGLE_CLIENT_SECRET が設定されていません".to_string())
    })?;

    let redirect_url = format!("{}/auth/google/callback", config::backend_url());

    // oauth2 5.0.0 のビルダーパターンを使用
    let client = BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
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
            tracing::error!("OAuth token exchange error: {:?}", e);
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
    use crate::{
        config::Config,
        db::connect_pool_lazy,
        errors::ErrorResponse,
        models::common::MessageResponse,
        routes::app_router,
        state::{AppState, Secrets},
    };
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        Router,
    };
    use reqwest::Client;
    use serde::de::DeserializeOwned;
    use std::sync::{atomic::AtomicBool, Arc};
    use tower::ServiceExt;

    const BODY_LIMIT: usize = 1024 * 1024;

    fn make_test_state() -> AppState {
        let database_url = "postgresql://user:password@localhost/test_db";
        let pool = connect_pool_lazy(database_url, 1).expect("pool");
        let secrets = Arc::new(Secrets {
            database_url: database_url.to_string(),
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
        });

        AppState {
            pool,
            secrets,
            client: Client::new(),
            background_task_running: Arc::new(AtomicBool::new(false)),
        }
    }

    fn test_app() -> Router {
        let config = Config {
            auth_rate_limit_rps: 0,
            jquants_rate_limit_rps: 0,
            ..Config::default()
        };

        app_router(make_test_state(), &config)
    }

    async fn read_json_response<T: DeserializeOwned>(response: axum::response::Response) -> T {
        let body = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn test_get_current_user_no_cookie() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/me")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let error: ErrorResponse = read_json_response(response).await;
        assert_eq!(error.error.code, "UNAUTHORIZED");
        assert!(error.error.message.contains("ログインが必要"));
    }

    #[tokio::test]
    async fn test_logout_no_cookie() {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/auth/logout")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let message: MessageResponse = read_json_response(response).await;
        assert_eq!(message.message, "ログアウトしました");
    }
}
