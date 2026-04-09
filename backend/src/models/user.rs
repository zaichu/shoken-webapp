use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

/// データベースのユーザーモデル
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub google_id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// APIレスポンス用のユーザー情報
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub picture_url: Option<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.to_string(),
            email: user.email,
            name: user.name,
            picture_url: user.picture_url,
        }
    }
}

/// Google OAuthから取得するユーザー情報
#[derive(Debug, Clone, Deserialize)]
pub struct GoogleUserInfo {
    pub sub: String, // Google ID
    pub email: String,
    pub name: Option<String>,
    pub picture: Option<String>,
}

#[cfg(test)] #[rustfmt::skip] mod tests {
    use super::*;
    #[test] fn test_user_response_from_user() { let user = User { id: Uuid::new_v4(), google_id: "google123".to_string(), email: "test@example.com".to_string(), name: Some("テストユーザー".to_string()), picture_url: Some("https://example.com/photo.jpg".to_string()), created_at: Utc::now(), updated_at: Utc::now() }; let response: UserResponse = user.clone().into(); assert_eq!((response.id, response.email, response.name, response.picture_url), (user.id.to_string(), user.email, user.name, user.picture_url)); }
}
