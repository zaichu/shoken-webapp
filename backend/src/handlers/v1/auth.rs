use crate::{
    config,
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    handlers::auth::same_site,
    models::{common::MessageResponse, user::UserResponse},
    services::auth as auth_service,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use hmac::{Hmac, Mac};
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
        .map_err(|_| ApiError::ApiError("アカウント削除確認の生成に失敗しました".to_string()))?;
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
        .map(|mac| mac.verify_slice(&tag).is_ok())
        .unwrap_or(false)
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
    crate::handlers::auth::get_current_user(State(state), jar).await
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
    crate::handlers::auth::logout(State(state), jar).await
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
    let session_id = crate::handlers::auth::get_session_id_from_jar(&jar)?;

    let confirmation = jar
        .get(ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME)
        .ok_or_else(|| ApiError::ApiError("アカウント削除確認が完了していません".to_string()))?;
    if !verify_account_delete_confirmation(
        confirmation.value(),
        &session_id.to_string(),
        chrono::Utc::now().timestamp(),
    ) {
        return Err(ApiError::ApiError(
            "アカウント削除確認が無効です".to_string(),
        ));
    }

    let user_id = auth_service::select_user_id_by_session(&state.pool, session_id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("セッションが無効または期限切れです".to_string()))?;

    auth_service::delete_account(&state.pool, user_id).await?;

    let is_secure = config::is_secure_cookie();
    let jar = jar
        .remove(crate::handlers::auth::clear_session_cookie(is_secure))
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
    auth_user: AuthenticatedUser,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    let _ = auth_user;
    let session_id = crate::handlers::auth::get_session_id_from_jar(&jar)?;
    let confirmation =
        issue_account_delete_confirmation(&session_id.to_string(), chrono::Utc::now().timestamp())?;
    let is_secure = config::is_secure_cookie();
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
    crate::handlers::auth::google_auth(State(state), jar).await
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
    Query(query): Query<crate::handlers::auth::AuthCallbackQuery>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::auth::google_callback(State(state), Query(query), jar).await
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
        }
    }

    fn test_user() -> User {
        User {
            id: uuid::Uuid::new_v4(),
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

        let expired = issue_account_delete_confirmation(
            &session,
            now - ACCOUNT_DELETE_CONFIRMATION_TTL_SECONDS - 1,
        )
        .unwrap();
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
        let response = create_account_deletion_confirmation(AuthenticatedUser(test_user()), jar)
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
            matches!(err, ApiError::DatabaseError(_)),
            "有効な確認は削除処理へ進むこと: {err:?}"
        );

        let expired = issue_account_delete_confirmation(
            &session_id.to_string(),
            chrono::Utc::now().timestamp() - ACCOUNT_DELETE_CONFIRMATION_TTL_SECONDS - 1,
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
                matches!(err, ApiError::ApiError(ref msg) if msg.contains("無効")),
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
            matches!(err, ApiError::ApiError(ref msg) if msg.contains("完了していません")),
            "Cookie 未提示: {err:?}"
        );
    }
}
