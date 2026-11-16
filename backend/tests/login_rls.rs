#![allow(clippy::unwrap_used, clippy::expect_used)]

use backend::{
    db::{self, Bind, Db, Executor, connect_pool_lazy, run_migrations},
    models::user::GoogleUserInfo,
    services::{
        auth,
        login_event::{self, ClientInfo, LoginOutcome, admin},
        login_notify,
    },
    test_constants::POSTGRES_TAG,
};
use shared::value::UserId;
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;

async fn stock_price_targets(pool: &Db) -> Vec<String> {
    db::query(
        r#"
        SELECT ab.security_code
        FROM asset_balances ab
        LEFT JOIN security_prices sp ON sp.security_code = ab.security_code
        GROUP BY ab.security_code
        HAVING BOOL_OR(sp.price_as_of IS NULL) OR MIN(sp.price_as_of) < CURRENT_DATE
        ORDER BY MIN(sp.updated_at) ASC NULLS FIRST
        LIMIT 15
        "#,
        vec![],
    )
    .fetch_all::<(String,), _>(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|(code,)| code)
    .collect()
}

async fn fund_price_targets(pool: &Db) -> Vec<String> {
    db::query(
        r#"
        SELECT fc.association_fund_cd
        FROM fund_balances fb
        JOIN fund_codes fc ON fc.normalized_name = fb.normalized_fund_name
        LEFT JOIN fund_prices fp ON fp.association_fund_cd = fc.association_fund_cd
        GROUP BY fc.association_fund_cd
        HAVING BOOL_OR(fp.price_as_of IS NULL) OR MIN(fp.price_as_of) < CURRENT_DATE
        ORDER BY MIN(fp.updated_at) ASC NULLS FIRST
        LIMIT 30
        "#,
        vec![],
    )
    .fetch_all::<(String,), _>(pool)
    .await
    .unwrap()
    .into_iter()
    .map(|(code,)| code)
    .collect()
}

async fn stock_price_targets_as(pool: &Db, user_id: UserId) -> Vec<String> {
    db::query(
        r#"
        SELECT ab.security_code
        FROM asset_balances ab
        LEFT JOIN security_prices sp ON sp.security_code = ab.security_code
        GROUP BY ab.security_code
        HAVING BOOL_OR(sp.price_as_of IS NULL) OR MIN(sp.price_as_of) < CURRENT_DATE
        ORDER BY MIN(sp.updated_at) ASC NULLS FIRST
        LIMIT 15
        "#,
        vec![],
    )
    .fetch_all_with_user(pool, user_id)
    .await
    .unwrap()
    .into_iter()
    .map(|(code,)| code)
    .collect()
}

async fn fund_price_targets_as(pool: &Db, user_id: UserId) -> Vec<String> {
    db::query(
        r#"
        SELECT fc.association_fund_cd
        FROM fund_balances fb
        JOIN fund_codes fc ON fc.normalized_name = fb.normalized_fund_name
        LEFT JOIN fund_prices fp ON fp.association_fund_cd = fc.association_fund_cd
        GROUP BY fc.association_fund_cd
        HAVING BOOL_OR(fp.price_as_of IS NULL) OR MIN(fp.price_as_of) < CURRENT_DATE
        ORDER BY MIN(fp.updated_at) ASC NULLS FIRST
        LIMIT 30
        "#,
        vec![],
    )
    .fetch_all_with_user(pool, user_id)
    .await
    .unwrap()
    .into_iter()
    .map(|(code,)| code)
    .collect()
}

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

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn force_rls_blocks_remaining_owner_paths_but_keeps_app_restricted_paths() {
    let (pool, _node) = owner_pool().await;
    let info = GoogleUserInfo {
        sub: "force-rls-synthetic".into(),
        email: "force-rls@example.com".into(),
        name: None,
        picture_url: None,
    };
    let token = auth::upsert_user_and_rotate_session(&pool, &info)
        .await
        .expect("FORCE 前の所有者ログイン");
    let user_id = auth::select_user_id_by_session(&pool, token)
        .await
        .unwrap()
        .expect("FORCE 前のセッション照会");
    let notification_token = login_notify::issue_token(&pool, user_id)
        .await
        .expect("利用者スコープで通知 token を発行");
    db::query(
        "INSERT INTO login_events (user_id, result, device_hash, occurred_at) VALUES ($1, 'success', $2, NOW() - INTERVAL '100 days')",
        vec![Bind::from(user_id), Bind::Bytes(vec![9; 32])],
    )
    .execute(&pool)
    .await
    .unwrap();
    db::query(
        "INSERT INTO asset_balances (user_id, security_code, security_name, shares, average_purchase_price, total_purchase_amount) VALUES ($1, '99999', '架空のFORCE検証銘柄', 1, 1, 1)",
        vec![Bind::from(user_id)],
    )
    .execute(&pool)
    .await
    .unwrap();
    db::query(
        "INSERT INTO fund_codes (normalized_name, association_fund_cd, fund_name) VALUES (fund_name_normalize('架空のFORCE検証ファンド'), '99999999', '架空のFORCE検証ファンド')",
        vec![],
    )
    .execute(&pool)
    .await
    .unwrap();
    db::query(
        "INSERT INTO fund_balances (user_id, fund_name, account, shares, average_purchase_price, total_purchase_amount, base_price, market_value, profit_loss, price_as_of) VALUES ($1, '架空のFORCE検証ファンド', '検証口座', 1, 1, 1, 1, 1, 0, CURRENT_DATE)",
        vec![Bind::from(user_id)],
    )
    .execute(&pool)
    .await
    .unwrap();
    let holdings: i64 = db::query_scalar(
        "SELECT (SELECT COUNT(*) FROM asset_balances WHERE user_id = $1) + (SELECT COUNT(*) FROM fund_balances WHERE user_id = $1)",
        vec![Bind::from(user_id)],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(holdings, 2);
    assert_eq!(stock_price_targets(&pool).await, ["99999"]);
    assert_eq!(fund_price_targets(&pool).await, ["99999999"]);

    pool.batch_execute(
        "ALTER TABLE users FORCE ROW LEVEL SECURITY;
         ALTER TABLE sessions FORCE ROW LEVEL SECURITY;
         ALTER TABLE dividends FORCE ROW LEVEL SECURITY;
         ALTER TABLE domestic_stocks FORCE ROW LEVEL SECURITY;
         ALTER TABLE mutualfunds FORCE ROW LEVEL SECURITY;
         ALTER TABLE asset_balances FORCE ROW LEVEL SECURITY;
         ALTER TABLE fund_balances FORCE ROW LEVEL SECURITY;
         ALTER TABLE csv_import_history FORCE ROW LEVEL SECURITY;
         ALTER TABLE login_events FORCE ROW LEVEL SECURITY;
         ALTER TABLE login_notify_tokens FORCE ROW LEVEL SECURITY;
         ALTER TABLE admin_login_views FORCE ROW LEVEL SECURITY;",
    )
    .await
    .unwrap();

    let forced: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'public' AND c.relkind = 'r' AND c.relforcerowsecurity",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(forced, 11);

    assert!(
        auth::select_user_id_by_session(&pool, token)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        auth::select_user_by_session(&pool, token)
            .await
            .unwrap()
            .is_none()
    );
    let error = auth::upsert_user_and_rotate_session(&pool, &info)
        .await
        .unwrap_err();
    assert_eq!(
        error.sql_state(),
        Some(db::SqlState::INSUFFICIENT_PRIVILEGE)
    );
    assert!(
        login_notify::peek_token(&pool, &notification_token)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        login_notify::consume_token(&pool, &notification_token)
            .await
            .unwrap()
            .is_none()
    );
    auth::delete_session(&pool, token).await.unwrap();
    let admin_results = admin::search(&pool, admin::AdminLoginQuery::default())
        .await
        .unwrap();
    assert_eq!(admin_results.total, 0);
    assert_eq!(login_event::prune_login_events(&pool).await.unwrap(), 0);
    assert!(stock_price_targets(&pool).await.is_empty());
    assert!(fund_price_targets(&pool).await.is_empty());
    assert_eq!(stock_price_targets_as(&pool, user_id).await, ["99999"]);
    assert_eq!(fund_price_targets_as(&pool, user_id).await, ["99999999"]);
    run_migrations(&pool)
        .await
        .expect("適用済み migration 履歴の確認は FORCE 下でも通る");

    let visible: i64 = db::query_scalar("SELECT COUNT(*) FROM users", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(visible, 0);
    pool.batch_execute("SET row_security = off").await.unwrap();
    let error = db::query_scalar::<i64>("SELECT COUNT(*) FROM users", vec![])
        .fetch_one(&pool)
        .await
        .unwrap_err();
    assert_eq!(
        error.sql_state(),
        Some(db::SqlState::INSUFFICIENT_PRIVILEGE)
    );
    pool.batch_execute("RESET row_security").await.unwrap();

    let history = login_event::list_login_history(&pool, user_id, 10, 0)
        .await
        .unwrap();
    assert_eq!(history.len(), 1);
    pool.batch_execute("ALTER TABLE sessions NO FORCE ROW LEVEL SECURITY")
        .await
        .unwrap();
    assert_eq!(
        auth::select_user_id_by_session(&pool, token).await.unwrap(),
        Some(user_id)
    );
    pool.batch_execute("ALTER TABLE sessions FORCE ROW LEVEL SECURITY")
        .await
        .unwrap();
    assert_eq!(auth::delete_all_sessions(&pool, user_id).await.unwrap(), 1);
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

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn auth_paths_use_limited_functions() {
    let (pool, _node) = owner_pool().await;

    let (is_super, can_bypass): (bool, bool) = db::query(
        "SELECT rolsuper, rolbypassrls FROM pg_roles WHERE rolname = CURRENT_USER",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(!is_super, "通常接続ロールは NOSUPERUSER であること");
    assert!(!can_bypass, "通常接続ロールは NOBYPASSRLS であること");

    for signature in [
        "public.auth_session_user_by_token(bytea)",
        "public.auth_lookup_user_id_by_session(bytea)",
        "public.auth_delete_session_by_token(bytea)",
    ] {
        let executable = db::query_scalar::<bool>(
            format!("SELECT has_function_privilege('app_restricted', '{signature}', 'EXECUTE')"),
            vec![],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(executable, "{signature} は app_restricted が実行できるはず");
        let public = db::query_scalar::<bool>(
            format!("SELECT has_function_privilege('public', '{signature}', 'EXECUTE')"),
            vec![],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!public, "{signature} は public から実行できないはず");
        let is_definer = db::query_scalar::<bool>(
            "SELECT prosecdef FROM pg_proc WHERE oid = $1::regprocedure",
            vec![Bind::from(signature)],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(is_definer, "{signature} は DEFINER のはず");
    }
    let volatile = db::query_scalar::<String>(
        "SELECT provolatile::text FROM pg_proc WHERE oid = 'public.auth_session_user_by_token(bytea)'::regprocedure",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(volatile, "s", "照会関数は STABLE のはず");
    let config = db::query(
        "SELECT unnest(proconfig) FROM pg_proc WHERE oid = 'public.auth_session_user_by_token(bytea)'::regprocedure",
        vec![],
    )
    .fetch_all::<(String,), _>(&pool)
    .await
    .unwrap();
    assert!(
        config.contains(&("search_path=public, pg_temp".to_string(),)),
        "照会関数は search_path を固定するはず"
    );

    let mut info = GoogleUserInfo {
        sub: "auth-paths-google".into(),
        email: "auth-paths@example.com".into(),
        name: Some("架空の認証経路利用者".into()),
        picture_url: None,
    };
    let token = auth::upsert_user_and_rotate_session(&pool, &info)
        .await
        .expect("新規ログインは通るはず");
    let user = auth::select_user_by_session(&pool, token)
        .await
        .expect("通常認証の照会は通るはず")
        .expect("新規ログインのセッションがあるはず");
    assert_eq!(user.email, info.email);
    let user_id = auth::select_user_id_by_session(&pool, token)
        .await
        .expect("利用者特定は通るはず")
        .expect("利用者 ID があるはず");
    assert_eq!(user.id, user_id);

    let unknown: auth::SessionToken = "550e8400-e29b-41d4-a716-446655440000".parse().unwrap();
    assert!(
        auth::select_user_by_session(&pool, unknown)
            .await
            .expect("未知トークンの照会は通るはず")
            .is_none(),
        "未知のトークンは未認証扱いであること"
    );
    assert!(
        auth::select_user_id_by_session(&pool, unknown)
            .await
            .expect("未知トークンの特定は通るはず")
            .is_none(),
        "未知のトークンから利用者は特定できないこと"
    );

    let info_expired = GoogleUserInfo {
        sub: "auth-paths-expired".into(),
        email: "auth-paths-expired@example.com".into(),
        name: None,
        picture_url: None,
    };
    let expired_token = auth::upsert_user_and_rotate_session(&pool, &info_expired)
        .await
        .expect("期限切れ検証用のログインは通るはず");
    db::query(
        "UPDATE sessions SET expires_at = NOW() - INTERVAL '1 second' WHERE token_hash = $1",
        vec![Bind::Bytes(expired_token.hash())],
    )
    .execute(&pool)
    .await
    .unwrap();
    assert!(
        auth::select_user_by_session(&pool, expired_token)
            .await
            .expect("期限切れトークンの照会は通るはず")
            .is_none(),
        "期限切れのトークンは未認証扱いであること"
    );

    info.name = Some("更新した架空の認証経路利用者".into());
    let device_token = auth::upsert_user_rotate_session_with_login_event(
        &pool,
        &info,
        vec![22; 32],
        &ClientInfo {
            country: Some("JP".into()),
            region: Some("13".into()),
            browser: Some("chrome".into()),
            os: Some("windows".into()),
        },
    )
    .await
    .expect("端末付きの再ログインは通るはず")
    .0;
    assert!(
        auth::select_user_by_session(&pool, token)
            .await
            .expect("失効トークンの照会は通るはず")
            .is_none(),
        "再ログインで旧トークンは失効すること"
    );
    let relogin_user = auth::select_user_by_session(&pool, device_token)
        .await
        .expect("再ログイン後の照会は通るはず")
        .expect("再ログインのセッションがあるはず");
    assert_eq!(relogin_user.id, user_id);
    assert_eq!(relogin_user.name, info.name);

    auth::delete_session(&pool, device_token)
        .await
        .expect("ログアウトは通るはず");
    let count: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 0, "ログアウトでセッション行が消えること");
    assert!(
        auth::select_user_by_session(&pool, device_token)
            .await
            .expect("ログアウト後の照会は通るはず")
            .is_none(),
        "ログアウト後は未認証扱いであること"
    );
    auth::delete_session(&pool, device_token)
        .await
        .expect("二重ログアウトも冪等に成功すること");

    let retoken = auth::upsert_user_and_rotate_session(&pool, &info)
        .await
        .expect("削除前のログインは通るはず");
    assert_eq!(
        auth::select_user_id_by_session(&pool, retoken)
            .await
            .expect("削除前の特定は通るはず"),
        Some(user_id)
    );
    auth::delete_account(&pool, user_id)
        .await
        .expect("アカウント削除は通るはず");
    assert!(
        auth::select_user_by_session(&pool, retoken)
            .await
            .expect("削除後の照会は通るはず")
            .is_none(),
        "アカウント削除後は未認証扱いであること"
    );
    assert!(
        auth::select_user_id_by_session(&pool, retoken)
            .await
            .expect("削除後の特定は通るはず")
            .is_none(),
        "アカウント削除後は利用者を特定できないこと"
    );
    let users: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM users WHERE id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(users, 0, "アカウント削除で利用者行が消えること");

    let info_c = GoogleUserInfo {
        sub: "auth-paths-restricted".into(),
        email: "auth-paths-restricted@example.com".into(),
        name: None,
        picture_url: None,
    };
    let (token_c, id_c) = login(&pool, &info_c, false).await;
    let mut tx = pool.begin().await.unwrap();
    db::query("SELECT app_enter($1)", vec![Bind::from(id_c)])
        .execute(&mut tx)
        .await
        .expect("app_enter は通るはず");
    let (role,): (String,) = db::query("SELECT current_user::text", vec![])
        .fetch_one(&mut tx)
        .await
        .unwrap();
    assert_eq!(role, "app_restricted");
    let restricted_user = db::query_as::<backend::models::user::User>(
        "SELECT * FROM auth_session_user_by_token($1)",
        vec![Bind::Bytes(token_c.hash())],
    )
    .fetch_optional(&mut tx)
    .await
    .expect("制限ロールで照会関数は呼べるはず")
    .expect("本人のセッションがあるはず");
    assert_eq!(restricted_user.id, id_c);
    let restricted_id = db::query_scalar::<Option<UserId>>(
        "SELECT auth_lookup_user_id_by_session($1)",
        vec![Bind::Bytes(token_c.hash())],
    )
    .fetch_one(&mut tx)
    .await
    .expect("制限ロールで特定関数は呼べるはず");
    assert_eq!(restricted_id, Some(id_c));
    tx.rollback().await.unwrap();
    assert_owner_reset(&pool).await;
}
