use crate::{
    config,
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    models::{common::MessageResponse, user::UserResponse},
    services::auth as auth_service,
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};

const ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME: &str = "account_delete_confirmation";

fn same_site(secure: bool) -> SameSite {
    if secure {
        SameSite::None
    } else {
        SameSite::Lax
    }
}

fn build_account_delete_confirmation_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((
        ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME,
        uuid::Uuid::new_v4().to_string(),
    ))
    .path("/api/v1/account")
    .http_only(true)
    .secure(secure)
    .same_site(same_site(secure))
    .max_age(time::Duration::minutes(10))
    .build()
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

// Session / account handlers
// ---------------------------------------------------------------------------

/// 現在ログイン中のユーザー情報を取得（v1）
#[utoipa::path(
    get,
    path = "/api/v1/session",
    operation_id = "v1_get_session",
    responses(
        (status = 200, body = UserResponse),
        (status = 401, body = ErrorResponse),
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
        (status = 200, body = MessageResponse),
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
        (status = 200, body = MessageResponse),
        (status = 400, body = ErrorResponse),
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

    let confirmation = jar
        .get(ACCOUNT_DELETE_CONFIRMATION_COOKIE_NAME)
        .ok_or_else(|| ApiError::ApiError("アカウント削除確認が完了していません".to_string()))?;
    confirmation
        .value()
        .parse::<uuid::Uuid>()
        .map_err(|_| ApiError::ApiError("アカウント削除確認が無効です".to_string()))?;

    let user_id = auth_service::select_user_id_by_session(&state.pool, session_id)
        .await?
        .ok_or_else(|| ApiError::Unauthorized("セッションが無効または期限切れです".to_string()))?;

    auth_service::delete_account(&state.pool, user_id).await?;

    let is_secure = config::is_secure_cookie();
    let jar = jar
        .remove(auth_service::clear_session_cookie(is_secure))
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
        (status = 200, body = MessageResponse),
        (status = 401, body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn create_account_deletion_confirmation(
    auth_user: AuthenticatedUser,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    let _ = auth_user;
    let is_secure = config::is_secure_cookie();
    let jar = jar.add(build_account_delete_confirmation_cookie(is_secure));

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
        (status = 500, body = ErrorResponse),
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
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse),
    ),
)]
pub async fn google_callback(
    State(state): State<AppState>,
    Query(query): Query<crate::handlers::auth::AuthCallbackQuery>,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    crate::handlers::auth::google_callback(State(state), Query(query), jar).await
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_delete_confirmation_cookie_is_http_only_and_scoped() {
        for secure in [true, false] {
            let cookie = build_account_delete_confirmation_cookie(secure);
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
}
