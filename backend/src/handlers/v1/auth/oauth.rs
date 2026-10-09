use crate::errors::{ApiError, ConfigError};
use crate::services::auth::{self as auth_service, GoogleOAuthClient, SessionToken};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use serde::Deserialize;

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

fn google_oauth_client(state: &AppState) -> Result<&GoogleOAuthClient, ApiError> {
    state.google_oauth.as_ref().ok_or_else(|| {
        ApiError::Config(ConfigError::Missing(
            "GOOGLE_CLIENT_ID / GOOGLE_CLIENT_SECRET が設定されていません",
        ))
    })
}

fn build_oauth_cookie(name: &'static str, value: &str, secure: bool) -> Cookie<'static> {
    Cookie::build((name, value.to_string()))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::minutes(10))
        .build()
}

fn clear_oauth_cookie(name: &'static str, secure: bool) -> Cookie<'static> {
    Cookie::build((name, ""))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::seconds(0))
        .build()
}

pub fn build_state_cookie(state: &str, secure: bool) -> Cookie<'static> {
    build_oauth_cookie(auth_service::OAUTH_STATE_COOKIE_NAME, state, secure)
}

pub fn clear_state_cookie(secure: bool) -> Cookie<'static> {
    clear_oauth_cookie(auth_service::OAUTH_STATE_COOKIE_NAME, secure)
}

pub fn build_session_cookie(token: SessionToken, secure: bool) -> Cookie<'static> {
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

pub fn get_session_id_from_jar(jar: &CookieJar) -> Result<SessionToken, ApiError> {
    jar.get(auth_service::SESSION_COOKIE_NAME)
        .ok_or_else(|| ApiError::Unauthorized("ログインが必要です"))?
        .value()
        .parse()
        .map_err(|_| ApiError::Unauthorized("無効なセッショントークンです"))
}

pub async fn google_auth(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    let client = google_oauth_client(&state)?;

    let flow = auth_service::begin_google_auth(client);

    let is_secure = state.config.secure_cookie;
    let jar = jar
        .add(build_state_cookie(&flow.state, is_secure))
        .add(build_oauth_cookie(
            auth_service::OAUTH_PKCE_VERIFIER_COOKIE_NAME,
            &flow.pkce_verifier,
            is_secure,
        ))
        .add(build_oauth_cookie(
            auth_service::OAUTH_NONCE_COOKIE_NAME,
            &flow.nonce,
            is_secure,
        ));

    Ok((jar, Redirect::to(flow.authorize_url.as_str())))
}

pub async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<AuthCallbackQuery>,
    jar: CookieJar,
) -> Result<Response, ApiError> {
    let stored_state = jar
        .get(auth_service::OAUTH_STATE_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| ApiError::Unauthorized("OAuth state が見つかりません"))?;

    if query.state != stored_state {
        return Err(ApiError::Unauthorized("OAuth state が一致しません"));
    }

    let is_secure = state.config.secure_cookie;
    let pkce_verifier = jar
        .get(auth_service::OAUTH_PKCE_VERIFIER_COOKIE_NAME)
        .map(|c| c.value().to_string());
    let nonce = jar
        .get(auth_service::OAUTH_NONCE_COOKIE_NAME)
        .map(|c| c.value().to_string());
    let jar = jar
        .remove(clear_state_cookie(is_secure))
        .remove(clear_oauth_cookie(
            auth_service::OAUTH_PKCE_VERIFIER_COOKIE_NAME,
            is_secure,
        ))
        .remove(clear_oauth_cookie(
            auth_service::OAUTH_NONCE_COOKIE_NAME,
            is_secure,
        ));

    let client = google_oauth_client(&state)?;

    let pkce_verifier = pkce_verifier
        .ok_or_else(|| ApiError::Unauthorized("OAuth の PKCE verifier が見つかりません"))?;
    let nonce = nonce.ok_or_else(|| ApiError::Unauthorized("OAuth nonce が見つかりません"))?;

    let session_token = auth_service::authenticate_with_google_code(
        &state.pool,
        client,
        query.code,
        pkce_verifier,
        &nonce,
    )
    .await?;

    // クロスオリジン（フロントエンド: Cloudflare Pages, バックエンド: Cloudflare Workers）で
    // Cookieを送受信するには SameSite=None + Secure が必要
    let cookie = build_session_cookie(session_token, is_secure);
    let jar = jar.add(cookie);

    let frontend_url = &state.secrets.frontend_url;
    let redirect_url = format!("{frontend_url}?login=success");

    Ok((jar, Redirect::to(&redirect_url)).into_response())
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
        Router,
        body::{Body, to_bytes},
        extract::{Query, State},
        http::{Method, Request, StatusCode},
    };
    use axum_extra::extract::{
        CookieJar,
        cookie::{Cookie, SameSite},
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
            config: Arc::new(crate::config::Config::default()),
            google_oauth: None,
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
        let token: SessionToken = "550e8400-e29b-41d4-a716-446655440000".parse().unwrap();
        for (secure, expected_secure) in [(true, true), (false, false)] {
            for (cookie, expected_name, expected_value) in [
                (
                    build_state_cookie("test_state", secure),
                    OAUTH_STATE_COOKIE_NAME,
                    "test_state",
                ),
                (
                    build_session_cookie(token, secure),
                    SESSION_COOKIE_NAME,
                    "550e8400-e29b-41d4-a716-446655440000",
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
    #[tokio::test]
    async fn test_google_oauth_client() {
        let mut state = make_test_state();
        assert!(google_oauth_client(&state).is_err());
        state.google_oauth = Some(
            crate::services::auth::create_oauth_client(
                "test-client-id",
                "test-client-secret",
                "http://localhost:3001",
            )
            .unwrap(),
        );
        assert!(google_oauth_client(&state).is_ok());
    }

    #[tokio::test]
    async fn test_google_callback_matching_state_proceeds_past_state_check() {
        use crate::services::auth::OAUTH_STATE_COOKIE_NAME;
        // state が一致する場合は Unauthorized にならず、後段の OAuth クライアント確認へ進む
        let jar = CookieJar::new().add(Cookie::new(OAUTH_STATE_COOKIE_NAME, "test-state"));
        let result = google_callback(
            State(make_test_state()),
            Query(AuthCallbackQuery {
                code: "auth-code".to_string(),
                state: "test-state".to_string(),
            }),
            jar,
        )
        .await;
        let err = result.expect_err("OAuth クライアント未設定のためエラーになる");
        assert!(
            matches!(err, ApiError::Config(_)),
            "state 一致時は Config エラーになるはず: {err:?}"
        );
    }

    #[test]
    fn test_jar_helpers() {
        use crate::services::auth::SESSION_COOKIE_NAME;
        assert!(get_session_id_from_jar(&CookieJar::new()).is_err());
        assert!(
            get_session_id_from_jar(
                &CookieJar::new().add(Cookie::new(SESSION_COOKIE_NAME, "invalid-uuid"))
            )
            .is_err()
        );
        let token = SessionToken::new();
        assert_eq!(
            get_session_id_from_jar(
                &CookieJar::new().add(Cookie::new(SESSION_COOKIE_NAME, token.to_string()))
            )
            .unwrap(),
            token
        );
    }
}
