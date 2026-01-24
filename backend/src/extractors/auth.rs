use crate::errors::ApiError;
use crate::models::user::User;
use crate::state::AppState;
use axum::extract::FromRef;
use axum_extra::extract::CookieJar;

const SESSION_COOKIE_NAME: &str = "session_token";

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

        // セッショントークンを取得
        let session_token = jar
            .get(SESSION_COOKIE_NAME)
            .map(|c| c.value().to_string())
            .ok_or_else(|| ApiError::Unauthorized("ログインが必要です".to_string()))?;

        // セッションIDをパース
        let session_id: uuid::Uuid = session_token
            .parse()
            .map_err(|_| ApiError::Unauthorized("無効なセッショントークンです".to_string()))?;

        // セッションテーブルからユーザーを取得（期限切れでないセッションのみ）
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT u.id, u.google_id, u.email, u.name, u.picture_url, u.created_at, u.updated_at
            FROM users u
            INNER JOIN sessions s ON u.id = s.user_id
            WHERE s.id = $1 AND s.expires_at > NOW()
            "#,
        )
        .bind(session_id)
        .fetch_optional(&app_state.pool)
        .await
        .map_err(|_| ApiError::Unauthorized("セッション検証に失敗しました".to_string()))?
        .ok_or_else(|| ApiError::Unauthorized("セッションが無効または期限切れです".to_string()))?;

        Ok(AuthenticatedUser(user))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compilation() {
        // モジュールが正常にコンパイルされることを確認
        assert!(true);
    }
}
