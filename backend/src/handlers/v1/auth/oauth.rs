use crate::errors::{ApiError, ConfigError, ErrorResponse};
use crate::services::auth::{self as auth_service, GoogleOAuthClient, SessionToken};
use crate::services::login_event;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthCallbackQuery {
    code: String,
    state: String,
}

pub const fn same_site(secure: bool) -> SameSite {
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

fn auth_cookie(
    name: &'static str,
    value: String,
    max_age: time::Duration,
    secure: bool,
) -> Cookie<'static> {
    Cookie::build((name, value))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(max_age)
        .build()
}

fn build_oauth_cookie(name: &'static str, value: &str, secure: bool) -> Cookie<'static> {
    auth_cookie(name, value.to_string(), time::Duration::minutes(10), secure)
}

fn clear_cookie(name: &'static str, secure: bool) -> Cookie<'static> {
    auth_cookie(name, String::new(), time::Duration::seconds(0), secure)
}

fn build_state_cookie(state: &str, secure: bool) -> Cookie<'static> {
    build_oauth_cookie(auth_service::OAUTH_STATE_COOKIE_NAME, state, secure)
}

fn clear_state_cookie(secure: bool) -> Cookie<'static> {
    clear_cookie(auth_service::OAUTH_STATE_COOKIE_NAME, secure)
}

fn build_session_cookie(token: SessionToken, secure: bool) -> Cookie<'static> {
    auth_cookie(
        auth_service::SESSION_COOKIE_NAME,
        token.to_string(),
        time::Duration::days(7),
        secure,
    )
}

fn build_device_cookie(token: auth_service::DeviceToken, secure: bool) -> Cookie<'static> {
    auth_cookie(
        auth_service::DEVICE_COOKIE_NAME,
        token.to_string(),
        time::Duration::days(365),
        secure,
    )
}

pub fn clear_session_cookie(secure: bool) -> Cookie<'static> {
    clear_cookie(auth_service::SESSION_COOKIE_NAME, secure)
}

fn oauth_state_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn get_session_id_from_jar(jar: &CookieJar) -> Result<SessionToken, ApiError> {
    parse_session_id_from_jar(jar)
}

fn parse_session_id_from_jar(jar: &CookieJar) -> Result<SessionToken, ApiError> {
    jar.get(auth_service::SESSION_COOKIE_NAME)
        .or_else(|| jar.get(auth_service::LEGACY_SESSION_COOKIE_NAME))
        .ok_or_else(|| ApiError::Unauthorized("ログインが必要です"))?
        .value()
        .parse()
        .map_err(|_| ApiError::Unauthorized("無効なセッショントークンです"))
}

pub fn session_cookie_needs_migration(jar: &CookieJar) -> Option<SessionToken> {
    if jar.get(auth_service::SESSION_COOKIE_NAME).is_some() {
        return None;
    }
    jar.get(auth_service::LEGACY_SESSION_COOKIE_NAME)
        .and_then(|cookie| cookie.value().parse().ok())
}

fn clear_legacy_session_cookie(secure: bool) -> Cookie<'static> {
    clear_cookie(auth_service::LEGACY_SESSION_COOKIE_NAME, secure)
}

fn set_cookie_header(cookie: &Cookie<'static>) -> Result<axum::http::HeaderValue, ApiError> {
    axum::http::HeaderValue::from_str(&cookie.encoded().to_string())
        .map_err(|_| ApiError::Internal("Cookieの設定に失敗しました"))
}

pub fn migrate_session_cookie(
    response: &mut Response,
    jar: &CookieJar,
    secure: bool,
) -> Result<(), ApiError> {
    let Some(token) = session_cookie_needs_migration(jar) else {
        return Ok(());
    };
    let new_cookie = build_session_cookie(token, secure);
    response.headers_mut().append(
        axum::http::header::SET_COOKIE,
        set_cookie_header(&new_cookie)?,
    );
    let mut legacy = clear_legacy_session_cookie(secure);
    legacy.make_removal();
    response
        .headers_mut()
        .append(axum::http::header::SET_COOKIE, set_cookie_header(&legacy)?);
    Ok(())
}

pub fn clear_all_session_cookies(response: &mut Response, secure: bool) -> Result<(), ApiError> {
    for cookie in [
        clear_session_cookie(secure),
        clear_legacy_session_cookie(secure),
    ] {
        let mut cookie = cookie;
        cookie.make_removal();
        response
            .headers_mut()
            .append(axum::http::header::SET_COOKIE, set_cookie_header(&cookie)?);
    }
    Ok(())
}

pub fn get_device_token_from_jar(jar: &CookieJar) -> auth_service::DeviceToken {
    jar.get(auth_service::DEVICE_COOKIE_NAME)
        .and_then(|c| c.value().parse().ok())
        .unwrap_or_default()
}

/// Google OAuth 認証を開始（v1）
#[utoipa::path(
    get,
    path = "/api/v1/oauth/google/authorize",
    operation_id = "v1_google_authorize",
    tags = ["handlers::v1::auth"],
    responses(
        (status = 302, description = "Google OAuth 認証ページへリダイレクト"),
        (status = 500, description = "サーバーエラー", body = ErrorResponse),
    ),
)]
pub async fn google_auth(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Redirect), ApiError> {
    let client = google_oauth_client(&state)?;

    let flow = auth_service::google_common::begin_auth(client);

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

/// Google OAuth コールバックを処理（v1）
#[utoipa::path(
    get,
    path = "/api/v1/oauth/google/callback",
    operation_id = "v1_google_callback",
    tags = ["handlers::v1::auth"],
    params(
        ("code" = String, Query, description = "Google OAuth 認証コード"),
        ("state" = String, Query, description = "OAuth CSRF state")
    ),
    responses(
        (status = 302, description = "ログイン成功後にフロントエンドへリダイレクト"),
        (status = 401, description = "OAuth の認証に失敗", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse),
    ),
)]
pub async fn google_callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthCallbackQuery>,
    jar: CookieJar,
) -> Result<Response, ApiError> {
    let stored_state = jar
        .get(auth_service::OAUTH_STATE_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| ApiError::Unauthorized("OAuth state が見つかりません"))?;

    if !oauth_state_equal(&query.state, &stored_state) {
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
        .remove(clear_cookie(
            auth_service::OAUTH_PKCE_VERIFIER_COOKIE_NAME,
            is_secure,
        ))
        .remove(clear_cookie(
            auth_service::OAUTH_NONCE_COOKIE_NAME,
            is_secure,
        ));

    let client = google_oauth_client(&state)?;

    let pkce_verifier = pkce_verifier
        .ok_or_else(|| ApiError::Unauthorized("OAuth の PKCE verifier が見つかりません"))?;
    let nonce = nonce.ok_or_else(|| ApiError::Unauthorized("OAuth nonce が見つかりません"))?;

    let device_token = get_device_token_from_jar(&jar);
    let client_info = login_event::client_info_from_headers(&headers);

    let (session_token, user, outcome) =
        auth_service::authenticate_with_google_code_and_login_event(
            &state.pool,
            client,
            query.code,
            pkce_verifier,
            &nonce,
            device_token.hash(),
            &client_info,
        )
        .await?;

    crate::services::login_notify::maybe_notify_on_login(
        &state.pool,
        state.secrets.gmail_user.as_deref(),
        state.secrets.gmail_app_password.as_deref(),
        &state.secrets.frontend_url,
        &user,
        outcome,
        &client_info,
    )
    .await;

    // クロスオリジン（フロントエンド: Cloudflare Pages, バックエンド: Cloudflare Workers）で
    // Cookieを送受信するには SameSite=None + Secure が必要
    let cookie = build_session_cookie(session_token, is_secure);
    let jar = jar
        .remove(clear_legacy_session_cookie(is_secure))
        .add(cookie)
        .add(build_device_cookie(device_token, is_secure));

    let frontend_url = &state.secrets.frontend_url;
    let redirect_url = format!("{frontend_url}?login=success");

    Ok((jar, Redirect::to(&redirect_url)).into_response())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::db;
    use crate::errors::ErrorResponse;
    use crate::models::common::MessageResponse;
    use crate::state::{AppState, Secrets};
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
    use {serde::de::DeserializeOwned, std::sync::Arc, tower::ServiceExt};
    const BODY_LIMIT: usize = 1024 * 1024;
    fn make_test_state() -> AppState {
        let pool =
            db::connect_pool_lazy("postgresql://user:password@localhost/test_db", 1).expect("pool");
        let secrets = Arc::new(Secrets {
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
            gmail_user: None,
            gmail_app_password: None,
        });
        AppState {
            pool,
            secrets,
            config: Arc::new(crate::config::Config::default()),
            google_oauth: None,
            jquants_client: None,
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
        use crate::services::auth;
        let token: SessionToken = "550e8400-e29b-41d4-a716-446655440000".parse().unwrap();
        for (secure, expected_secure) in [(true, true), (false, false)] {
            for (cookie, expected_name, expected_value) in [
                (
                    build_state_cookie("test_state", secure),
                    auth::OAUTH_STATE_COOKIE_NAME,
                    "test_state",
                ),
                (
                    build_session_cookie(token, secure),
                    auth::SESSION_COOKIE_NAME,
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
            (clear_state_cookie(true), auth::OAUTH_STATE_COOKIE_NAME),
            (clear_session_cookie(true), auth::SESSION_COOKIE_NAME),
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
                "http://localhost:8787",
            )
            .unwrap(),
        );
        assert!(google_oauth_client(&state).is_ok());
    }

    #[tokio::test]
    async fn test_google_callback_matching_state_proceeds_past_state_check() {
        use crate::services::auth;
        // state が一致する場合は Unauthorized にならず、後段の OAuth クライアント確認へ進む
        let jar = CookieJar::new().add(Cookie::new(auth::OAUTH_STATE_COOKIE_NAME, "test-state"));
        let result = google_callback(
            State(make_test_state()),
            HeaderMap::default(),
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

    #[tokio::test]
    async fn test_google_callback_mismatched_state_is_unauthorized() {
        use crate::services::auth;
        let jar = CookieJar::new().add(Cookie::new(auth::OAUTH_STATE_COOKIE_NAME, "stored-state"));
        let result = google_callback(
            State(make_test_state()),
            HeaderMap::default(),
            Query(AuthCallbackQuery {
                code: "auth-code".to_string(),
                state: "different-state".to_string(),
            }),
            jar,
        )
        .await;
        let err = result.expect_err("state 不一致は拒否される");
        assert!(
            matches!(err, ApiError::Unauthorized(_)),
            "state 不一致は Unauthorized になるはず: {err:?}"
        );
    }

    #[test]
    fn test_oauth_state_equal() {
        assert!(oauth_state_equal("abc123", "abc123"));
        assert!(!oauth_state_equal("abc123", "abc124"));
        assert!(!oauth_state_equal("abc123", "abc12"));
        assert!(!oauth_state_equal("abc123", "abc1234"));
        assert!(!oauth_state_equal("", "a"));
    }

    #[test]
    fn test_jar_helpers() {
        use crate::services::auth;
        assert!(parse_session_id_from_jar(&CookieJar::new()).is_err());
        assert!(
            parse_session_id_from_jar(
                &CookieJar::new().add(Cookie::new(auth::SESSION_COOKIE_NAME, "invalid-uuid"))
            )
            .is_err()
        );
        let token = SessionToken::new();
        let jar = CookieJar::new().add(Cookie::new(auth::SESSION_COOKIE_NAME, token.to_string()));
        assert_eq!(parse_session_id_from_jar(&jar).unwrap(), token);
        assert_eq!(get_session_id_from_jar(&jar).unwrap(), token);
    }

    #[test]
    fn test_session_cookie_migration_prefers_new_cookie_and_accepts_old_cookie() {
        use crate::services::auth;

        let old_token = SessionToken::new();
        let new_token = SessionToken::new();
        let old_jar = CookieJar::new().add(Cookie::new(
            auth::LEGACY_SESSION_COOKIE_NAME,
            old_token.to_string(),
        ));
        assert_eq!(parse_session_id_from_jar(&old_jar).unwrap(), old_token);
        assert_eq!(session_cookie_needs_migration(&old_jar), Some(old_token));

        let both_jar = old_jar.add(Cookie::new(
            auth::SESSION_COOKIE_NAME,
            new_token.to_string(),
        ));
        assert_eq!(parse_session_id_from_jar(&both_jar).unwrap(), new_token);
        assert_eq!(session_cookie_needs_migration(&both_jar), None);
    }

    #[test]
    fn test_new_session_cookie_uses_host_prefix_requirements() {
        use crate::services::auth;

        let token = SessionToken::new();
        let cookie = build_session_cookie(token, true);

        assert_eq!(cookie.name(), auth::SESSION_COOKIE_NAME);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.domain(), None);
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.http_only(), Some(true));
        assert_eq!(cookie.same_site(), Some(SameSite::None));
        assert_eq!(cookie.partitioned(), None);
        assert_eq!(cookie.max_age(), Some(time::Duration::days(7)));
    }

    #[test]
    fn test_migrating_old_cookie_reissues_new_and_clears_old() {
        use crate::services::auth;

        let token = SessionToken::new();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            axum::http::header::COOKIE,
            format!("{}={token}", auth::LEGACY_SESSION_COOKIE_NAME)
                .parse()
                .unwrap(),
        );
        let jar = CookieJar::from_headers(&headers);
        let mut response = Response::new(axum::body::Body::empty());
        migrate_session_cookie(&mut response, &jar, true).unwrap();
        let set_cookies = response
            .headers()
            .get_all(axum::http::header::SET_COOKIE)
            .iter()
            .filter_map(|value| value.to_str().ok())
            .collect::<Vec<_>>();
        assert!(
            set_cookies
                .iter()
                .any(|value| value.starts_with("__Host-session_token=")
                    && value.contains(&token.to_string()))
        );
        assert!(
            set_cookies.iter().any(|value| {
                value.starts_with("session_token=") && value.contains("Max-Age=0")
            })
        );
    }

    #[test]
    fn test_device_cookie_helpers() {
        use crate::services::auth;
        for secure in [true, false] {
            let cookie = build_device_cookie(auth_service::DeviceToken::new(), secure);
            assert_eq!(
                (
                    cookie.name(),
                    cookie.path(),
                    cookie.http_only(),
                    cookie.secure(),
                    cookie.same_site(),
                    cookie.max_age(),
                ),
                (
                    auth::DEVICE_COOKIE_NAME,
                    Some("/"),
                    Some(true),
                    Some(secure),
                    Some(same_site(secure)),
                    Some(time::Duration::days(365)),
                )
            );
        }
        let missing = get_device_token_from_jar(&CookieJar::new());
        let invalid = get_device_token_from_jar(
            &CookieJar::new().add(Cookie::new(auth::DEVICE_COOKIE_NAME, "invalid-uuid")),
        );
        assert_ne!(missing, invalid);
        let token = auth_service::DeviceToken::new();
        assert_eq!(
            get_device_token_from_jar(
                &CookieJar::new().add(Cookie::new(auth::DEVICE_COOKIE_NAME, token.to_string()))
            ),
            token
        );
    }
}
