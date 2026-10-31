use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    handlers::v1::auth::oauth::get_device_token_from_jar,
    models::common::MessageResponse,
    services::{
        auth as auth_service,
        login_event::{self, DeviceListResponse, LoginHistoryListResponse, clamp_history_limit},
        login_notify,
    },
    state::AppState,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
}

/// 自分のログイン履歴を新しい順に返す（v1）
#[utoipa::path(
    get,
    path = "/api/v1/login-history",
    operation_id = "v1_login_history_list",
    params(
        ("limit" = Option<i64>, Query, description = "取得上限（既定: 50、最大: 100）"),
    ),
    responses(
        (status = 200, description = "自分のログイン履歴を新しい順に返す", body = LoginHistoryListResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list_history(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Query(query): Query<HistoryQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let entries = login_event::list_login_history(
        &state.pool,
        auth_user.id(),
        clamp_history_limit(query.limit),
    )
    .await?;
    Ok(Json(LoginHistoryListResponse { data: entries }))
}

/// 自分が使った端末の一覧を返す（v1）
#[utoipa::path(
    get,
    path = "/api/v1/devices",
    operation_id = "v1_device_list",
    responses(
        (status = 200, description = "自分が使った端末の一覧を返す", body = DeviceListResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list_devices(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    let current = get_device_token_from_jar(&jar);
    let devices = login_event::list_devices(&state.pool, auth_user.id(), &current.hash()).await?;
    Ok(Json(DeviceListResponse { data: devices }))
}

/// 端末を1台解除する（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/devices/{id}",
    operation_id = "v1_device_delete",
    params(
        ("id" = String, Path, description = "端末一覧で返した不透明な ID"),
    ),
    responses(
        (status = 200, description = "端末を解除しました", body = MessageResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
        (status = 404, description = "端末が見つからない", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_device(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    jar: CookieJar,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let device_hash = login_event::resolve_device_hash(&state.pool, auth_user.id(), &id)
        .await?
        .ok_or(ApiError::NotFound)?;
    auth_service::delete_device_sessions(&state.pool, auth_user.id(), &device_hash).await?;
    let current = get_device_token_from_jar(&jar);
    if current.hash() == device_hash {
        let jar = jar.remove(crate::handlers::v1::auth::oauth::clear_session_cookie(
            state.config.secure_cookie,
        ));
        return Ok((
            jar,
            Json(MessageResponse {
                message: "端末を解除しました".to_string(),
            }),
        ));
    }
    Ok((
        jar,
        Json(MessageResponse {
            message: "端末を解除しました".to_string(),
        }),
    ))
}

/// すべての端末をログアウトする（v1）
#[utoipa::path(
    delete,
    path = "/api/v1/sessions",
    operation_id = "v1_session_delete_all",
    responses(
        (status = 200, description = "すべての端末からログアウトしました", body = MessageResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn delete_all_sessions(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    jar: CookieJar,
) -> Result<impl IntoResponse, ApiError> {
    auth_service::delete_all_sessions(&state.pool, auth_user.id()).await?;
    let jar = jar.remove(crate::handlers::v1::auth::oauth::clear_session_cookie(
        state.config.secure_cookie,
    ));
    Ok((
        StatusCode::OK,
        jar,
        Json(MessageResponse {
            message: "すべての端末からログアウトしました".to_string(),
        }),
    ))
}

/// 自分のログイン通知設定を取得する（v1）
#[utoipa::path(
    get,
    path = "/api/v1/notification-settings",
    operation_id = "v1_notification_settings_get",
    responses(
        (status = 200, description = "自分のログイン通知設定を返す", body = login_notify::NotificationSettingsResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn get_notification_settings(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
) -> Result<impl IntoResponse, ApiError> {
    let enabled = login_notify::get_setting(&state.pool, auth_user.id()).await?;
    Ok(Json(login_notify::NotificationSettingsResponse {
        login_notify_enabled: enabled,
    }))
}

/// 自分のログイン通知設定を更新する（v1）
#[utoipa::path(
    put,
    path = "/api/v1/notification-settings",
    operation_id = "v1_notification_settings_put",
    request_body = login_notify::UpdateNotificationSettingsRequest,
    responses(
        (status = 200, description = "更新後のログイン通知設定を返す", body = login_notify::NotificationSettingsResponse),
        (status = 401, description = "認証が必要", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn put_notification_settings(
    State(state): State<AppState>,
    auth_user: AuthenticatedUser,
    Json(body): Json<login_notify::UpdateNotificationSettingsRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let enabled =
        login_notify::set_setting(&state.pool, auth_user.id(), body.login_notify_enabled).await?;
    Ok(Json(login_notify::NotificationSettingsResponse {
        login_notify_enabled: enabled,
    }))
}

#[derive(Debug, Deserialize)]
pub struct LogoutTokenQuery {
    pub token: Option<String>,
}

fn logout_token_value(token: Option<&str>) -> Result<&str, ApiError> {
    let token = token.unwrap_or_default().trim();
    if token.is_empty() || token.len() > 256 {
        return Err(ApiError::NotFound);
    }
    Ok(token)
}

/// 使い捨てリンクの有効性を確認する（v1、認証不要、GET では実行しない）
#[utoipa::path(
    get,
    path = "/api/v1/notification-logout",
    operation_id = "v1_notification_logout_info",
    params(
        ("token" = String, Query, description = "通知メール内の使い捨てトークン"),
    ),
    responses(
        (status = 200, description = "リンクが有効な場合の有効期限を返す", body = login_notify::LogoutTokenInfoResponse),
        (status = 404, description = "リンクが無効・期限切れ・使用済み", body = ErrorResponse),
    ),
)]
pub async fn logout_token_info(
    State(state): State<AppState>,
    Query(query): Query<LogoutTokenQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let token = logout_token_value(query.token.as_deref())?;
    let (_, expires_at) = login_notify::peek_token(&state.pool, token)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(login_notify::LogoutTokenInfoResponse { expires_at }))
}

/// 使い捨てリンクで全端末をログアウトする（v1、認証不要、確認画面からの POST のみ実行）
#[utoipa::path(
    post,
    path = "/api/v1/notification-logout",
    operation_id = "v1_notification_logout_execute",
    request_body = login_notify::LogoutWithTokenRequest,
    responses(
        (status = 200, description = "すべての端末からログアウトしました", body = MessageResponse),
        (status = 404, description = "リンクが無効・期限切れ・使用済み", body = ErrorResponse),
    ),
)]
pub async fn logout_with_token(
    State(state): State<AppState>,
    Json(body): Json<login_notify::LogoutWithTokenRequest>,
) -> Result<impl IntoResponse, ApiError> {
    let token = logout_token_value(Some(&body.token))?;
    let user_id = login_notify::consume_token(&state.pool, token)
        .await?
        .ok_or(ApiError::NotFound)?;
    auth_service::delete_all_sessions(&state.pool, user_id).await?;
    Ok(Json(MessageResponse {
        message: "すべての端末からログアウトしました".to_string(),
    }))
}
