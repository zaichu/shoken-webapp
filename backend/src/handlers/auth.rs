use crate::config;
use crate::errors::ApiError;
use crate::models::user::UserResponse;
use crate::services::auth::{self as auth_service, create_oauth_client};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Json, Redirect, Response},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use oauth2::{CsrfToken, Scope};
use serde::Deserialize;

/// コールバック時のクエリパラメータ
#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    pub code: String,
    pub state: String,
}

pub fn same_site(secure: bool) -> SameSite {
    if secure {
        SameSite::None
    } else {
        SameSite::Lax
    }
}

fn google_oauth_credentials(state: &AppState) -> Result<(&str, &str), ApiError> {
    let client_id =
        state.secrets.google_client_id.as_deref().ok_or_else(|| {
            ApiError::ApiError("GOOGLE_CLIENT_ID が設定されていません".to_string())
        })?;
    let client_secret = state
        .secrets
        .google_client_secret
        .as_deref()
        .ok_or_else(|| {
            ApiError::ApiError("GOOGLE_CLIENT_SECRET が設定されていません".to_string())
        })?;

    Ok((client_id, client_secret))
}

pub fn build_state_cookie(state: &str, secure: bool) -> Cookie<'static> {
    Cookie::build((auth_service::OAUTH_STATE_COOKIE_NAME, state.to_string()))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::minutes(10))
        .build()
}

pub fn clear_state_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((auth_service::OAUTH_STATE_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::seconds(0))
        .build()
}

pub fn build_session_cookie(token: &str, secure: bool) -> Cookie<'static> {
    Cookie::build((auth_service::SESSION_COOKIE_NAME, token.to_string()))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::days(7))
        .build()
}

pub fn clear_session_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((auth_service::SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::seconds(0))
        .build()
}

pub fn get_session_id_from_jar(jar: &CookieJar) -> Result<uuid::Uuid, ApiError> {
    let session_token = jar
        .get(auth_service::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| ApiError::Unauthorized("ログインが必要です".to_string()))?;

    let session_id: uuid::Uuid = session_token
        .parse()
        .map_err(|_| ApiError::Unauthorized("無効なセッショントークンです".to_string()))?;

    Ok(session_id)
}

/// Google OAuth認証を開始（直接リダイレクト）
pub async fn google_auth(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    let (client_id, client_secret) = google_oauth_credentials(&state)?;
    let client = create_oauth_client(client_id, client_secret)?;

    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    // CSRF トークンを Cookie に保存（10分間有効）
    let is_secure = config::is_secure_cookie();
    let state_cookie = build_state_cookie(csrf_token.secret(), is_secure);

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
    let jar = jar.remove(clear_state_cookie(is_secure));

    let (client_id, client_secret) = google_oauth_credentials(&state)?;
    let client = create_oauth_client(client_id, client_secret)?;

    let session_token = auth_service::authenticate_with_google_code(
        &state.pool,
        &state.client,
        &client,
        query.code,
    )
    .await?;

    // Cookieを設定
    // クロスオリジン（フロントエンド: GitHub Pages, バックエンド: Fly.io）で
    // Cookieを送受信するには SameSite=None + Secure が必要
    let cookie = build_session_cookie(&session_token, is_secure);
    let jar = jar.add(cookie);

    // フロントエンドにリダイレクト
    let frontend_url = &state.secrets.frontend_url;
    let redirect_url = format!("{}?login=success", frontend_url);

    Ok((jar, Redirect::to(&redirect_url)).into_response())
}

/// 現在ログイン中のユーザー情報を取得
pub async fn get_current_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<UserResponse>, ApiError> {
    let session_id = get_session_id_from_jar(&jar)?;

    // セッションテーブルからユーザーを取得（期限切れでないセッションのみ）
    let user = auth_service::select_user_by_session(&state.pool, session_id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("セッションが無効または期限切れです".to_string()))?;

    Ok(Json(user.into()))
}

/// ログアウト処理
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
    let cookie = clear_session_cookie(is_secure);

    let jar = jar.remove(cookie);

    (
        jar,
        Json(serde_json::json!({"message": "ログアウトしました"})),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::connect_pool_lazy,
        errors::ErrorResponse,
        models::common::MessageResponse,
        state::{AppState, Secrets},
    };
    use axum::{
        body::{to_bytes, Body},
        http::{Method, Request, StatusCode},
        Router,
    };
    use axum_extra::extract::{
        cookie::{Cookie, SameSite},
        CookieJar,
    };
    use {reqwest::Client, serde::de::DeserializeOwned, std::sync::Arc, tower::ServiceExt};
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
            dividend_cache: crate::state::DividendCacheState::default(),
        }
    }
    fn test_app() -> Router {
        crate::handlers::v1::auth_routes().with_state(make_test_state())
    }
    async fn read_json_response<T: DeserializeOwned>(response: axum::response::Response) -> T {
        let body = to_bytes(response.into_body(), BODY_LIMIT).await.unwrap();
        serde_json::from_slice(&body).unwrap()
    }
    async fn request_json<T: DeserializeOwned>(method: Method, uri: &str) -> (StatusCode, T) {
        let response = test_app()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        (status, read_json_response(response).await)
    }
    #[tokio::test]
    async fn test_auth_endpoints_without_cookie() {
        let (status, error) = request_json::<ErrorResponse>(Method::GET, "/api/v1/session").await;
        assert_eq!(
            (
                status,
                error.error.code.as_str(),
                error.error.message.contains("ログインが必要")
            ),
            (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", true)
        );
        let (status, message) =
            request_json::<MessageResponse>(Method::DELETE, "/api/v1/session").await;
        assert_eq!(
            (status, message.message.as_str()),
            (StatusCode::OK, "ログアウトしました")
        );
    }
    #[test]
    fn test_cookie_helpers() {
        use crate::services::auth::{OAUTH_STATE_COOKIE_NAME, SESSION_COOKIE_NAME};
        for (secure, expected_secure) in [(true, true), (false, false)] {
            for (cookie, expected_name, expected_value) in [
                (
                    build_state_cookie("test_state", secure),
                    OAUTH_STATE_COOKIE_NAME,
                    "test_state",
                ),
                (
                    build_session_cookie("test_token", secure),
                    SESSION_COOKIE_NAME,
                    "test_token",
                ),
            ] {
                assert_eq!(
                    (
                        cookie.name(),
                        cookie.value(),
                        cookie.secure(),
                        cookie.http_only()
                    ),
                    (
                        expected_name,
                        expected_value,
                        Some(expected_secure),
                        Some(true)
                    )
                );
            }
        }
        for (cookie, expected_name) in [
            (clear_state_cookie(true), OAUTH_STATE_COOKIE_NAME),
            (clear_session_cookie(true), SESSION_COOKIE_NAME),
        ] {
            assert_eq!((cookie.name(), cookie.value()), (expected_name, ""));
        }
        assert_eq!(
            (same_site(true), same_site(false)),
            (SameSite::None, SameSite::Lax)
        );
    }
    #[test]
    fn test_jar_helpers() {
        use crate::services::auth::SESSION_COOKIE_NAME;
        assert!(get_session_id_from_jar(&CookieJar::new()).is_err());
        assert!(get_session_id_from_jar(
            &CookieJar::new().add(Cookie::new(SESSION_COOKIE_NAME, "invalid-uuid"))
        )
        .is_err());
        let uuid = uuid::Uuid::new_v4();
        assert_eq!(
            get_session_id_from_jar(
                &CookieJar::new().add(Cookie::new(SESSION_COOKIE_NAME, uuid.to_string()))
            )
            .unwrap(),
            uuid
        );
    }
}
