#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use backend::{
    config::Config,
    db::{self, Bind},
    models::user::GoogleUserInfo,
    services::auth::{self as auth_svc, SessionToken},
    state::{AppState, Secrets},
};
use std::{sync::Arc, time::Duration};
use tokio::time::timeout;
use tower::ServiceExt;
use uuid::Uuid;

mod common;

use common::*;

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn session_cookie_legacy_migration_preserves_valid_sessions_and_rejects_expired_sessions() {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header::SET_COOKIE},
        routing::get,
    };
    use backend::{
        config::Config,
        handlers::v1::auth,
        models::user::GoogleUserInfo,
        services::login_event::ClientInfo,
        state::{AppState, Secrets},
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    let (pool, _node) = start_test_pool().await;
    let (valid_token, _, _) = auth_svc::upsert_user_rotate_session_with_login_event(
        &pool,
        &GoogleUserInfo {
            sub: "cookie-migration-valid".to_string(),
            email: "cookie-migration@example.com".to_string(),
            name: Some("移行テスト".to_string()),
            picture_url: None,
        },
        vec![1; 32],
        &ClientInfo::default(),
    )
    .await
    .unwrap();
    let state = AppState {
        pool: pool.clone(),
        secrets: Arc::new(Secrets {
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "https://shoken-webapp.pages.dev".to_string(),
            gmail_user: None,
            gmail_app_password: None,
        }),
        config: Arc::new(Config::default()),
        google_oauth: None,
        jquants_client: None,
    };
    let app = Router::new()
        .route("/api/v1/session", get(auth::get_session))
        .with_state(state);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/session")
                .header("cookie", format!("session_token={valid_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookies = response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect::<Vec<_>>();
    assert!(
        cookies
            .iter()
            .any(|value| value.starts_with("__Host-session_token="))
    );
    assert!(
        cookies
            .iter()
            .any(|value| value.starts_with("session_token=") && value.contains("Expires="))
    );

    db::query(
        "UPDATE sessions SET expires_at = NOW() - INTERVAL '1 second' WHERE token_hash = $1",
        vec![Bind::Bytes(valid_token.hash())],
    )
    .execute(&pool)
    .await
    .unwrap();
    let expired = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/session")
                .header("cookie", format!("session_token={valid_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(expired.status(), StatusCode::UNAUTHORIZED);
    assert!(expired.headers().get(SET_COOKIE).is_none());
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
#[allow(clippy::literal_string_with_formatting_args)]
async fn deleting_current_device_clears_both_session_cookie_names() {
    use axum::{
        Router,
        body::Body,
        http::{Method, Request, StatusCode, header::SET_COOKIE},
        routing::delete,
    };
    use backend::{
        config::Config,
        handlers::v1::login_history,
        models::user::GoogleUserInfo,
        services::{auth::DeviceToken, login_event::ClientInfo},
        state::{AppState, Secrets},
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    let (pool, _node) = start_test_pool().await;
    let (session_token, user, _) = auth_svc::upsert_user_rotate_session_with_login_event(
        &pool,
        &GoogleUserInfo {
            sub: "delete-current-device".to_string(),
            email: "delete-current-device@example.com".to_string(),
            name: Some("端末解除テスト".to_string()),
            picture_url: None,
        },
        vec![4; 32],
        &ClientInfo::default(),
    )
    .await
    .unwrap();
    let device_token = DeviceToken::new();
    db::query(
        "UPDATE sessions SET device_hash = $1 WHERE token_hash = $2",
        vec![
            Bind::Bytes(device_token.hash()),
            Bind::Bytes(session_token.hash()),
        ],
    )
    .execute(&pool)
    .await
    .unwrap();
    let id = backend::services::login_event::device_opaque_id(&user.id, &device_token.hash());
    let state = AppState {
        pool: pool.clone(),
        secrets: Arc::new(Secrets {
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "https://shoken-webapp.pages.dev".to_string(),
            gmail_user: None,
            gmail_app_password: None,
        }),
        config: Arc::new(Config::default()),
        google_oauth: None,
        jquants_client: None,
    };
    let app = Router::new()
        .route("/api/v1/devices/{id}", delete(login_history::delete_device))
        .with_state(state);
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/v1/devices/{id}"))
                .header(
                    "cookie",
                    format!("device_id={device_token}; session_token={session_token}"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookies = response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .collect::<Vec<_>>();
    assert!(cookies.iter().any(|value| {
        value.starts_with("__Host-session_token=") && value.contains("Max-Age=0")
    }));
    assert!(
        cookies
            .iter()
            .any(|value| { value.starts_with("session_token=") && value.contains("Max-Age=0") })
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn auth_session_upsert_rotate_and_delete() {
    let (pool, _node) = start_test_pool().await;

    let info = GoogleUserInfo {
        sub: "google-sub-0001".to_string(),
        email: "first@example.com".to_string(),
        name: Some("ユーザー".to_string()),
        picture_url: None,
    };

    let token1 = auth_svc::upsert_user_and_rotate_session(&pool, &info)
        .await
        .expect("初回ログインでセッション発行");
    let session1 = token1;

    let user = auth_svc::select_user_by_session(&pool, session1)
        .await
        .expect("select_user_by_session が成功")
        .expect("有効セッションからユーザーが解決できる");
    assert_eq!(user.google_id, "google-sub-0001");
    assert_eq!(user.email, "first@example.com");
    assert_eq!(
        auth_svc::select_user_id_by_session(&pool, session1)
            .await
            .expect("select_user_id_by_session が成功"),
        Some(user.id)
    );

    // 同一 Google アカウントの再ログインはユーザー情報を更新し、旧セッションを失効させる
    let info2 = GoogleUserInfo {
        email: "updated@example.com".to_string(),
        ..info.clone()
    };
    let token2 = auth_svc::upsert_user_and_rotate_session(&pool, &info2)
        .await
        .expect("再ログインでセッション再発行");
    let session2 = token2;
    assert_ne!(session1, session2);
    assert!(
        auth_svc::select_user_id_by_session(&pool, session1)
            .await
            .unwrap()
            .is_none(),
        "ローテーションで旧セッションは失効する"
    );
    let updated = auth_svc::select_user_by_session(&pool, session2)
        .await
        .unwrap()
        .expect("新セッションからユーザー解決");
    assert_eq!(updated.id, user.id);
    assert_eq!(updated.email, "updated@example.com");

    // 期限切れセッションはユーザー解決しない
    let updated = db::query("UPDATE sessions SET expires_at = NOW() - INTERVAL '1 hour' WHERE token_hash = sha256(convert_to($1::text, 'UTF8'))",
        vec![Bind::from(session2.to_string())])
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(updated, 1, "期限変更が対象行を更新すること");
    assert!(
        auth_svc::select_user_id_by_session(&pool, session2)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        auth_svc::select_user_by_session(&pool, session2)
            .await
            .unwrap()
            .is_none()
    );

    // delete_session で明示失効
    let updated = db::query("UPDATE sessions SET expires_at = NOW() + INTERVAL '1 hour' WHERE token_hash = sha256(convert_to($1::text, 'UTF8'))",
        vec![Bind::from(session2.to_string())])
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(updated, 1, "期限復帰が対象行を更新すること");
    auth_svc::delete_session(&pool, session2)
        .await
        .expect("セッション削除");
    assert!(
        auth_svc::select_user_id_by_session(&pool, session2)
            .await
            .unwrap()
            .is_none()
    );

    // delete_account はセッションごとユーザーを削除する
    let token3 = auth_svc::upsert_user_and_rotate_session(&pool, &info2)
        .await
        .expect("3回目のセッション発行");
    let session3 = token3;
    auth_svc::delete_account(&pool, user.id)
        .await
        .expect("アカウント削除");
    assert!(
        auth_svc::select_user_by_session(&pool, session3)
            .await
            .unwrap()
            .is_none()
    );
    let remaining: Option<(Uuid,)> = db::query_as(
        "SELECT id FROM users WHERE id = $1",
        vec![Bind::from(user.id)],
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(remaining.is_none());
}

/// rotate が期限切れ行を掃除し、ロック中の行をスキップすること。
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn session_rotation_skips_locked_expired_sessions() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // rotate は期限切れの行を掃除する。他トランザクションがロック中の行はスキップして待たない
    let other_user = create_test_user(&pool).await;
    let expired_row = Uuid::new_v4();
    db::query(
        "INSERT INTO sessions (id, user_id, token_hash, expires_at) VALUES ($1, $2, sha256(convert_to($1::uuid::text, 'UTF8')), NOW() - INTERVAL '1 hour')",
        vec![Bind::from(expired_row), Bind::from(other_user)],
    )
    .execute(&pool)
    .await
    .expect("期限切れセッション挿入");
    let locked_expired = Uuid::new_v4();
    db::query(
        "INSERT INTO sessions (id, user_id, token_hash, expires_at) VALUES ($1, $2, sha256(convert_to($1::uuid::text, 'UTF8')), NOW() - INTERVAL '1 hour')",
        vec![Bind::from(locked_expired), Bind::from(other_user)],
    )
    .execute(&pool)
    .await
    .expect("ロック対象の期限切れセッション挿入");
    let mut lock_tx = pool.begin().await.expect("ロック用トランザクション");
    db::query_as::<(Uuid,)>(
        "SELECT id FROM sessions WHERE id = $1 FOR UPDATE",
        vec![Bind::from(locked_expired)],
    )
    .fetch_one(&mut lock_tx)
    .await
    .expect("期限切れ行のロック");

    let info = GoogleUserInfo {
        sub: format!("test_google_{user_id}"),
        email: "rolling@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    // ロック中の他ユーザーの期限切れ行があってもログインは停滞しない
    let new_token = timeout(
        Duration::from_secs(5),
        auth_svc::upsert_user_and_rotate_session(&pool, &info),
    )
    .await
    .expect("他ユーザーのロック中の期限切れ行でログインが停滞しないこと")
    .expect("新版のセッション発行");
    let new_session = new_token;

    // 期限切れ行は掃除されるが、ロック中の行はスキップされる
    let expired_row_left: Option<(Uuid,)> = db::query_as(
        "SELECT id FROM sessions WHERE id = $1",
        vec![Bind::from(expired_row)],
    )
    .fetch_optional(&pool)
    .await
    .expect("期限切れ行の確認");
    assert!(
        expired_row_left.is_none(),
        "他ユーザーの期限切れ行が rotate で削除されること"
    );
    let locked_row_left: Option<(Uuid,)> = db::query_as(
        "SELECT id FROM sessions WHERE id = $1",
        vec![Bind::from(locked_expired)],
    )
    .fetch_optional(&pool)
    .await
    .expect("ロック中の期限切れ行の確認");
    assert!(
        locked_row_left.is_some(),
        "ロック中の期限切れ行は削除をスキップされること"
    );
    lock_tx.rollback().await.expect("ロック解除");

    // 新版は id にトークンを書かない
    let row: Option<(Uuid,)> = db::query_as(
        "SELECT id FROM sessions WHERE token_hash = sha256(convert_to($1::text, 'UTF8'))",
        vec![Bind::from(new_session.to_string())],
    )
    .fetch_optional(&pool)
    .await
    .expect("新版発行行の取得");
    let (id,) = row.expect("新版発行のセッション行がある");
    assert_ne!(
        id.to_string(),
        new_session.to_string(),
        "id にトークンを保持しないこと"
    );
    assert!(
        db::query(
            "INSERT INTO sessions (id, user_id) VALUES ($1, $2)",
            vec![Bind::from(Uuid::new_v4()), Bind::from(user_id)],
        )
        .execute(&pool)
        .await
        .is_err(),
        "token_hash の無いセッションは作れないこと"
    );
    auth_svc::delete_session(&pool, new_session)
        .await
        .expect("新版発行セッションの削除");
    assert!(
        auth_svc::select_user_id_by_session(&pool, new_session)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn account_delete_confirmation_http_lifecycle() {
    let (pool, _node) = start_test_pool().await;
    let info = GoogleUserInfo {
        sub: format!("google-sub-http-{}", Uuid::new_v4()),
        email: "http-lifecycle@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    let session_token = auth_svc::upsert_user_and_rotate_session(&pool, &info)
        .await
        .expect("セッション発行");
    let user_id: Option<(Uuid,)> = db::query_as(
        "SELECT user_id FROM sessions WHERE token_hash = sha256(convert_to($1::text, 'UTF8'))",
        vec![Bind::from(session_token.to_string())],
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    let user_id = user_id.expect("セッション行がある").0;

    let state = AppState {
        pool: pool.clone(),
        secrets: Arc::new(Secrets {
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
            gmail_user: None,
            gmail_app_password: None,
        }),
        config: Arc::new(Config::default()),
        google_oauth: None,
        jquants_client: None,
    };
    let app = backend::handlers::v1::auth_routes().with_state(state);

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/account-deletion-confirmations")
                .header("cookie", format!("session_token={session_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let set_cookie = response
        .headers()
        .get_all(axum::http::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|c| c.starts_with("account_delete_confirmation="))
        .expect("確認 Cookie が発行されること");
    let confirmation = set_cookie
        .split(';')
        .next()
        .and_then(|kv| kv.split_once('='))
        .map(|(_, v)| v.to_string())
        .expect("Cookie 値を取り出せること");

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri("/api/v1/account")
                .header(
                    "cookie",
                    format!(
                        "session_token={session_token}; account_delete_confirmation={confirmation}"
                    ),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let cookies: Vec<String> = response
        .headers()
        .get_all(axum::http::header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok().map(String::from))
        .collect();
    for name in ["session_token", "account_delete_confirmation"] {
        let cleared = cookies
            .iter()
            .any(|c| c.starts_with(&format!("{name}=")) && c.contains("Max-Age=0"));
        assert!(cleared, "{name} が削除応答で失効すること: {cookies:?}");
    }

    let remaining_session: Option<(Uuid,)> = db::query_as(
        "SELECT id FROM sessions WHERE token_hash = sha256(convert_to($1::text, 'UTF8'))",
        vec![Bind::from(session_token.to_string())],
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(remaining_session.is_none());
    let remaining_user: Option<(Uuid,)> = db::query_as(
        "SELECT id FROM users WHERE id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(remaining_user.is_none());
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/v1/session")
                .header("cookie", format!("session_token={session_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn account_delete_with_invalid_session_returns_unauthorized() {
    let (pool, _node) = start_test_pool().await;
    let info = GoogleUserInfo {
        sub: format!("google-sub-invalid-delete-{}", Uuid::new_v4()),
        email: "invalid-delete@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    let token = auth_svc::upsert_user_and_rotate_session(&pool, &info)
        .await
        .expect("セッション発行");
    let state = AppState {
        pool: pool.clone(),
        secrets: Arc::new(Secrets {
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
            gmail_user: None,
            gmail_app_password: None,
        }),
        config: Arc::new(Config::default()),
        google_oauth: None,
        jquants_client: None,
    };
    let app = backend::handlers::v1::auth_routes().with_state(state);
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/v1/account-deletion-confirmations")
                .header("cookie", format!("session_token={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let confirmation = response
        .headers()
        .get_all(axum::http::header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find(|cookie| cookie.starts_with("account_delete_confirmation="))
        .and_then(|cookie| cookie.split(';').next())
        .and_then(|cookie| cookie.split_once('='))
        .map(|(_, value)| value.to_string())
        .expect("確認 Cookie が発行される");

    let revoked_token = auth_svc::upsert_user_and_rotate_session(
        &pool,
        &GoogleUserInfo {
            sub: format!("google-sub-revoked-{}", Uuid::new_v4()),
            email: "revoked-delete@example.com".to_string(),
            name: None,
            picture_url: None,
        },
    )
    .await
    .expect("失効用セッション発行");
    let unknown_token = SessionToken::new();
    let unknown_confirmation = backend::handlers::v1::auth::issue_account_delete_confirmation(
        &unknown_token.to_string(),
        time::OffsetDateTime::now_utc().unix_timestamp(),
    )
    .expect("未知 token 用確認 Cookie");
    for (invalid_token, expire, revoke, confirmation) in [
        (token, true, false, confirmation.clone()),
        (
            revoked_token,
            false,
            true,
            backend::handlers::v1::auth::issue_account_delete_confirmation(
                &revoked_token.to_string(),
                time::OffsetDateTime::now_utc().unix_timestamp(),
            )
            .expect("失効 token 用確認 Cookie"),
        ),
        (unknown_token, false, false, unknown_confirmation),
    ] {
        if expire {
            db::query(
                "UPDATE sessions SET expires_at = NOW() - INTERVAL '1 hour' WHERE token_hash = $1",
                vec![Bind::Bytes(token.hash())],
            )
            .execute(&pool)
            .await
            .unwrap();
        }
        if revoke {
            auth_svc::delete_session(&pool, invalid_token)
                .await
                .expect("セッション失効");
        }
        assert!(
            auth_svc::select_user_id_by_session(&pool, invalid_token)
                .await
                .unwrap()
                .is_none()
        );
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::DELETE)
                    .uri("/api/v1/account")
                    .header(
                        "cookie",
                        format!("session_token={invalid_token}; account_delete_confirmation={confirmation}"),
                    )
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn select_user_id_by_unknown_session_returns_none() {
    let (pool, _node) = start_test_pool().await;
    assert_eq!(
        auth_svc::select_user_id_by_session(&pool, SessionToken::new())
            .await
            .expect("未知セッションの照会は成功する"),
        None
    );
}
