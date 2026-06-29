use crate::errors::ApiError;
use crate::models::user::User;
use crate::services::auth as auth_service;
use crate::state::AppState;
use axum::extract::FromRef;
use axum_extra::extract::CookieJar;

/// 認証済みユーザーを表すエクストラクター
/// ハンドラーの引数に指定することで、認証チェックを自動的に行う
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub User);

impl AuthenticatedUser {
    /// ユーザーIDを取得
    pub fn id(&self) -> uuid::Uuid {
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

        // CookieJarを取得
        let jar = CookieJar::from_request_parts(parts, state)
            .await
            .map_err(|_| ApiError::Unauthorized("Cookieの取得に失敗しました".to_string()))?;

        // セッショントークンを取得してセッションIDにパース
        let session_token = jar
            .get(auth_service::SESSION_COOKIE_NAME)
            .map(|c| c.value().to_string())
            .ok_or_else(|| ApiError::Unauthorized("ログインが必要です".to_string()))?;
        let session_id: uuid::Uuid = session_token
            .parse()
            .map_err(|_| ApiError::Unauthorized("無効なセッショントークンです".to_string()))?;
        // セッションテーブルからユーザーを取得（期限切れでないセッションのみ）
        let user = auth_service::select_user_by_session(&app_state.pool, session_id)
            .await
            .map_err(|_| ApiError::Unauthorized("セッション検証に失敗しました".to_string()))?
            .ok_or_else(|| {
                ApiError::Unauthorized("セッションが無効または期限切れです".to_string())
            })?;

        Ok(AuthenticatedUser(user))
    }
}
#[cfg(test)]
mod tests {
    use {super::AuthenticatedUser, crate::models::user::User, chrono::Utc, uuid::Uuid};
    #[test]
    fn id_returns_wrapped_user_id() {
        let user = User {
            id: Uuid::new_v4(),
            google_id: "google-123".to_string(),
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
            picture_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        assert_eq!(AuthenticatedUser(user.clone()).id(), user.id);
    }
}
