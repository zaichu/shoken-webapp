use crate::db::DbError;
use crate::errors::ApiError;
use crate::models::user::User;
use crate::services::auth::{self as auth_service, SessionToken};
use crate::state::AppState;
use axum::extract::FromRef;
use axum_extra::extract::CookieJar;
use shared::value::UserId;

fn session_lookup_error(error: DbError) -> ApiError {
    let error_kind = match &error {
        DbError::Pg(_) => "postgres",
        DbError::RowNotFound => "row_not_found",
        DbError::Other(_) => "other",
        DbError::State { .. } => "sql_state",
        DbError::TransactionUnavailable => "transaction_unavailable",
    };
    tracing::warn!(error_kind, "session lookup failed");
    ApiError::Database(error)
}

/// 認証済みユーザーを表すエクストラクター
/// ハンドラーの引数に指定することで、認証チェックを自動的に行う
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub User);

impl AuthenticatedUser {
    pub const fn id(&self) -> UserId {
        self.0.id
    }
}

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::Unauthorized("Cookieの取得に失敗しました"))?;

        let cookie = jar
            .get(auth_service::SESSION_COOKIE_NAME)
            .ok_or_else(|| ApiError::Unauthorized("ログインが必要です"))?;
        let token: SessionToken = cookie
            .value()
            .parse()
            .map_err(|_| ApiError::Unauthorized("無効なセッショントークンです"))?;
        // セッションテーブルからユーザーを取得（期限切れでないセッションのみ）
        let user = auth_service::select_user_by_session(&app_state.pool, token)
            .await
            .map_err(session_lookup_error)?
            .ok_or(ApiError::Unauthorized("セッションが無効または期限切れです"))?;

        Ok(Self(user))
    }
}
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use {
        super::AuthenticatedUser,
        crate::{
            config::Config,
            db,
            errors::ApiError,
            models::user::User,
            services::auth,
            services::auth::SessionToken,
            state::{AppState, Secrets},
        },
        axum::extract::FromRequestParts,
        axum::http::StatusCode,
        shared::value::UserId,
        std::sync::Arc,
        time::OffsetDateTime,
    };

    #[tokio::test]
    async fn session_lookup_database_error_is_not_unauthorized() {
        let pool = db::connect_pool_lazy(
            "postgresql://user:password@127.0.0.1:1/test_db?connect_timeout=1",
            1,
        )
        .expect("pool should be created lazily");
        let state = AppState {
            pool,
            secrets: Arc::new(Secrets {
                jquants_api_key: None,
                google_client_id: None,
                google_client_secret: None,
                frontend_url: "http://localhost".to_string(),
                gmail_user: None,
                gmail_app_password: None,
            }),
            config: Arc::new(Config::default()),
            google_oauth: None,
            jquants_client: None,
        };
        let request = axum::http::Request::builder()
            .header(
                axum::http::header::COOKIE,
                format!("{}={}", auth::SESSION_COOKIE_NAME, SessionToken::default()),
            )
            .body(())
            .expect("request should be built");
        let (mut parts, ()) = request.into_parts();

        let error = AuthenticatedUser::from_request_parts(&mut parts, &state)
            .await
            .expect_err("database connection failure should reject extraction");

        assert!(matches!(error, ApiError::Database(_)));
        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn id_returns_wrapped_user_id() {
        let user = User {
            id: UserId::from(uuid::Uuid::new_v4()),
            google_id: "google-123".to_string(),
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            picture_url: None,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };
        assert_eq!(AuthenticatedUser(user.clone()).id(), user.id);
    }
}
