use crate::{
    errors::{ApiError, ErrorResponse},
    extractors::auth::AuthenticatedUser,
    services::login_event::admin::{self, AdminLoginListResponse, AdminLoginQuery},
    state::AppState,
};
use axum::{
    Json,
    extract::{Query, State},
};

#[utoipa::path(
    get,
    path = "/api/v1/admin/logins",
    operation_id = "v1_admin_logins_list",
    params(
        ("page" = Option<i64>, Query, description = "ページ番号（既定: 1）"),
        ("per_page" = Option<i64>, Query, description = "取得件数（既定: 200、最大: 1000）"),
        ("q" = Option<String>, Query, description = "メール・場所の部分一致（空白区切り AND）"),
        ("email" = Option<String>, Query, description = "メールの部分一致"),
        ("place" = Option<String>, Query, description = "国・地域の部分一致"),
        ("sort_by" = Option<String>, Query, description = "occurred_at / email / result / place（既定: occurred_at）"),
        ("sort_order" = Option<String>, Query, description = "asc / desc（既定: desc）"),
    ),
    responses(
        (status = 200, description = "全利用者のログイン記録。閲覧者と日時を監査表に記録する。IP・端末識別子・財産のデータは含まない", body = AdminLoginListResponse),
        (status = 400, description = "検索条件が不正", body = ErrorResponse),
        (status = 403, description = "管理者権限が必要（未認証・ADMIN_EMAILS 未設定を含む）", body = ErrorResponse),
        (status = 500, description = "サーバーエラー", body = ErrorResponse),
    ),
    security(("cookieAuth" = []))
)]
pub async fn list(
    State(state): State<AppState>,
    auth_user: Result<AuthenticatedUser, ApiError>,
    query: Result<Query<AdminLoginQuery>, axum::extract::rejection::QueryRejection>,
) -> Result<Json<AdminLoginListResponse>, ApiError> {
    let auth_user = auth_user.map_err(|_| ApiError::Forbidden)?;
    if !state.config.is_admin_email(&auth_user.0.email) {
        return Err(ApiError::Forbidden);
    }
    admin::record_view(&state.pool, auth_user.id()).await?;
    let Query(query) = query.map_err(|_| ApiError::Validation("検索条件が不正です".into()))?;
    Ok(Json(admin::search(&state.pool, query).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, db::connect_pool_lazy, models::user::User, state::Secrets};
    use axum::{http::StatusCode, response::IntoResponse};
    use std::sync::Arc;

    #[tokio::test]
    async fn admin_logins_checks_permission_before_accessing_database() {
        for (emails, email, expected) in [
            (None, "admin@example.com", StatusCode::FORBIDDEN),
            (Some(""), "admin@example.com", StatusCode::FORBIDDEN),
            (
                Some("admin@example.com"),
                "other@example.com",
                StatusCode::FORBIDDEN,
            ),
            (
                Some("admin@example.com"),
                "admin@example.com.evil",
                StatusCode::FORBIDDEN,
            ),
            (
                Some("ADMIN@EXAMPLE.COM"),
                "admin@example.com",
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
            (
                Some("admin@example.com"),
                "ADMIN@EXAMPLE.COM",
                StatusCode::INTERNAL_SERVER_ERROR,
            ),
        ] {
            let pool = connect_pool_lazy("postgres://user:password@localhost/test_db", 1).unwrap();
            pool.close().await;
            let state = AppState {
                pool,
                secrets: Arc::new(Secrets {
                    jquants_api_key: None,
                    google_client_id: None,
                    google_client_secret: None,
                    frontend_url: "http://localhost:8080".into(),
                }),
                config: Arc::new(Config {
                    admin_emails: emails.map(str::to_owned),
                    ..Config::default()
                }),
                google_oauth: None,
                jquants_client: None,
            };
            let user = User {
                id: uuid::Uuid::new_v4().into(),
                google_id: "admin-test".into(),
                email: email.into(),
                name: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            let response = list(
                State(state),
                Ok(AuthenticatedUser(user)),
                Ok(Query(AdminLoginQuery::default())),
            )
            .await
            .into_response();
            assert_eq!(response.status(), expected, "{emails:?}: {email}");
        }
    }
}
