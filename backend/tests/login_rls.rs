use backend::{
    db::{self, Bind, Db, Executor, connect_pool_lazy, run_migrations},
    models::user::GoogleUserInfo,
    services::{
        auth,
        login_event::{self, ClientInfo, LoginOutcome},
    },
    test_constants::POSTGRES_TAG,
};
use shared::value::UserId;
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

async fn owner_pool() -> (Db, impl Drop) {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let admin = connect_pool_lazy(
        &format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres"),
        1,
    )
    .unwrap();
    admin
        .batch_execute(
            "CREATE ROLE login_test_owner LOGIN PASSWORD 'test' CREATEROLE NOSUPERUSER NOBYPASSRLS;
         GRANT CREATE ON DATABASE postgres TO login_test_owner;
         GRANT CREATE ON SCHEMA public TO login_test_owner;",
        )
        .await
        .unwrap();
    let pool = connect_pool_lazy(
        &format!("postgres://login_test_owner:test@127.0.0.1:{port}/postgres"),
        1,
    )
    .unwrap();
    run_migrations(&pool)
        .await
        .expect("非特権の所有者で migration を適用");
    let (member, can_set) = db::query(
        "SELECT pg_has_role(current_user, 'app_restricted', 'MEMBER'), pg_has_role(current_user, 'app_restricted', 'SET')",
        vec![],
    ).fetch_one::<(bool, bool), _>(&pool).await.unwrap();
    assert!(member, "CREATEROLE による自動所属がある");
    assert!(can_set, "所属だけでなく SET ROLE の許可が必要");
    (pool, node)
}

async fn login(
    pool: &Db,
    info: &GoogleUserInfo,
    with_device: bool,
) -> (auth::SessionToken, UserId) {
    if with_device {
        let (token, user, _) = auth::upsert_user_rotate_session_with_login_event(
            pool,
            info,
            vec![11; 32],
            &ClientInfo {
                country: Some("JP".into()),
                region: Some("13".into()),
                browser: Some("chrome".into()),
                os: Some("windows".into()),
            },
        )
        .await
        .expect("端末付きの本番ログイン取引");
        assert_eq!(user.name, info.name);
        (token, user.id)
    } else {
        let token = auth::upsert_user_and_rotate_session(pool, info)
            .await
            .expect("端末なしのログイン取引");
        let id = auth::select_user_id_by_session(pool, token)
            .await
            .unwrap()
            .unwrap();
        (token, id)
    }
}

async fn assert_owner_reset(pool: &Db) {
    let (role, setting) = db::query(
        "SELECT current_user::text, current_setting('app.current_user_id', true)",
        vec![],
    )
    .fetch_one::<(String, Option<String>), _>(pool)
    .await
    .unwrap();
    assert_eq!(role, "login_test_owner");
    assert!(setting.is_none_or(|value| value.is_empty()));
}

async fn login_case(existing: bool, with_device: bool) {
    let (pool, _node) = owner_pool().await;
    let mut info = GoogleUserInfo {
        sub: "login-rls-synthetic".into(),
        email: "login-rls@example.com".into(),
        name: Some("架空の長い名前のログイン利用者".into()),
        picture_url: None,
    };
    let previous = if existing {
        Some(login(&pool, &info, with_device).await)
    } else {
        None
    };
    info.name = Some("更新した架空の長い名前のログイン利用者".into());
    let (token, id) = login(&pool, &info, with_device).await;
    assert_owner_reset(&pool).await;
    if let Some((old_token, previous_id)) = previous {
        assert_eq!(id, previous_id);
        assert!(
            auth::select_user_id_by_session(&pool, old_token)
                .await
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(
        auth::select_user_id_by_session(&pool, token).await.unwrap(),
        Some(id)
    );
    let count: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE user_id = $1",
        vec![Bind::from(id)],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
    if with_device {
        let events = login_event::list_login_history(&pool, id, 10, 0)
            .await
            .unwrap();
        assert_eq!(events.len(), if existing { 2 } else { 1 });
        assert!(
            events
                .iter()
                .any(|e| e.result == LoginOutcome::NewDevice.as_str())
        );
        if existing {
            assert!(
                events
                    .iter()
                    .any(|e| e.result == LoginOutcome::Success.as_str())
            );
        }
    }
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn login_rls_new_without_device() {
    login_case(false, false).await;
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn login_rls_existing_without_device() {
    login_case(true, false).await;
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn login_rls_new_with_device() {
    login_case(false, true).await;
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn login_rls_existing_with_device() {
    login_case(true, true).await;
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn login_rls_repairs_existing_membership_without_inheriting() {
    let (pool, _node) = owner_pool().await;
    pool.batch_execute("GRANT app_restricted TO CURRENT_USER WITH SET FALSE, INHERIT FALSE")
        .await
        .unwrap();
    let (member_without_inherit, can_set) = db::query(
        "SELECT pg_has_role(current_user, 'app_restricted', 'MEMBER') AND NOT pg_has_role(current_user, 'app_restricted', 'USAGE'), pg_has_role(current_user, 'app_restricted', 'SET')",
        vec![],
    ).fetch_one::<(bool, bool), _>(&pool).await.unwrap();
    assert!(member_without_inherit);
    assert!(!can_set);
    let mut tx = pool.begin().await.unwrap();
    let error = db::query("SELECT app_enter($1)", vec![Bind::from(uuid::Uuid::nil())])
        .execute(&mut tx)
        .await
        .unwrap_err();
    assert_eq!(
        error.sql_state(),
        Some(db::SqlState::INSUFFICIENT_PRIVILEGE)
    );
    assert_eq!(
        backend::errors::ApiError::Database(error).code(),
        "DATABASE_ERROR"
    );
    tx.rollback().await.unwrap();
    for _ in 0..2 {
        pool.batch_execute(include_str!(
            "../migrations/0029_app_restricted_set_option.sql"
        ))
        .await
        .unwrap();
    }
    let (can_set, inherits) = db::query(
        "SELECT pg_has_role(current_user, 'app_restricted', 'SET'), pg_has_role(current_user, 'app_restricted', 'USAGE')",
        vec![],
    ).fetch_one::<(bool, bool), _>(&pool).await.unwrap();
    assert!(can_set);
    assert!(!inherits);
    let info = GoogleUserInfo {
        sub: "repaired-google".into(),
        email: "repaired@example.com".into(),
        name: None,
        picture_url: None,
    };
    login(&pool, &info, true).await;
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn login_rls_transaction_settings_reset() {
    let (pool, _node) = owner_pool().await;
    for commit in [true, false] {
        let mut tx = pool.begin().await.unwrap();
        let before = db::query_scalar::<i32>("SELECT pg_backend_pid()", vec![])
            .fetch_one(&mut tx)
            .await
            .unwrap();
        db::query("SELECT app_enter($1)", vec![Bind::from(uuid::Uuid::nil())])
            .execute(&mut tx)
            .await
            .unwrap();
        let (role, after) = db::query("SELECT current_user::text, pg_backend_pid()", vec![])
            .fetch_one::<(String, i32), _>(&mut tx)
            .await
            .unwrap();
        assert_eq!(role, "app_restricted");
        assert_eq!(before, after);
        if commit {
            tx.commit().await.unwrap();
        } else {
            tx.rollback().await.unwrap();
        }
        assert_owner_reset(&pool).await;
    }
    let mut tx = pool.begin().await.unwrap();
    db::query("SELECT app_enter($1)", vec![Bind::from(uuid::Uuid::nil())])
        .execute(&mut tx)
        .await
        .unwrap();
    drop(tx);
    assert_owner_reset(&pool).await;
}
