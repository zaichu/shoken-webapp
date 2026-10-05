pub mod oauth;

use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    handlers::v1::auth::oauth::{clear_session_cookie, get_session_id_from_jar, same_site},
    models::{common::MessageResponse, user::UserResponse},
    services::auth::{self as auth_service, SessionToken},
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use hmac::{digest::KeyInit, Hmac, Mac};
use sha2::Sha256;

const ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME: &str = "account_delete_confirmation";
const ACCOUNT_DELETE_CONFIRMATION_TTL_SECONDS: i64 = 10 * 60;

fn build_account_delete_confirmation_cookie(value: String, secure: bool) -> Cookie<'static> {
    Cookie::build((ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME, value))
        .path("/api/v1/account")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::seconds(
            ACCOUNT_DELETE_CONFIRMATION_TTL_SECONDS,
        ))
        .build()
}

fn account_delete_confirmation_mac(
    session_token: &str,
    issued_at: i64,
    nonce: &str,
) -> Result<Hmac<Sha256>, ApiError> {
    let mut mac = Hmac::<Sha256>::new_from_slice(session_token.as_bytes())
        .map_err(|_| ApiError::Internal("アカウント削除確認の生成に失敗しました"))?;
    mac.update(format!("account-delete:{issued_at}:{nonce}").as_bytes());
    Ok(mac)
}

fn issue_account_delete_confirmation(
    session_token: &str,
    issued_at: i64,
) -> Result<String, ApiError> {
    let nonce = uuid::Uuid::new_v4().simple().to_string();
    let tag = account_delete_confirmation_mac(session_token, issued_at, &nonce)?
        .finalize()
        .into_bytes();
    Ok(format!("{issued_at}.{nonce}.{}", hex::encode(tag)))
}

fn verify_account_delete_confirmation(value: &str, session_token: &str, now: i64) -> bool {
    let mut parts = value.splitn(3, '.');
    let (Some(issued_at), Some(nonce), Some(tag)) = (parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    let Ok(issued_at) = issued_at.parse::<i64>() else {
        return false;
    };
    if !(0..=ACCOUNT_DELETE_CONFIRMATION_TTL_SECONDS).contains(&(now - issued_at)) {
        return false;
    }
    let Ok(tag) = hex::decode(tag) else {
        return false;
    };
    account_delete_confirmation_mac(session_token, issued_at, nonce)
        .is_ok_and(|mac| mac.verify_slice(&tag).is_ok())
}

fn clear_account_delete_confirmation_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME, ""))
        .path("/api/v1/account")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::seconds(0))
        .build()
}

/// 現在ログイン中のユーザー情報を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/session",
    operation_id = "v1_get_session",
    responses(
        (status = 200, description = "現在のユーザー情報を返す", body = UserResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn get_session(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<UserResponse>, ApiError> {
    let token = get_session_id_from_jar(&jar)?;

    // セッションテーブルからユーザーを取得（期限切れでないセッションのみ）
    let user = auth_service::select_user_by_session(&state.pool, token)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("セッションが無効または期限切れです"))?;

    Ok(Json(user.into()))
}

/// セッションを削除してログアウト（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/session",
    operation_id = "v1_delete_session",
    responses(
        (status = 200, description = "ログアウトしました", body = MessageResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_session(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    if let Some(token) = jar
        .get(auth_service::SESSION_COOKIE_NAME)
        .and_then(|c| c.value().parse::<SessionToken>().ok())
    {
        let _ = auth_service::delete_session(&state.pool, token).await;
    }

    let cookie = clear_session_cookie(state.config.secure_cookie);

    let jar = jar.remove(cookie);

    (
        jar,
        Json(serde_json::json!({"message": "ログアウトしました"})),
    )
}

/// アカウントを削除（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/account",
    operation_id = "v1_delete_account",
    responses(
        (status = 200, description = "アカウントを削除しました", body = MessageResponse),
        (status = 400, description = "削除の確認が取れていない", body = ErrorResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_account(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = crate::handlers::v1::auth::oauth::get_session_id_from_jar(&jar)?;

    let confirmation = jar
        .get(ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME)
        .ok_or_else(|| ApiError::Validation("アカウント削除確認が完了していません".to_string()))?;
    if !verify_account_delete_confirmation(
        confirmation.value(),
        &session_id.to_string(),
        chrono::Utc::now().timestamp(),
    ) {
        return Err(ApiError::Validation(
            "アカウント削除確認が無効です".to_string(),
        ));
    }

    let user_id = auth_service::select_user_id_by_session(&state.pool, session_id)
        .await?
        .ok_or(ApiError::Unauthorized("セッションが無効または期限切れです"))?;

    auth_service::delete_account(&state.pool, user_id).await?;

    let is_secure = state.config.secure_cookie;
    let jar = jar
        .remove(crate::handlers::v1::auth::oauth::clear_session_cookie(
            is_secure,
        ))
        .remove(clear_account_delete_confirmation_cookie(is_secure));

    Ok((
        jar,
        Json(MessageResponse {
            message: "アカウントを削除しました".to_string(),
        }),
    ))
}

/// アカウント削除確認を開始（v1）
#[utoipa::path(
    post,
    path = "/api/v1/account-deletion-confirmations",
    operation_id = "v1_create_account_deletion_confirmation",
    responses(
        (status = 200, description = "削除確認の Cookie を発行しました", body = MessageResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn create_account_deletion_confirmation(
    State(state): State<AppState>,
    _auth_user: AuthenticatedUser,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    let session_id = crate::handlers::v1::auth::oauth::get_session_id_from_jar(&jar)?;
    let confirmation =
        issue_account_delete_confirmation(&session_id.to_string(), chrono::Utc::now().timestamp())?;
    let is_secure = state.config.secure_cookie;
    let jar = jar.add(build_account_delete_confirmation_cookie(
        confirmation,
        is_secure,
    ));

    Ok((
        jar,
        (
            StatusCode::OK,
            Json(MessageResponse {
                message: "アカウント削除確認を開始しました".to_string(),
            }),
        ),
    ))
}

/// Google OAuth 認証を開始（v1）
#[utoipa::path(
    get,
    path = "/api/v1/oauth/google/authorize",
    operation_id = "v1_google_authorize",
    responses(
        (status = 302, description = "Google OAuth 認証ページへリダイレクト"),
        (status = 500, description = "サーバーエラー", body = ErrorResponse),
    ),
)]
pub async fn google_authorize(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::v1::auth::oauth::google_auth(State(state), jar).await
}

/// Google OAuth コールバックを処理（v1）
#[utoipa::path(
    get,
    path = "/api/v1/oauth/google/callback",
    operation_id = "v1_google_callback",
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
    Query(query): Query<crate::handlers::v1::auth::oauth::AuthCallbackQuery>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::v1::auth::oauth::google_callback(State(state), Query(query), jar).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db::connect_pool_lazy,
        extractors::auth::AuthenticatedUser,
        models::user::User,
        state::{AppState, Secrets},
    };
    use axum_extra::extract::cookie::SameSite;
    use std::sync::Arc;

    fn make_test_state() -> AppState {
        let database_url = "postgresql://user:password@localhost/test_db";
        AppState {
            pool: connect_pool_lazy(database_url, 1).expect("pool"),
            secrets: Arc::new(Secrets {
                database_url: database_url.to_string(),
                jquants_api_key: None,
                google_client_id: None,
                google_client_secret: None,
                frontend_url: "http://localhost:8080".to_string(),
            }),
            client: reqwest::Client::new(),
            dividend_cache: crate::state::DividendCacheState::default(),
            config: Arc::new(crate::config::Config::default()),
            google_oauth: None,
        }
    }

    fn test_user() -> User {
        User {
            id: shared::value::UserId::from(uuid::Uuid::new_v4()),
            google_id: "google-123".to_string(),
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            picture_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }
    }

    fn tampered_value(value: &str) -> String {
        let (head, tag) = value.rsplit_once('.').unwrap();
        let mut tag = tag.as_bytes().to_vec();
        tag[0] = if tag[0] == b'0' { b'1' } else { b'0' };
        format!("{head}.{}", std::str::from_utf8(&tag).unwrap())
    }

    #[tokio::test]
    async fn test_session_handlers_preserve_errors_and_logout_cookie() {
        use axum::{
            body::{to_bytes, Body},
            http::{header::SET_COOKIE, Method, Request},
        };
        use tower::ServiceExt;

        for secure in [true, false] {
            let mut state = make_test_state();
            state.config = Arc::new(crate::config::Config {
                secure_cookie: secure,
                ..crate::config::Config::default()
            });
            state.pool.close().await;
            let app = crate::handlers::v1::auth_routes().with_state(state);
            for (token, status, message) in [
                (None, StatusCode::UNAUTHORIZED, "ログインが必要です"),
                (
                    Some("invalid-uuid"),
                    StatusCode::UNAUTHORIZED,
                    "無効なセッショントークンです",
                ),
                (
                    Some("550e8400-e29b-41d4-a716-446655440000"),
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "",
                ),
            ] {
                for method in [Method::GET, Method::DELETE] {
                    let mut request = Request::builder()
                        .method(method.clone())
                        .uri("/api/v1/session");
                    if let Some(token) = token {
                        request = request.header("cookie", format!("session_token={token}"));
                    }
                    let response = app
                        .clone()
                        .oneshot(request.body(Body::empty()).unwrap())
                        .await
                        .unwrap();
                    if method == Method::DELETE {
                        assert_eq!(response.status(), StatusCode::OK);
                        if token.is_some() {
                            let cookie =
                                Cookie::parse(response.headers()[SET_COOKIE].to_str().unwrap())
                                    .unwrap();
                            assert_eq!(cookie.name(), auth_service::SESSION_COOKIE_NAME);
                            assert_eq!(cookie.path(), Some("/"));
                            assert_eq!(cookie.http_only(), Some(true));
                            assert_eq!(cookie.secure().unwrap_or(false), secure);
                            assert_eq!(cookie.same_site(), Some(same_site(secure)));
                            assert_eq!(cookie.max_age(), Some(time::Duration::ZERO));
                        }
                        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
                        assert_eq!(
                            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
                            serde_json::json!({"message": "ログアウトしました"})
                        );
                    } else {
                        assert_eq!(response.status(), status);
                        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
                        let error: ErrorResponse = serde_json::from_slice(&body).unwrap();
                        if status == StatusCode::UNAUTHORIZED {
                            assert_eq!(error.error.code, "UNAUTHORIZED");
                            assert_eq!(error.error.message, message);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_account_delete_confirmation_cookie_is_http_only_and_scoped() {
        for secure in [true, false] {
            let cookie = build_account_delete_confirmation_cookie(String::new(), secure);
            assert_eq!(cookie.name(), ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME);
            assert_eq!(cookie.path(), Some("/api/v1/account"));
            assert_eq!(cookie.http_only(), Some(true));
            assert_eq!(cookie.secure(), Some(secure));
            assert_eq!(
                cookie.same_site(),
                Some(if secure {
                    SameSite::None
                } else {
                    SameSite::Lax
                })
            );
        }
    }

    #[test]
    fn test_account_delete_confirmation_issue_and_verify() {
        let session = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();

        let value = issue_account_delete_confirmation(&session, now).unwrap();
        assert!(
            verify_account_delete_confirmation(&value, &session, now),
            "発行した確認は同じセッション・期限内で受理されること"
        );

        let just_valid = issue_account_delete_confirmation(&session, now - 599).unwrap();
        assert!(
            verify_account_delete_confirmation(&just_valid, &session, now),
            "TTL 直前の確認は受理されること"
        );
        let at_boundary = issue_account_delete_confirmation(&session, now - 600).unwrap();
        assert!(
            verify_account_delete_confirmation(&at_boundary, &session, now),
            "TTL ちょうどの確認は受理されること"
        );
        let expired = issue_account_delete_confirmation(&session, now - 601).unwrap();
        assert!(
            !verify_account_delete_confirmation(&expired, &session, now),
            "期限切れの確認は拒否されること"
        );
        let future = issue_account_delete_confirmation(&session, now + 1).unwrap();
        assert!(
            !verify_account_delete_confirmation(&future, &session, now),
            "未来に発行された確認は拒否されること"
        );

        assert!(
            !verify_account_delete_confirmation(&tampered_value(&value), &session, now),
            "署名を改ざんした確認は拒否されること"
        );

        assert!(
            !verify_account_delete_confirmation(&value, &uuid::Uuid::new_v4().to_string(), now),
            "別セッションでは受理されないこと"
        );

        for bad in [
            "",
            "garbage",
            "1.2",
            "not-a-timestamp.nonce.00",
            "1.nonce.not-hex",
        ] {
            assert!(
                !verify_account_delete_confirmation(bad, &session, now),
                "不正な形式 {bad:?} は拒否されること"
            );
        }
    }

    #[tokio::test]
    async fn test_account_delete_confirmation_handler_lifecycle() {
        use axum::http::header::SET_COOKIE;

        let session_id = uuid::Uuid::new_v4();
        let jar = CookieJar::new().add(Cookie::new(
            auth_service::SESSION_COOKIE_NAME,
            session_id.to_string(),
        ));
        let response = create_account_deletion_confirmation(
            State(make_test_state()),
            AuthenticatedUser(test_user()),
            jar,
        )
        .await
        .expect("確認発行が失敗しないこと")
        .into_response();
        let set_cookie = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|c| c.starts_with(ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME))
            .expect("確認 Cookie が発行されること");
        assert!(set_cookie.contains("HttpOnly"));
        assert!(set_cookie.contains("Path=/api/v1/account"));
        let value = set_cookie
            .split(';')
            .next()
            .and_then(|kv| kv.split_once('='))
            .map(|(_, v)| v.to_string())
            .expect("Cookie の値を取り出せること");
        assert!(
            verify_account_delete_confirmation(
                &value,
                &session_id.to_string(),
                chrono::Utc::now().timestamp()
            ),
            "発行された Cookie は発行元セッションで受理されること"
        );

        let jar_with = |value: &str| {
            CookieJar::new()
                .add(Cookie::new(
                    auth_service::SESSION_COOKIE_NAME,
                    session_id.to_string(),
                ))
                .add(Cookie::new(
                    ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME,
                    value.to_string(),
                ))
        };
        // 有効な確認はガードを通過し、後段のDB処理へ進む(閉じたpoolのため即座に失敗する)
        let state = make_test_state();
        state.pool.close().await;
        let err = delete_account(State(state), jar_with(&value))
            .await
            .err()
            .expect("閉じたpoolのため削除は失敗する");
        assert!(
            matches!(err, ApiError::Database(_)),
            "有効な確認は削除処理へ進むこと: {err:?}"
        );

        let expired = issue_account_delete_confirmation(
            &session_id.to_string(),
            chrono::Utc::now().timestamp() - 601,
        )
        .unwrap();
        let other_session = issue_account_delete_confirmation(
            &uuid::Uuid::new_v4().to_string(),
            chrono::Utc::now().timestamp(),
        )
        .unwrap();
        for (case, cookie_value) in [
            ("改ざん", tampered_value(&value)),
            ("期限切れ", expired),
            ("別セッション", other_session),
        ] {
            let err = delete_account(State(make_test_state()), jar_with(&cookie_value))
                .await
                .err()
                .expect("無効な確認は拒否されること");
            assert!(
                matches!(err, ApiError::Validation(ref msg) if msg.contains("無効")),
                "{case}: {err:?}"
            );
        }

        let jar_no_confirmation = CookieJar::new().add(Cookie::new(
            auth_service::SESSION_COOKIE_NAME,
            session_id.to_string(),
        ));
        let err = delete_account(State(make_test_state()), jar_no_confirmation)
            .await
            .err()
            .expect("確認 Cookie なしは拒否されること");
        assert!(
            matches!(err, ApiError::Validation(ref msg) if msg.contains("完了していません")),
            "Cookie 未提示: {err:?}"
        );
    }
}
