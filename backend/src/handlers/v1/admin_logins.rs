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
    let Query(query) = query.map_err(|_| ApiError::Validation("検索条件が不正です".into()))?;
    admin::record_view(&state.pool, auth_user.id()).await?;
    Ok(Json(admin::search(&state.pool, query).await?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::Config, db::connect_pool_lazy, models::user::User, state::Secrets};
    use axum::{http::StatusCode, response::IntoResponse};
    use std::sync::Arc;

    #[tokio::test]
    #[ignore = "requires Docker to run Postgres container"]
    async fn successful_api_call_records_viewer_and_denied_calls_do_not() {
        use crate::{
            db::{self, Bind},
            models::user::GoogleUserInfo,
            services::auth,
        };
        use axum::{
            body::{Body, to_bytes},
            http::Request,
        };
        use tower::ServiceExt;
        let (pool, _node) = crate::test_db::start_test_pool().await;
        let token = auth::upsert_user_and_rotate_session(
            &pool,
            &GoogleUserInfo {
                sub: "admin-api".into(),
                email: "admin@example.com".into(),
                name: None,
            },
        )
        .await
        .unwrap();
        let user_id = auth::select_user_id_by_session(&pool, token)
            .await
            .unwrap()
            .unwrap();
        for (emails, expected) in [
            (None, StatusCode::FORBIDDEN),
            (Some("other@example.com"), StatusCode::FORBIDDEN),
            (Some("ADMIN@EXAMPLE.COM"), StatusCode::OK),
        ] {
            let state = AppState {
                pool: pool.clone(),
                secrets: Arc::new(Secrets {
                    jquants_api_key: None,
                    google_client_id: None,
                    google_client_secret: None,
                    frontend_url: "http://localhost:8080".into(),
                }),
                config: Arc::new(Config {
                    admin_emails: emails.map(str::to_owned),
                    ..Default::default()
                }),
                google_oauth: None,
                jquants_client: None,
            };
            let response = crate::handlers::v1::data_routes().with_state(state).oneshot(Request::builder()
                .uri("/api/v1/admin/logins?per_page=5000&page=0&email=admin&place=Tokyo&sort_by=email&sort_order=asc")
                .header("cookie", format!("session_token={token}"))
                .body(Body::empty()).unwrap()).await.unwrap();
            assert_eq!(response.status(), expected);
            if expected == StatusCode::OK {
                let body: serde_json::Value =
                    serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap())
                        .unwrap();
                assert_eq!(
                    (body["page"].as_i64(), body["per_page"].as_i64()),
                    (Some(1), Some(1000))
                );
                assert!(body["data"].as_array().unwrap().is_empty());
            }
        }
        let recorded: i64 = db::query_scalar(
            "SELECT COUNT(*) FROM admin_login_views WHERE user_id = $1 AND occurred_at <= NOW()",
            vec![Bind::from(user_id)],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(recorded, 1);
    }

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
