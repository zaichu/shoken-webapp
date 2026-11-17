#![allow(clippy::unwrap_used, clippy::expect_used)]

use backend::{
    db::{self, Bind, Db, Executor, connect_pool_lazy, run_migrations},
    models::user::GoogleUserInfo,
    services::{
        auth, dividend_cache,
        fund_price::{self, FundNav},
        jquants::JQuantsClient,
        login_event::{self, ClientInfo, LoginOutcome, admin},
        login_notify, stock, stock_master,
        stock_price::{self, ClosePrice},
    },
    test_constants::POSTGRES_TAG,
};
use shared::value::{SecurityCode, UserId};
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

async fn stock_price_targets_limited(pool: &Db) -> Vec<String> {
    let mut tx = pool.begin().await.unwrap();
    let codes = db::query(
        "SELECT security_code FROM scheduled_stock_price_targets_v2(CURRENT_DATE, 15)",
        vec![],
    )
    .fetch_all::<(String,), _>(&mut tx)
    .await
    .unwrap()
    .into_iter()
    .map(|(code,)| code)
    .collect();
    tx.commit().await.unwrap();
    codes
}

async fn fund_price_targets_limited(pool: &Db) -> Vec<String> {
    let mut tx = pool.begin().await.unwrap();
    let codes = db::query(
        "SELECT association_fund_cd FROM scheduled_fund_price_targets_v2(CURRENT_DATE, 30)",
        vec![],
    )
    .fetch_all::<(String,), _>(&mut tx)
    .await
    .unwrap()
    .into_iter()
    .map(|(code,)| code)
    .collect();
    tx.commit().await.unwrap();
    codes
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

async fn owner_pool() -> (Db, Db, Db, impl Drop) {
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
    // 関数所有ロールを作るため、実行ロールに限定的な CREATEROLE 権限を与える。
    admin
        .batch_execute(
            "CREATE ROLE login_test_owner LOGIN PASSWORD 'test' CREATEROLE NOSUPERUSER BYPASSRLS;
         GRANT CREATE ON DATABASE postgres TO login_test_owner;
         GRANT CREATE ON SCHEMA public TO login_test_owner WITH GRANT OPTION;
         CREATE ROLE app_restricted NOLOGIN;
         GRANT app_restricted TO login_test_owner WITH ADMIN TRUE, SET TRUE, INHERIT FALSE;",
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
        .expect("非 superuser のテスト用 owner で migration を適用");
    admin
        .batch_execute(
            "ALTER FUNCTION public.auth_lookup_user_id_by_session(bytea) OWNER TO postgres;
             ALTER FUNCTION public.auth_session_user_by_token(bytea) OWNER TO postgres;
             ALTER FUNCTION public.auth_delete_session_by_token(bytea) OWNER TO postgres;
             ALTER FUNCTION public.auth_upsert_user(text, text, text, text) OWNER TO postgres;
             ALTER FUNCTION public.auth_rotate_session(uuid, bytea, bytea) OWNER TO postgres;
             ALTER FUNCTION public.login_notify_peek_by_token(bytea) OWNER TO postgres",
        )
        .await
        .unwrap();
    admin
        .batch_execute("ALTER ROLE login_test_owner NOBYPASSRLS")
        .await
        .unwrap();
    admin
        .batch_execute("ALTER ROLE app_worker PASSWORD 'test'")
        .await
        .unwrap();
    let worker = connect_pool_lazy(
        &format!("postgres://app_worker:test@127.0.0.1:{port}/postgres"),
        1,
    )
    .unwrap();
    let (member, can_set) = db::query(
        "SELECT pg_has_role(current_user, 'app_restricted', 'MEMBER'), pg_has_role(current_user, 'app_restricted', 'SET')",
        vec![],
    ).fetch_one::<(bool, bool), _>(&pool).await.unwrap();
    assert!(member, "CREATEROLE による自動所属がある");
    assert!(can_set, "所属だけでなく SET ROLE の許可が必要");
    let owner_is_non_superuser = db::query_scalar::<bool>(
        "SELECT rolcanlogin AND NOT rolsuper AND NOT rolbypassrls
         FROM pg_catalog.pg_roles WHERE rolname = current_user",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        owner_is_non_superuser,
        "旧 postgres 接続相当は非 superuser のはず"
    );
    (pool, worker, admin, node)
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn force_rls_blocks_remaining_owner_paths_but_keeps_app_restricted_paths() {
    let (pool, worker, _admin, _node) = owner_pool().await;
    let info = GoogleUserInfo {
        sub: "force-rls-synthetic".into(),
        email: "force-rls@example.com".into(),
        name: None,
        picture_url: None,
    };
    let worker_role: bool = db::query_scalar(
        "SELECT rolcanlogin AND NOT rolsuper AND NOT rolbypassrls AND NOT rolcreaterole
         FROM pg_catalog.pg_roles WHERE rolname = current_user",
        vec![],
    )
    .fetch_one(&worker)
    .await
    .unwrap();
    assert!(worker_role);
    let worker_membership: bool = db::query_scalar(
        "SELECT pg_has_role('app_worker', 'app_restricted', 'SET')
            AND NOT pg_has_role('app_worker', 'app_restricted', 'USAGE')
            AND NOT EXISTS (
                SELECT 1 FROM pg_catalog.pg_roles r
                WHERE r.rolname IN (
                    'app_notify_token_v2_executor', 'app_admin_login_executor',
                    'app_price_target_executor', 'app_login_event_prune_executor',
                    'app_shared_write_executor'
                ) AND pg_has_role('app_worker', r.oid, 'MEMBER')
            )
            AND NOT EXISTS (
                SELECT 1 FROM pg_catalog.pg_class c
                JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                WHERE n.nspname = 'public'
                  AND c.relkind IN ('r', 'p', 'v', 'm')
                  AND c.relowner = (SELECT oid FROM pg_catalog.pg_roles WHERE rolname = 'app_worker')
            )",
        vec![],
    )
    .fetch_one(&worker)
    .await
    .unwrap();
    assert!(worker_membership);
    // app_worker は共有表(利用者の行なし)の読み取りだけを持ち、個人表の権限は持たない。
    // 共有表への書き込みは限定関数の EXECUTE に限る。
    for table in [
        "public.stock",
        "public.security_prices",
        "public.dividend_per_share_cache",
        "public.market_data_provider_rate_control",
        "public.fund_codes",
        "public.fund_prices",
    ] {
        let readable = db::query_scalar::<bool>(
            "SELECT has_table_privilege('app_worker', $1, 'SELECT')",
            vec![Bind::from(table)],
        )
        .fetch_one(&worker)
        .await
        .unwrap();
        assert!(readable, "app_worker は {table} を読める");
        for privilege in ["INSERT", "UPDATE", "DELETE"] {
            let writable = db::query_scalar::<bool>(
                "SELECT has_table_privilege('app_worker', $1, $2)",
                vec![Bind::from(table), Bind::from(privilege)],
            )
            .fetch_one(&worker)
            .await
            .unwrap();
            assert!(!writable, "app_worker は {table} へ {privilege} できない");
        }
    }
    // 残高ビューは所有者権限で動くため app_worker 直からは読めない。
    // 利用者の残高読み取りは app_restricted 経由(WHERE user_id 付き)の既存経路だけを使う。
    for view in ["public.asset_balances_view", "public.fund_balances_view"] {
        let readable = db::query_scalar::<bool>(
            "SELECT has_table_privilege('app_worker', $1, 'SELECT')",
            vec![Bind::from(view)],
        )
        .fetch_one(&worker)
        .await
        .unwrap();
        assert!(!readable, "app_worker は {view} を読めてはならない");
    }
    for signature in [
        "public.admin_login_search_data_v2(text[],text[],text[],text,text,bigint,bigint)",
        "public.admin_login_search_count_v2(text[],text[],text[])",
        "public.scheduled_stock_price_targets_v2(date,bigint)",
        "public.scheduled_fund_price_targets_v2(date,bigint)",
        "public.scheduled_stock_price_mark_attempted_v2(text[])",
        "public.scheduled_stock_price_apply_v2(text,numeric,numeric,date)",
        "public.scheduled_fund_price_mark_attempted_v2(text[])",
        "public.scheduled_fund_price_apply_v2(text,numeric,numeric,date)",
        "public.scheduled_fund_price_record_failure_v2(text,text)",
        "public.scheduled_fund_price_clear_failure_v2(text)",
        "public.prune_login_events_older_than_90_days_v2()",
        "public.dividend_cache_enqueue_v2(varchar[])",
        "public.dividend_cache_apply_success_v2(text,numeric,varchar)",
        "public.dividend_cache_apply_error_v2(text,text,bigint)",
        "public.rate_control_acquire_jquants_v2()",
        "public.rate_control_cooldown_v2(text,bigint)",
        "public.stock_master_claim_v2()",
        "public.stock_master_postpone_v2()",
        "public.stock_master_replace_v2(date[],text[],text[],text[],text[],text[],text[])",
    ] {
        let executable = db::query_scalar::<bool>(
            "SELECT has_function_privilege('app_worker', $1, 'EXECUTE')",
            vec![Bind::from(signature)],
        )
        .fetch_one(&worker)
        .await
        .unwrap();
        assert!(executable, "{signature}");
        let restricted = db::query_scalar::<bool>(
            "SELECT has_function_privilege('app_restricted', $1, 'EXECUTE')",
            vec![Bind::from(signature)],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(!restricted, "app_restricted は {signature} を実行できない");
    }
    for signature in [
        "public.admin_login_search_data_v2(text[],text[],text[],text,text,bigint,bigint)",
        "public.admin_login_search_count_v2(text[],text[],text[])",
        "public.scheduled_stock_price_targets_v2(date,bigint)",
        "public.scheduled_fund_price_targets_v2(date,bigint)",
        "public.scheduled_stock_price_mark_attempted_v2(text[])",
        "public.scheduled_stock_price_apply_v2(text,numeric,numeric,date)",
        "public.scheduled_fund_price_mark_attempted_v2(text[])",
        "public.scheduled_fund_price_apply_v2(text,numeric,numeric,date)",
        "public.scheduled_fund_price_record_failure_v2(text,text)",
        "public.scheduled_fund_price_clear_failure_v2(text)",
        "public.prune_login_events_older_than_90_days_v2()",
        "public.dividend_cache_enqueue_v2(varchar[])",
        "public.dividend_cache_apply_success_v2(text,numeric,varchar)",
        "public.dividend_cache_apply_error_v2(text,text,bigint)",
        "public.rate_control_acquire_jquants_v2()",
        "public.rate_control_cooldown_v2(text,bigint)",
        "public.stock_master_claim_v2()",
        "public.stock_master_postpone_v2()",
        "public.stock_master_replace_v2(date[],text[],text[],text[],text[],text[],text[])",
    ] {
        let executable = db::query_scalar::<bool>(
            "SELECT has_function_privilege(current_user, $1, 'EXECUTE')",
            vec![Bind::from(signature)],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(executable, "旧 postgres 接続で {signature} を実行できる");
    }
    let mut token = auth::upsert_user_and_rotate_session(&worker, &info)
        .await
        .expect("app_worker でログイン");
    let user_id = auth::select_user_id_by_session(&worker, token)
        .await
        .unwrap()
        .expect("app_worker でセッション照会");
    let notification_token = login_notify::issue_token(&worker, user_id)
        .await
        .expect("app_worker で通知 token を発行");
    for db_pool in [&pool, &worker] {
        let token = login_notify::issue_token(db_pool, user_id)
            .await
            .expect("通知 token を発行");
        assert_eq!(
            login_notify::peek_token(db_pool, &token)
                .await
                .unwrap()
                .map(|(id, _)| id),
            Some(user_id)
        );
        assert_eq!(
            login_notify::consume_token(db_pool, &token).await.unwrap(),
            Some(user_id)
        );
    }
    let owner_login = auth::upsert_user_and_rotate_session(&pool, &info)
        .await
        .expect("旧 postgres 接続でログイン");
    assert_eq!(
        auth::select_user_id_by_session(&pool, owner_login)
            .await
            .unwrap(),
        Some(user_id)
    );
    auth::delete_session(&pool, owner_login)
        .await
        .expect("旧 postgres 接続でログアウト");
    let worker_login = auth::upsert_user_and_rotate_session(&worker, &info)
        .await
        .expect("app_worker でログイン");
    assert_eq!(
        auth::select_user_id_by_session(&worker, worker_login)
            .await
            .unwrap(),
        Some(user_id)
    );
    auth::delete_session(&worker, worker_login)
        .await
        .expect("app_worker でログアウト");
    // 上のログイン/ログアウトで元のセッションは回転により消えるため取り直す。
    token = auth::upsert_user_and_rotate_session(&worker, &info)
        .await
        .expect("app_worker で再ログイン");
    db::query(
        "INSERT INTO login_events (user_id, result, device_hash, occurred_at) VALUES ($1, 'success', $2, NOW() - INTERVAL '100 days')",
        vec![Bind::from(user_id), Bind::Bytes(vec![9; 32])],
    )
    .execute(&pool)
    .await
    .unwrap();
    db::query(
        "INSERT INTO login_events (user_id, result, device_hash, occurred_at) VALUES ($1, 'success', $2, NOW())",
        vec![Bind::from(user_id), Bind::Bytes(vec![8; 32])],
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

    let force_info = GoogleUserInfo {
        sub: "force-login-after-rls".into(),
        email: "force-login-after-rls@example.com".into(),
        name: Some("FORCE 後ログイン".into()),
        picture_url: None,
    };
    let (force_token, force_user, _) = auth::upsert_user_rotate_session_with_login_event(
        &worker,
        &force_info,
        vec![7; 32],
        &ClientInfo {
            country: Some("JP".into()),
            region: Some("13".into()),
            browser: Some("chrome".into()),
            os: Some("linux".into()),
        },
    )
    .await
    .expect("FORCE 下の app_worker ログイン");
    assert_eq!(force_user.email, "force-login-after-rls@example.com");
    assert_eq!(
        auth::select_user_id_by_session(&worker, force_token)
            .await
            .unwrap(),
        Some(force_user.id)
    );
    auth::delete_session(&worker, force_token).await.unwrap();

    let executor_roles_are_limited: bool = db::query_scalar(
        "SELECT COUNT(*) = 5 AND bool_and(NOT rolcanlogin AND rolbypassrls)
         FROM pg_catalog.pg_roles WHERE rolname = ANY($1)",
        vec![Bind::from(vec![
            "app_notify_token_v2_executor".to_string(),
            "app_admin_login_executor".to_string(),
            "app_price_target_executor".to_string(),
            "app_login_event_prune_executor".to_string(),
            "app_shared_write_executor".to_string(),
        ])],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(executor_roles_are_limited);
    let restricted_executor_memberships: String = db::query_scalar(
        "SELECT COALESCE(string_agg(rolname || ':'
            || pg_catalog.pg_has_role('app_restricted', oid, 'MEMBER')::text || ':'
            || pg_catalog.pg_has_role('app_restricted', oid, 'SET')::text || ':'
            || pg_catalog.pg_has_role('app_restricted', oid, 'USAGE')::text, ', '), '')
         FROM pg_catalog.pg_roles
         WHERE rolname = ANY($1)
           AND (pg_catalog.pg_has_role('app_restricted', oid, 'MEMBER')
                OR pg_catalog.pg_has_role('app_restricted', oid, 'SET')
                OR pg_catalog.pg_has_role('app_restricted', oid, 'USAGE'))",
        vec![Bind::from(vec![
            "app_notify_token_v2_executor".to_string(),
            "app_admin_login_executor".to_string(),
            "app_price_target_executor".to_string(),
            "app_login_event_prune_executor".to_string(),
            "app_shared_write_executor".to_string(),
        ])],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        restricted_executor_memberships.is_empty(),
        "{restricted_executor_memberships}"
    );
    let function_acl_violations: Vec<(String,)> = db::query(
        "SELECT p.proname || ':' || owner.rolname || ':' ||
            has_function_privilege('app_restricted', p.oid, 'EXECUTE')::text || ':' ||
            has_function_privilege(
                CASE p.proname
                    WHEN 'login_notify_peek_v2' THEN 'app_notify_token_v2_executor'
                    WHEN 'login_notify_consume_v2' THEN 'app_notify_token_v2_executor'
                    WHEN 'admin_login_search_data_v2' THEN 'app_admin_login_executor'
                    WHEN 'admin_login_search_count_v2' THEN 'app_admin_login_executor'
                    WHEN 'scheduled_stock_price_targets_v2' THEN 'app_price_target_executor'
                    WHEN 'scheduled_fund_price_targets_v2' THEN 'app_price_target_executor'
                    WHEN 'scheduled_stock_price_mark_attempted_v2' THEN 'app_price_target_executor'
                    WHEN 'scheduled_stock_price_apply_v2' THEN 'app_price_target_executor'
                    WHEN 'scheduled_fund_price_mark_attempted_v2' THEN 'app_price_target_executor'
                    WHEN 'scheduled_fund_price_apply_v2' THEN 'app_price_target_executor'
                    WHEN 'scheduled_fund_price_record_failure_v2' THEN 'app_price_target_executor'
                    WHEN 'scheduled_fund_price_clear_failure_v2' THEN 'app_price_target_executor'
                    WHEN 'dividend_cache_enqueue_v2' THEN 'app_shared_write_executor'
                    WHEN 'dividend_cache_apply_success_v2' THEN 'app_shared_write_executor'
                    WHEN 'dividend_cache_apply_error_v2' THEN 'app_shared_write_executor'
                    WHEN 'rate_control_acquire_jquants_v2' THEN 'app_shared_write_executor'
                    WHEN 'rate_control_cooldown_v2' THEN 'app_shared_write_executor'
                    WHEN 'stock_master_claim_v2' THEN 'app_shared_write_executor'
                    WHEN 'stock_master_postpone_v2' THEN 'app_shared_write_executor'
                    WHEN 'stock_master_replace_v2' THEN 'app_shared_write_executor'
                    ELSE 'app_login_event_prune_executor'
                END, p.oid, 'EXECUTE')::text || ':' ||
            EXISTS (
                SELECT 1 FROM pg_catalog.aclexplode(COALESCE(p.proacl, pg_catalog.acldefault('f', p.proowner))) acl
                WHERE acl.privilege_type = 'EXECUTE'
                  AND acl.grantee <> p.proowner
                  AND acl.grantee <> 0
            )::text
         FROM pg_catalog.pg_proc p
         JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
         JOIN pg_catalog.pg_roles owner ON owner.oid = p.proowner
         WHERE n.nspname = 'public'
           AND p.proname = ANY($1)
           AND NOT (p.prosecdef
                AND p.proconfig @> ARRAY['search_path=pg_catalog, public, pg_temp', 'row_security=off']::text[]
                AND owner.rolbypassrls
                AND NOT owner.rolcanlogin
                AND has_function_privilege(
                    CASE p.proname
                        WHEN 'login_notify_peek_v2' THEN 'app_notify_token_v2_executor'
                        WHEN 'login_notify_consume_v2' THEN 'app_notify_token_v2_executor'
                        WHEN 'admin_login_search_data_v2' THEN 'app_admin_login_executor'
                        WHEN 'admin_login_search_count_v2' THEN 'app_admin_login_executor'
                        WHEN 'scheduled_stock_price_targets_v2' THEN 'app_price_target_executor'
                        WHEN 'scheduled_fund_price_targets_v2' THEN 'app_price_target_executor'
                        WHEN 'scheduled_stock_price_mark_attempted_v2' THEN 'app_price_target_executor'
                        WHEN 'scheduled_stock_price_apply_v2' THEN 'app_price_target_executor'
                        WHEN 'scheduled_fund_price_mark_attempted_v2' THEN 'app_price_target_executor'
                        WHEN 'scheduled_fund_price_apply_v2' THEN 'app_price_target_executor'
                        WHEN 'scheduled_fund_price_record_failure_v2' THEN 'app_price_target_executor'
                        WHEN 'scheduled_fund_price_clear_failure_v2' THEN 'app_price_target_executor'
                        WHEN 'dividend_cache_enqueue_v2' THEN 'app_shared_write_executor'
                        WHEN 'dividend_cache_apply_success_v2' THEN 'app_shared_write_executor'
                        WHEN 'dividend_cache_apply_error_v2' THEN 'app_shared_write_executor'
                        WHEN 'rate_control_acquire_jquants_v2' THEN 'app_shared_write_executor'
                        WHEN 'rate_control_cooldown_v2' THEN 'app_shared_write_executor'
                        WHEN 'stock_master_claim_v2' THEN 'app_shared_write_executor'
                        WHEN 'stock_master_postpone_v2' THEN 'app_shared_write_executor'
                        WHEN 'stock_master_replace_v2' THEN 'app_shared_write_executor'
                        ELSE 'app_login_event_prune_executor'
                    END, p.oid, 'EXECUTE')
                AND NOT EXISTS (
                    SELECT 1
                    FROM pg_catalog.aclexplode(COALESCE(p.proacl, pg_catalog.acldefault('f', p.proowner))) acl
                    WHERE acl.privilege_type = 'EXECUTE'
                      AND acl.grantee <> p.proowner
                      AND acl.grantee <> 0
                      AND acl.grantee <> (SELECT oid FROM pg_catalog.pg_roles WHERE rolname = 'app_restricted')
                      AND NOT (
                          acl.grantee = (SELECT oid FROM pg_catalog.pg_roles WHERE rolname = 'login_test_owner')
                          OR
                          p.proname IN (
                              'admin_login_search_data_v2', 'admin_login_search_count_v2',
                              'scheduled_stock_price_targets_v2', 'scheduled_fund_price_targets_v2',
                              'scheduled_stock_price_mark_attempted_v2', 'scheduled_stock_price_apply_v2',
                              'scheduled_fund_price_mark_attempted_v2', 'scheduled_fund_price_apply_v2',
                              'scheduled_fund_price_record_failure_v2', 'scheduled_fund_price_clear_failure_v2',
                              'prune_login_events_older_than_90_days_v2',
                              'dividend_cache_enqueue_v2', 'dividend_cache_apply_success_v2',
                              'dividend_cache_apply_error_v2', 'rate_control_acquire_jquants_v2',
                              'rate_control_cooldown_v2', 'stock_master_claim_v2',
                              'stock_master_postpone_v2',
                              'stock_master_replace_v2'
                          )
                          AND acl.grantee = (SELECT oid FROM pg_catalog.pg_roles WHERE rolname = 'app_worker')
                      )
                ))",
        vec![Bind::from(vec![
            "login_notify_peek_v2".to_string(),
            "login_notify_consume_v2".to_string(),
            "admin_login_search_data_v2".to_string(),
            "admin_login_search_count_v2".to_string(),
            "scheduled_stock_price_targets_v2".to_string(),
            "scheduled_fund_price_targets_v2".to_string(),
            "scheduled_stock_price_mark_attempted_v2".to_string(),
            "scheduled_stock_price_apply_v2".to_string(),
            "scheduled_fund_price_mark_attempted_v2".to_string(),
            "scheduled_fund_price_apply_v2".to_string(),
            "scheduled_fund_price_record_failure_v2".to_string(),
            "scheduled_fund_price_clear_failure_v2".to_string(),
            "prune_login_events_older_than_90_days_v2".to_string(),
            "dividend_cache_enqueue_v2".to_string(),
            "dividend_cache_apply_success_v2".to_string(),
            "dividend_cache_apply_error_v2".to_string(),
            "rate_control_acquire_jquants_v2".to_string(),
            "rate_control_cooldown_v2".to_string(),
            "stock_master_claim_v2".to_string(),
            "stock_master_postpone_v2".to_string(),
            "stock_master_replace_v2".to_string(),
        ])],
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        function_acl_violations.is_empty(),
        "{function_acl_violations:?}"
    );

    for statement in [
        "SELECT * FROM public.admin_login_search_data_v2(ARRAY[]::text[], ARRAY[]::text[], ARRAY[]::text[], 'occurred_at', 'desc', 10, 0)",
        "SELECT public.admin_login_search_count_v2(ARRAY[]::text[], ARRAY[]::text[], ARRAY[]::text[])",
        "SELECT * FROM public.scheduled_stock_price_targets_v2(CURRENT_DATE, 10)",
        "SELECT * FROM public.scheduled_fund_price_targets_v2(CURRENT_DATE, 10)",
        "SELECT public.scheduled_stock_price_mark_attempted_v2(ARRAY['99999'])",
        "SELECT public.scheduled_stock_price_apply_v2('99999', 10, 9, CURRENT_DATE)",
        "SELECT public.scheduled_fund_price_mark_attempted_v2(ARRAY['99999999'])",
        "SELECT public.scheduled_fund_price_apply_v2('99999999', 1.2, 0.01, CURRENT_DATE)",
        "SELECT public.scheduled_fund_price_record_failure_v2('99999999', 'test')",
        "SELECT public.scheduled_fund_price_clear_failure_v2('99999999')",
        "SELECT public.prune_login_events_older_than_90_days_v2()",
    ] {
        let mut restricted = worker.begin().await.unwrap();
        db::query("SELECT set_config('role', 'app_restricted', true)", vec![])
            .execute(&mut restricted)
            .await
            .unwrap();
        db::query(
            "SELECT set_config('app.current_user_id', $1, true)",
            vec![Bind::from(user_id.to_string())],
        )
        .execute(&mut restricted)
        .await
        .unwrap();
        let role_state: (String, String) =
            db::query("SELECT current_user::text, session_user::text", vec![])
                .fetch_one(&mut restricted)
                .await
                .unwrap();
        assert_eq!(role_state.0, "app_restricted");
        let can_execute: bool = db::query_scalar(
            "SELECT has_function_privilege(current_user, $1, 'EXECUTE')",
            vec![Bind::from(
                "public.admin_login_search_data_v2(text[],text[],text[],text,text,bigint,bigint)",
            )],
        )
        .fetch_one(&mut restricted)
        .await
        .unwrap();
        assert!(!can_execute, "app_restricted must not execute admin search");
        let role_setting: String = db::query_scalar("SELECT current_setting('role')", vec![])
            .fetch_one(&mut restricted)
            .await
            .unwrap();
        assert_eq!(role_setting, "app_restricted");
        let err = if statement.contains("admin_login_search_data") {
            db::query(statement, vec![])
                .fetch_all::<admin::AdminLoginEntry, _>(&mut restricted)
                .await
                .unwrap_err()
        } else if statement.contains("admin_login_search_count")
            || statement.contains("prune_login_events")
        {
            db::query(statement, vec![])
                .fetch_scalar::<i64, _>(&mut restricted)
                .await
                .unwrap_err()
        } else {
            db::query(statement, vec![])
                .fetch_all::<(String,), _>(&mut restricted)
                .await
                .unwrap_err()
        };
        assert_eq!(
            err.sql_state(),
            Some(db::SqlState::INSUFFICIENT_PRIVILEGE),
            "{statement}: {err}"
        );
        restricted.rollback().await.unwrap();
    }

    let owner_admin_count: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM login_events le JOIN users u ON u.id = le.user_id",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(owner_admin_count, 0);
    let owner_pruned = db::query(
        "DELETE FROM login_events WHERE occurred_at < NOW() - INTERVAL '90 days'",
        vec![],
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(owner_pruned, 0);

    assert_eq!(
        auth::select_user_id_by_session(&worker, token)
            .await
            .unwrap(),
        Some(user_id)
    );
    let token_exists: bool = db::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM login_notify_tokens WHERE token_hash = $1)",
        vec![Bind::Bytes(login_notify::token_hash(&notification_token))],
    )
    .fetch_one_with_user(&worker, user_id)
    .await
    .unwrap();
    assert!(token_exists);
    assert_eq!(
        login_notify::peek_token(&worker, &notification_token)
            .await
            .unwrap()
            .map(|(id, _)| id),
        Some(user_id)
    );
    assert_eq!(
        login_notify::consume_token(&worker, &notification_token)
            .await
            .unwrap(),
        Some(user_id)
    );
    assert!(
        login_notify::consume_token(&worker, &notification_token)
            .await
            .unwrap()
            .is_none()
    );
    auth::delete_session(&worker, token).await.unwrap();
    assert!(
        auth::select_user_id_by_session(&worker, token)
            .await
            .unwrap()
            .is_none()
    );
    let admin_results = admin::search(&worker, admin::AdminLoginQuery::default())
        .await
        .unwrap();
    assert_eq!(admin_results.total, 3);
    admin::record_view(&worker, user_id).await.unwrap();
    let audit_count: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM admin_login_views WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one_with_user(&pool, user_id)
    .await
    .unwrap();
    assert_eq!(audit_count, 1);
    assert_eq!(login_event::prune_login_events(&worker).await.unwrap(), 1);
    assert_eq!(
        admin::search(&pool, admin::AdminLoginQuery::default())
            .await
            .unwrap()
            .total,
        2
    );
    assert_eq!(login_event::prune_login_events(&pool).await.unwrap(), 0);
    assert_eq!(stock_price_targets_limited(&pool).await, ["99999"]);
    assert_eq!(fund_price_targets_limited(&pool).await, ["99999999"]);
    assert!(stock_price_targets(&pool).await.is_empty());
    assert!(fund_price_targets(&pool).await.is_empty());
    assert_eq!(stock_price_targets_limited(&worker).await, ["99999"]);
    assert_eq!(fund_price_targets_limited(&worker).await, ["99999999"]);
    let fund_write_access: bool = db::query_scalar(
        "SELECT has_table_privilege('app_price_target_executor', 'public.fund_prices', 'INSERT')
            AND has_table_privilege('app_price_target_executor', 'public.fund_prices', 'UPDATE')
            AND
            (SELECT rolname = 'app_price_target_executor' FROM pg_catalog.pg_roles WHERE oid =
                (SELECT proowner FROM pg_catalog.pg_proc WHERE proname = 'scheduled_fund_price_record_failure_v2'))",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(fund_write_access);
    db::query(
        "SELECT public.scheduled_stock_price_mark_attempted_v2(ARRAY['99999'])",
        vec![],
    )
    .execute(&worker)
    .await
    .unwrap();
    assert_eq!(
        stock_price::apply_close_price(
            &worker,
            &SecurityCode::from_raw("99999".into()),
            &ClosePrice {
                close: rust_decimal::Decimal::new(12345, 2),
                as_of: chrono::Utc::now().date_naive(),
                prev_close: Some(rust_decimal::Decimal::new(12000, 2)),
            },
        )
        .await
        .unwrap(),
        1
    );
    db::query(
        "SELECT public.scheduled_fund_price_mark_attempted_v2(ARRAY['99999999'])",
        vec![],
    )
    .execute(&worker)
    .await
    .unwrap();
    db::query(
        "SELECT public.scheduled_fund_price_record_failure_v2('99999999', 'test failure')",
        vec![],
    )
    .execute(&worker)
    .await
    .unwrap();
    db::query(
        "SELECT public.scheduled_fund_price_clear_failure_v2('99999999')",
        vec![],
    )
    .execute(&worker)
    .await
    .unwrap();
    assert_eq!(
        fund_price::apply_fund_nav(
            &worker,
            "99999999",
            &FundNav {
                nav: rust_decimal::Decimal::new(125, 2),
                prev_day_change: rust_decimal::Decimal::new(1, 2),
                base_date: chrono::Utc::now().date_naive(),
            },
        )
        .await
        .unwrap(),
        1
    );
    assert!(stock_price_targets_as(&pool, user_id).await.is_empty());
    assert!(fund_price_targets_as(&pool, user_id).await.is_empty());
    // 共有表の読み書きは app_worker・旧 postgres のどちらの接続でも限定経路で動く。
    db::query(
        "INSERT INTO stock (date, code, name, market_category) VALUES (CURRENT_DATE, '9999', '架空の銘柄', 'プライム')",
        vec![],
    )
    .execute(&pool)
    .await
    .unwrap();
    let mut dividend_codes = Vec::new();
    for db_pool in [&pool, &worker] {
        let dividend_code = if std::ptr::eq(db_pool, &pool) {
            "9998"
        } else {
            "9997"
        };
        dividend_codes.push(dividend_code);
        let found = stock::search(db_pool, "9999")
            .await
            .expect("銘柄検索は通る");
        assert_eq!(found.code.as_str(), "9999");
        let client = JQuantsClient::new("test-key".to_string());
        let items = dividend_cache::get_batch(
            db_pool,
            Some(&client),
            &[SecurityCode::from_raw(dividend_code.into())],
        )
        .await
        .expect("未取得配当の取得とキュー投入は通る");
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].status,
            shared::dividend_per_share::DividendCacheStatus::Pending
        );
        let pending: Vec<(String,)> = db::query(
            "SELECT security_code FROM dividend_per_share_cache WHERE security_code = $1 AND status = 'pending'",
            vec![Bind::from(dividend_code)],
        )
        .fetch_all(db_pool)
        .await
        .unwrap();
        assert_eq!(pending.len(), 1, "pending 行をどちらの接続からも確認できる");
        db::query(
            "SELECT public.dividend_cache_enqueue_v2($1)",
            vec![Bind::from(vec![SecurityCode::from_raw("9999".into())])],
        )
        .execute(db_pool)
        .await
        .expect("配当キュー投入は通る");
        db::query(
            "SELECT public.dividend_cache_apply_error_v2($1, $2, $3)",
            vec![Bind::from("9999"), Bind::from("test"), Bind::from(60i64)],
        )
        .execute(db_pool)
        .await
        .expect("配当エラー記録は通る");
    }
    let client = JQuantsClient::new("test-key".to_string());
    let processed = dividend_cache::drain_refresh_queue(&worker, &client)
        .await
        .expect("app_worker の cron drain は enqueue 行を選定できる");
    assert_eq!(processed, 2, "app_worker cron drain が2件を処理対象にする");
    for code in dividend_codes {
        let status: (String,) = db::query(
            "SELECT status FROM dividend_per_share_cache WHERE security_code = $1",
            vec![Bind::from(code)],
        )
        .fetch_one(&worker)
        .await
        .unwrap();
        assert_eq!(status.0, "error", "テスト環境の上流取得失敗を記録する");
    }
    let cached: Vec<(String,)> = db::query(
        "SELECT security_code FROM dividend_per_share_cache WHERE security_code = '9999'",
        vec![],
    )
    .fetch_all(&worker)
    .await
    .unwrap();
    assert_eq!(cached.len(), 1, "app_worker は共有キャッシュを読める");
    dividend_cache::acquire_rate_slot(&worker)
        .await
        .expect("レート枠の取得は通る");
    let claimed = db::query_scalar::<i32>("SELECT public.stock_master_claim_v2()", vec![])
        .fetch_optional(&worker)
        .await
        .unwrap();
    assert!(claimed.is_some(), "初回の銘柄マスタ枠取りは通る");
    let claimed_again = db::query_scalar::<i32>("SELECT public.stock_master_claim_v2()", vec![])
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(
        claimed_again.is_none(),
        "枠取り済みのため旧 postgres 接続の2回目は空振り"
    );
    for db_pool in [&pool, &worker] {
        let replaced = stock_master::replace_master(
            db_pool,
            &[stock_master::StockRow {
                date: chrono::Utc::now().date_naive(),
                code: "9999".to_string(),
                name: "架空の銘柄".to_string(),
                market_category: "プライム".to_string(),
                industry_category_33: None,
                industry_category_17: None,
                size_category: None,
            }],
        )
        .await
        .expect("銘柄マスタ置換は通る");
        assert_eq!(replaced, 1);
    }
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
        None
    );
    pool.batch_execute("ALTER TABLE sessions FORCE ROW LEVEL SECURITY")
        .await
        .unwrap();
    assert_eq!(
        auth::delete_all_sessions(&worker, user_id).await.unwrap(),
        0
    );
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
    let (pool, _, _, _node) = owner_pool().await;
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
    let (pool, _, admin, _node) = owner_pool().await;
    admin
        .batch_execute(
            "REVOKE app_restricted FROM login_test_owner CASCADE;
             GRANT app_restricted TO login_test_owner WITH ADMIN TRUE, SET FALSE, INHERIT FALSE",
        )
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
    let (pool, _, _, _node) = owner_pool().await;
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
    let (pool, _, _, _node) = owner_pool().await;

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
