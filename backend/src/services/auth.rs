use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use sqlx::PgPool;

use crate::errors::ApiError;
use crate::models::user::{GoogleUserInfo, User};

pub const SESSION_COOKIE_NAME: &str = "session_token";
pub const OAUTH_STATE_COOKIE_NAME: &str = "oauth_state";

fn same_site(secure: bool) -> SameSite {
    if secure {
        SameSite::None
    } else {
        SameSite::Lax
    }
}

pub fn get_session_id_from_jar(jar: &CookieJar) -> Result<uuid::Uuid, ApiError> {
    let session_token = jar
        .get(SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or_else(|| ApiError::Unauthorized("ログインが必要です".to_string()))?;

    let session_id: uuid::Uuid = session_token
        .parse()
        .map_err(|_| ApiError::Unauthorized("無効なセッショントークンです".to_string()))?;

    Ok(session_id)
}

pub fn build_state_cookie(state: &str, secure: bool) -> Cookie<'static> {
    Cookie::build((OAUTH_STATE_COOKIE_NAME, state.to_string()))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::minutes(10))
        .build()
}

pub fn clear_state_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((OAUTH_STATE_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::seconds(0))
        .build()
}

pub fn build_session_cookie(token: &str, secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, token.to_string()))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::days(7))
        .build()
}

pub fn clear_session_cookie(secure: bool) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE_NAME, ""))
        .path("/")
        .http_only(true)
        .secure(secure)
        .same_site(same_site(secure))
        .max_age(time::Duration::seconds(0))
        .build()
}

pub async fn select_user_by_session(
    pool: &PgPool,
    session_id: uuid::Uuid,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT u.id, u.google_id, u.email, u.name, u.picture_url, u.created_at, u.updated_at
        FROM users u
        INNER JOIN sessions s ON u.id = s.user_id
        WHERE s.id = $1 AND s.expires_at > NOW()
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
}

pub async fn select_user_id_by_session(
    pool: &PgPool,
    session_id: uuid::Uuid,
) -> Result<Option<uuid::Uuid>, sqlx::Error> {
    let user_id: Option<(uuid::Uuid,)> = sqlx::query_as(
        r#"
        SELECT user_id FROM sessions
        WHERE id = $1 AND expires_at > NOW()
        "#,
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    Ok(user_id.map(|record| record.0))
}

pub async fn create_session(pool: &PgPool, user_id: uuid::Uuid) -> Result<String, sqlx::Error> {
    let session_id: (uuid::Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO sessions (user_id)
        VALUES ($1)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(session_id.0.to_string())
}

pub async fn upsert_user(pool: &PgPool, user_info: &GoogleUserInfo) -> Result<User, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (google_id, email, name, picture_url)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (google_id) DO UPDATE SET
            email = EXCLUDED.email,
            name = EXCLUDED.name,
            picture_url = EXCLUDED.picture_url,
            updated_at = NOW()
        RETURNING id, google_id, email, name, picture_url, created_at, updated_at
        "#,
    )
    .bind(&user_info.sub)
    .bind(&user_info.email)
    .bind(&user_info.name)
    .bind(&user_info.picture)
    .fetch_one(pool)
    .await
}

pub async fn delete_session(pool: &PgPool, session_id: uuid::Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;

    Ok(())
}
