#![allow(clippy::unwrap_used, clippy::expect_used)]

use backend::services::csv::pipeline::CsvImport;
use backend::test_constants::POSTGRES_TAG;
use backend::{
    db::{self, Bind, Db, Executor, run_migrations},
    models::common::{PaginationParams, SearchQueryParams},
    models::fund_balance::FundBalanceSearchQueryParams,
    models::user::GoogleUserInfo,
    services::asset_balance as asset_balance_svc,
    services::auth::{self as auth_svc, SessionToken},
    services::dividend as dividend_svc,
    services::domain::bulk::{self, RowLimit},
    services::domain::search::search as domain_search,
    services::domestic_stock as domestic_stock_svc,
    services::fund_balance::FundBalanceDomain,
    services::mutualfund as mutualfund_svc,
};
use rust_decimal_macros::dec;
use shared::value::UserId;
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use time::{Date, OffsetDateTime};

mod common;

use common::*;

/// RLS 第一段階: `csv_import_history` は所有者の挙動を変えず、
/// 権限を絞ったロールでは利用者ごとに分離される
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn csv_import_history_rls_isolates_restricted_role() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");

    let rls = db::query_scalar::<bool>(
        "SELECT relrowsecurity FROM pg_class WHERE relname = 'csv_import_history'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("RLS 有効の取得に失敗");
    assert!(rls, "csv_import_history に RLS が有効のはず");
    let forced = db::query_scalar::<bool>(
        "SELECT relforcerowsecurity FROM pg_class WHERE relname = 'csv_import_history'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("FORCE 状態の取得に失敗");
    assert!(!forced, "FORCE は付けない(所有者の挙動を変えない)");

    let user_a = create_test_user(&pool).await;
    let user_b = create_test_user(&pool).await;
    for user_id in [user_a, user_b] {
        db::query(
            "INSERT INTO csv_import_history (user_id, kind) VALUES ($1, 'dividend')",
            vec![Bind::from(user_id)],
        )
        .execute(&pool)
        .await
        .expect("履歴の投入に失敗");
    }
    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM csv_import_history", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2, "所有者は両利用者の行を読めるはず");

    pool.batch_execute("CREATE ROLE rls_app WITH LOGIN PASSWORD 'rls-test-password'")
        .await
        .expect("制限ロールの作成に失敗");
    pool.batch_execute("GRANT CONNECT ON DATABASE postgres TO rls_app")
        .await
        .expect("CONNECT 付与に失敗");
    pool.batch_execute("GRANT USAGE ON SCHEMA public TO rls_app")
        .await
        .expect("USAGE 付与に失敗");
    pool.batch_execute("GRANT SELECT, INSERT, UPDATE, DELETE ON csv_import_history TO rls_app")
        .await
        .expect("表権限の付与に失敗");
    let restricted_url = format!("postgres://rls_app:rls-test-password@127.0.0.1:{port}/postgres");
    let restricted = connect_with_retry(&restricted_url).await;

    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM csv_import_history", vec![])
        .fetch_one(&restricted)
        .await
        .unwrap();
    assert_eq!(count, 0, "利用者 ID 未設定の制限ロールは何も見えないはず");

    let mut tx = restricted.begin().await.expect("tx 開始に失敗");
    db::query(
        "SELECT set_config('app.current_user_id', $1, true)",
        vec![Bind::from(user_a.to_string())],
    )
    .execute(&mut tx)
    .await
    .expect("利用者 ID の設定に失敗");
    let kinds = db::query("SELECT kind FROM csv_import_history", vec![])
        .fetch_all::<(String,), _>(&mut tx)
        .await
        .expect("履歴の取得に失敗");
    assert_eq!(
        kinds,
        vec![("dividend".to_string(),)],
        "利用者 A の行だけが見えるはず"
    );
    db::query(
        "INSERT INTO csv_import_history (user_id, kind) VALUES ($1, 'assetbalance')",
        vec![Bind::from(user_a)],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    let err = db::query(
        "INSERT INTO csv_import_history (user_id, kind) VALUES ($1, 'assetbalance')",
        vec![Bind::from(user_b)],
    )
    .execute(&mut tx)
    .await
    .expect_err("他人名義の書き込みは失敗するはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "行共有違反のはず: {err:?}"
    );
    tx.rollback().await.expect("rollback に失敗");
}

/// RLS 第二段階: 監査系 3 表は所有者の挙動を変えず、
/// 権限を絞ったロールでは利用者ごとに分離される
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn audit_tables_rls_isolates_restricted_role() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");

    for table in ["login_events", "admin_login_views", "login_notify_tokens"] {
        let rls = db::query_scalar::<bool>(
            "SELECT relrowsecurity FROM pg_class WHERE relname = $1",
            vec![Bind::from(table)],
        )
        .fetch_one(&pool)
        .await
        .expect("RLS 有効の取得に失敗");
        assert!(rls, "{table} に RLS が有効のはず");
        let forced = db::query_scalar::<bool>(
            "SELECT relforcerowsecurity FROM pg_class WHERE relname = $1",
            vec![Bind::from(table)],
        )
        .fetch_one(&pool)
        .await
        .expect("FORCE 状態の取得に失敗");
        assert!(
            !forced,
            "{table} に FORCE は付けない(所有者の挙動を変えない)"
        );
    }

    let user_a = create_test_user(&pool).await;
    let user_b = create_test_user(&pool).await;
    for (user_id, device_byte, token_byte) in [(user_a, 41u8, 42u8), (user_b, 43u8, 44u8)] {
        db::query(
            "INSERT INTO login_events (user_id, result, device_hash) VALUES ($1, 'success', $2)",
            vec![Bind::from(user_id), Bind::Bytes(vec![device_byte; 32])],
        )
        .execute(&pool)
        .await
        .expect("ログイン記録の投入に失敗");
        db::query(
            "INSERT INTO admin_login_views (user_id) VALUES ($1)",
            vec![Bind::from(user_id)],
        )
        .execute(&pool)
        .await
        .expect("閲覧記録の投入に失敗");
        db::query(
            "INSERT INTO login_notify_tokens (user_id, token_hash) VALUES ($1, $2)",
            vec![Bind::from(user_id), Bind::Bytes(vec![token_byte; 32])],
        )
        .execute(&pool)
        .await
        .expect("通知トークンの投入に失敗");
    }
    for (table, expected) in [
        ("login_events", 2i64),
        ("admin_login_views", 2i64),
        ("login_notify_tokens", 2i64),
    ] {
        let count = db::query_scalar::<i64>(format!("SELECT COUNT(*) FROM {table}"), vec![])
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, expected, "所有者は両利用者の行を読めるはず: {table}");
    }

    pool.batch_execute("CREATE ROLE rls_app WITH LOGIN PASSWORD 'rls-test-password'")
        .await
        .expect("制限ロールの作成に失敗");
    pool.batch_execute("GRANT CONNECT ON DATABASE postgres TO rls_app")
        .await
        .expect("CONNECT 付与に失敗");
    pool.batch_execute("GRANT USAGE ON SCHEMA public TO rls_app")
        .await
        .expect("USAGE 付与に失敗");
    for table in ["login_events", "admin_login_views", "login_notify_tokens"] {
        pool.batch_execute(&format!(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON {table} TO rls_app"
        ))
        .await
        .expect("表権限の付与に失敗");
    }
    let restricted_url = format!("postgres://rls_app:rls-test-password@127.0.0.1:{port}/postgres");
    let restricted = connect_with_retry(&restricted_url).await;

    for table in ["login_events", "admin_login_views", "login_notify_tokens"] {
        let count = db::query_scalar::<i64>(format!("SELECT COUNT(*) FROM {table}"), vec![])
            .fetch_one(&restricted)
            .await
            .unwrap();
        assert_eq!(
            count, 0,
            "利用者 ID 未設定の制限ロールは何も見えないはず: {table}"
        );
    }

    let mut tx = restricted.begin().await.expect("tx 開始に失敗");
    db::query(
        "SELECT set_config('app.current_user_id', $1, true)",
        vec![Bind::from(user_a.to_string())],
    )
    .execute(&mut tx)
    .await
    .expect("利用者 ID の設定に失敗");
    for table in ["login_events", "admin_login_views", "login_notify_tokens"] {
        let owners = db::query(format!("SELECT user_id::text FROM {table}"), vec![])
            .fetch_all::<(String,), _>(&mut tx)
            .await
            .expect("監査行の取得に失敗");
        assert_eq!(
            owners,
            vec![(user_a.to_string(),)],
            "利用者 A の行だけが見えるはず: {table}"
        );
    }
    db::query(
        "INSERT INTO login_events (user_id, result, device_hash) VALUES ($1, 'success', $2)",
        vec![Bind::from(user_a), Bind::Bytes(vec![45u8; 32])],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    db::query(
        "INSERT INTO admin_login_views (user_id) VALUES ($1)",
        vec![Bind::from(user_a)],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    db::query(
        "INSERT INTO login_notify_tokens (user_id, token_hash) VALUES ($1, $2)",
        vec![Bind::from(user_a), Bind::Bytes(vec![46u8; 32])],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    let err = db::query(
        "INSERT INTO login_events (user_id, result, device_hash) VALUES ($1, 'success', $2)",
        vec![Bind::from(user_b), Bind::Bytes(vec![47u8; 32])],
    )
    .execute(&mut tx)
    .await
    .expect_err("他人名義の書き込みは失敗するはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "行共有違反のはず: {err:?}"
    );
    tx.rollback().await.expect("rollback に失敗");

    for (table, binds, sql) in [
        (
            "admin_login_views",
            vec![Bind::from(user_b)],
            "INSERT INTO admin_login_views (user_id) VALUES ($1)",
        ),
        (
            "login_notify_tokens",
            vec![Bind::from(user_b), Bind::Bytes(vec![48u8; 32])],
            "INSERT INTO login_notify_tokens (user_id, token_hash) VALUES ($1, $2)",
        ),
    ] {
        let mut tx = restricted.begin().await.expect("tx 開始に失敗");
        db::query(
            "SELECT set_config('app.current_user_id', $1, true)",
            vec![Bind::from(user_a.to_string())],
        )
        .execute(&mut tx)
        .await
        .expect("利用者 ID の設定に失敗");
        let err = db::query(sql, binds)
            .execute(&mut tx)
            .await
            .expect_err("他人名義の書き込みは失敗するはず");
        assert!(
            err.sql_state()
                .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
            "行共有違反のはず({table}): {err:?}"
        );
        tx.rollback().await.expect("rollback に失敗");
    }
}

/// RLS 第三段階(前半): 取引明細・残高の5表は所有者の挙動を変えず、
/// 権限を絞ったロールでは利用者ごとに分離される
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn trading_tables_rls_isolates_restricted_role() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");

    for table in [
        "dividends",
        "domestic_stocks",
        "mutualfunds",
        "asset_balances",
        "fund_balances",
    ] {
        let rls = db::query_scalar::<bool>(
            "SELECT relrowsecurity FROM pg_class WHERE relname = $1",
            vec![Bind::from(table)],
        )
        .fetch_one(&pool)
        .await
        .expect("RLS 有効の取得に失敗");
        assert!(rls, "{table} に RLS が有効のはず");
        let forced = db::query_scalar::<bool>(
            "SELECT relforcerowsecurity FROM pg_class WHERE relname = $1",
            vec![Bind::from(table)],
        )
        .fetch_one(&pool)
        .await
        .expect("FORCE 状態の取得に失敗");
        assert!(
            !forced,
            "{table} に FORCE は付けない(所有者の挙動を変えない)"
        );
    }

    let user_a = create_test_user(&pool).await;
    let user_b = create_test_user(&pool).await;
    dividend_svc::bulk_create(
        &pool,
        user_a,
        &[make_dividend_item("9931")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当の投入に失敗");
    dividend_svc::bulk_create(
        &pool,
        user_b,
        &[make_dividend_item("9932")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当の投入に失敗");
    domestic_stock_svc::bulk_create(
        &pool,
        user_a,
        &[make_domestic_stock_item("9935")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("国内株式の投入に失敗");
    domestic_stock_svc::bulk_create(
        &pool,
        user_b,
        &[make_domestic_stock_item("9936")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("国内株式の投入に失敗");
    mutualfund_svc::bulk_create(
        &pool,
        user_a,
        &[make_mutualfund_item("架空投信A")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("投信の投入に失敗");
    mutualfund_svc::bulk_create(
        &pool,
        user_b,
        &[make_mutualfund_item("架空投信B")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("投信の投入に失敗");
    asset_balance_svc::bulk_create(
        &pool,
        user_a,
        &[make_asset_item("9931")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("残高の投入に失敗");
    asset_balance_svc::bulk_create(
        &pool,
        user_b,
        &[make_asset_item("9901")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("残高の投入に失敗");
    let today = (OffsetDateTime::now_utc() + time::Duration::hours(9)).date();
    insert_fund_row(
        &pool,
        user_a,
        "架空残高投信A",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        Some(dec!(5)),
        dec!(100000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;
    insert_fund_row(
        &pool,
        user_b,
        "架空残高投信B",
        dec!(10000),
        dec!(100000),
        dec!(10000),
        Some(dec!(5)),
        dec!(100000),
        dec!(0),
        today,
        OffsetDateTime::now_utc(),
    )
    .await;

    let uploaded = dividend_svc::DividendDomain::upload_csv(
        &pool,
        user_a,
        make_dividend_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当 CSV 取込は通るはず");
    assert_eq!(uploaded.inserted, 1);
    let uploaded = domestic_stock_svc::DomesticStockDomain::upload_csv(
        &pool,
        user_a,
        make_domestic_stock_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("国内株式 CSV 取込は通るはず");
    assert_eq!(uploaded.inserted, 1);
    let uploaded = mutualfund_svc::MutualfundDomain::upload_csv(
        &pool,
        user_a,
        make_mutualfund_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("投信 CSV 取込は通るはず");
    assert_eq!(uploaded.inserted, 1);
    let uploaded = asset_balance_svc::AssetBalanceDomain::upload_csv(
        &pool,
        user_a,
        make_asset_balance_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("残高 CSV 取込は通るはず");
    assert_eq!(uploaded.inserted, 1);
    let uploaded = FundBalanceDomain::upload_csv(
        &pool,
        user_a,
        concat!(
            "投資信託種別,口座区分,ファンド,分配金コース,保有数量[口],平均取得価額[円],取得総額[円],基準価額[円],時価評価額[円],評価損益[円]\n",
            "投資信託,特定,架空長期積立ファンド,再投資型,10000,10000,10000,11000,11000,1000"
        )
        .as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("ファンド残高 CSV 取込は通るはず");
    assert_eq!(uploaded.inserted, 1);

    for (table, expected) in [
        ("dividends", 3i64),
        ("domestic_stocks", 3i64),
        ("mutualfunds", 3i64),
        ("asset_balances", 2i64),
        ("fund_balances", 2i64),
    ] {
        let count = db::query_scalar::<i64>(format!("SELECT COUNT(*) FROM {table}"), vec![])
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, expected, "所有者は両利用者の行を読めるはず: {table}");
    }

    let dividends_a = domain_search::<dividend_svc::DividendDomain>(
        &pool,
        user_a,
        default_dividend_search_params(),
    )
    .await
    .expect("配当一覧は通るはず");
    assert_eq!(dividends_a.data.len(), 2);
    let mut dividend_summary_params = default_dividend_search_params();
    dividend_summary_params.search.include_summary = Some(true);
    let dividends_a =
        domain_search::<dividend_svc::DividendDomain>(&pool, user_a, dividend_summary_params)
            .await
            .expect("配当集計は通るはず");
    assert!(dividends_a.summary.is_some(), "配当集計が返るはず");
    let domestic_a = domain_search::<domestic_stock_svc::DomesticStockDomain>(
        &pool,
        user_a,
        default_domestic_stock_search_params(),
    )
    .await
    .expect("国内株式一覧は通るはず");
    assert_eq!(domestic_a.data.len(), 2);
    let mutualfund_a = domain_search::<mutualfund_svc::MutualfundDomain>(
        &pool,
        user_a,
        default_mutualfund_search_params(),
    )
    .await
    .expect("投信一覧は通るはず");
    assert_eq!(mutualfund_a.data.len(), 2);
    let mut asset_params = default_asset_balance_search_params();
    asset_params.search.include_summary = Some(true);
    let asset_a =
        domain_search::<asset_balance_svc::AssetBalanceDomain>(&pool, user_a, asset_params)
            .await
            .expect("残高一覧・集計は通るはず");
    assert_eq!(asset_a.data.len(), 1);
    assert!(asset_a.summary.is_some(), "残高集計が返るはず");
    let fund_a = domain_search::<FundBalanceDomain>(
        &pool,
        user_a,
        FundBalanceSearchQueryParams {
            search: SearchQueryParams {
                include_summary: Some(true),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await
    .expect("ファンド残高一覧・集計は通るはず");
    assert_eq!(fund_a.data.len(), 1);
    assert!(fund_a.summary.is_some(), "ファンド残高集計が返るはず");

    pool.batch_execute("CREATE ROLE rls_app WITH LOGIN PASSWORD 'rls-test-password'")
        .await
        .expect("制限ロールの作成に失敗");
    pool.batch_execute("GRANT CONNECT ON DATABASE postgres TO rls_app")
        .await
        .expect("CONNECT 付与に失敗");
    pool.batch_execute("GRANT USAGE ON SCHEMA public TO rls_app")
        .await
        .expect("USAGE 付与に失敗");
    for table in [
        "dividends",
        "domestic_stocks",
        "mutualfunds",
        "asset_balances",
        "fund_balances",
    ] {
        pool.batch_execute(&format!(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON {table} TO rls_app"
        ))
        .await
        .expect("表権限の付与に失敗");
    }
    let restricted_url = format!("postgres://rls_app:rls-test-password@127.0.0.1:{port}/postgres");
    let restricted = connect_with_retry(&restricted_url).await;

    for table in [
        "dividends",
        "domestic_stocks",
        "mutualfunds",
        "asset_balances",
        "fund_balances",
    ] {
        let count = db::query_scalar::<i64>(format!("SELECT COUNT(*) FROM {table}"), vec![])
            .fetch_one(&restricted)
            .await
            .unwrap();
        assert_eq!(
            count, 0,
            "利用者 ID 未設定の制限ロールは何も見えないはず: {table}"
        );
    }

    let mut tx = restricted.begin().await.expect("tx 開始に失敗");
    db::query(
        "SELECT set_config('app.current_user_id', $1, true)",
        vec![Bind::from(user_a.to_string())],
    )
    .execute(&mut tx)
    .await
    .expect("利用者 ID の設定に失敗");
    for (table, expected) in [
        ("dividends", 2usize),
        ("domestic_stocks", 2usize),
        ("mutualfunds", 2usize),
        ("asset_balances", 1usize),
        ("fund_balances", 1usize),
    ] {
        let owners = db::query(format!("SELECT user_id::text FROM {table}"), vec![])
            .fetch_all::<(String,), _>(&mut tx)
            .await
            .expect("明細・残高の取得に失敗");
        assert_eq!(
            owners.len(),
            expected,
            "利用者 A の行数が合わないはず: {table}"
        );
        assert!(
            owners.iter().all(|(id,)| id == &user_a.to_string()),
            "利用者 A の行だけが見えるはず: {table}"
        );
    }
    db::query(
        "INSERT INTO dividends (user_id, settlement_date, product, account, security_code, \
         security_name, unit_price, shares, dividends_before_tax, taxes, net_amount_received, \
         content_hash, occurrence_index) \
         VALUES ($1, '2024-04-01', '国内株式', '特定', '9921', '架空自己配当', 100, 50, 5000, 1015, 3985, 'rls-own-div-hash-a', 0)",
        vec![Bind::from(user_a)],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    db::query(
        "INSERT INTO domestic_stocks (user_id, trade_date, settlement_date, security_code, \
         security_name, account, shares, asked_price, proceeds, purchase_price, \
         realized_profit_and_loss, taxes, realized_profit_and_loss_after_tax, content_hash, \
         occurrence_index) VALUES ($1, '2024-03-20', '2024-03-25', '9941', '架空自己株式', '特定', \
         10, 1000, 10000, 9000, 1000, 203, 797, 'rls-own-hash-a', 0)",
        vec![Bind::from(user_a)],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    db::query(
        "INSERT INTO mutualfunds (user_id, trade_date, settlement_date, fund_name, account, shares, \
         exchange_rate, cancellation_unit_price_yen, cancellation_amount_yen, \
         average_acquisition_price_yen, realized_profit_and_loss, taxes, \
         realized_profit_and_loss_after_tax, content_hash, occurrence_index) VALUES ($1, '2024-03-20', '2024-03-25', \
         '架空自己投信A', '特定', 1000, 1, 12000, 12000000, 10000, 2000000, 406060, 1593940, 'rls-own-mf-hash-a', 0)",
        vec![Bind::from(user_a)],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    db::query(
        "INSERT INTO asset_balances (user_id, security_code, security_name, shares, \
         average_purchase_price, total_purchase_amount) \
         VALUES ($1, '9911', '架空自己残高', 100, 1000, 100000)",
        vec![Bind::from(user_a)],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    db::query(
        "INSERT INTO fund_balances (user_id, fund_name, account, shares, average_purchase_price, \
         total_purchase_amount, base_price, market_value, profit_loss, price_as_of) \
         VALUES ($1, '架空自己残高投信A', '特定', 10000, 10000, 100000, 11000, 110000, 10000, '2024-03-25')",
        vec![Bind::from(user_a)],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    let err = db::query(
        "INSERT INTO dividends (user_id, settlement_date, product, account, security_code, \
         security_name, unit_price, shares, dividends_before_tax, taxes, net_amount_received, \
         content_hash, occurrence_index) \
         VALUES ($1, '2024-04-01', '国内株式', '特定', '9922', '架空他人配当', 100, 50, 5000, 1015, 3985, 'rls-other-div-hash-b', 0)",
        vec![Bind::from(user_b)],
    )
    .execute(&mut tx)
    .await
    .expect_err("他人名義の書き込みは失敗するはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "行共有違反のはず: {err:?}"
    );
    tx.rollback().await.expect("rollback に失敗");

    for (table, binds, sql) in [
        (
            "domestic_stocks",
            vec![Bind::from(user_b)],
            "INSERT INTO domestic_stocks (user_id, trade_date, settlement_date, security_code, \
             security_name, account, shares, asked_price, proceeds, purchase_price, \
             realized_profit_and_loss, taxes, realized_profit_and_loss_after_tax, content_hash, \
             occurrence_index) VALUES ($1, '2024-03-20', '2024-03-25', '9942', '架空他人株式', '特定', \
             10, 1000, 10000, 9000, 1000, 203, 797, 'rls-other-hash-b', 0)",
        ),
        (
            "mutualfunds",
            vec![Bind::from(user_b)],
            "INSERT INTO mutualfunds (user_id, trade_date, settlement_date, fund_name, account, shares, \
             exchange_rate, cancellation_unit_price_yen, cancellation_amount_yen, \
             average_acquisition_price_yen, realized_profit_and_loss, taxes, \
             realized_profit_and_loss_after_tax, content_hash, occurrence_index) VALUES ($1, '2024-03-20', '2024-03-25', \
             '架空他人投信B', '特定', 1000, 1, 12000, 12000000, 10000, 2000000, 406060, 1593940, 'rls-other-mf-hash-b', 0)",
        ),
        (
            "asset_balances",
            vec![Bind::from(user_b)],
            "INSERT INTO asset_balances (user_id, security_code, security_name, shares, \
             average_purchase_price, total_purchase_amount) \
             VALUES ($1, '9912', '架空他人残高', 100, 1000, 100000)",
        ),
        (
            "fund_balances",
            vec![Bind::from(user_b)],
            "INSERT INTO fund_balances (user_id, fund_name, account, shares, average_purchase_price, \
             total_purchase_amount, base_price, market_value, profit_loss, price_as_of) \
             VALUES ($1, '架空他人残高投信B', '特定', 10000, 10000, 100000, 11000, 110000, 10000, '2024-03-25')",
        ),
    ] {
        let mut tx = restricted.begin().await.expect("tx 開始に失敗");
        db::query(
            "SELECT set_config('app.current_user_id', $1, true)",
            vec![Bind::from(user_a.to_string())],
        )
        .execute(&mut tx)
        .await
        .expect("利用者 ID の設定に失敗");
        let err = db::query(sql, binds)
            .execute(&mut tx)
            .await
            .expect_err("他人名義の書き込みは失敗するはず");
        assert!(
            err.sql_state()
                .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
            "行共有違反のはず({table}): {err:?}"
        );
        tx.rollback().await.expect("rollback に失敗");
    }
}

/// RLS 第三段階(後半): 書き込み経路は `begin_with_user` で開いた
/// トランザクション内で動くため、制限ロールでも本人の行だけが書ける。
/// 読み取り経路の配線は接続ロールの切り替え段階まで見送り、ここでは所有者側で挙動が変わらないことだけ見る。
/// 未認証経路は設定を残さない。値はトランザクションをまたいで残らない
/// (接続に残る空文字のプレースホルダは利用者の値ではなく、比較では不一致になる)
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn rls_set_local_wiring_scopes_authenticated_paths() {
    use backend::services::login_event::admin::record_view as record_admin_view;

    async fn current_setting(executor: &Db) -> Option<String> {
        db::query_scalar::<Option<String>>(
            "SELECT current_setting('app.current_user_id', true)",
            vec![],
        )
        .fetch_one(executor)
        .await
        .expect("設定値の取得に失敗")
    }
    async fn assert_no_user_setting(executor: &Db) {
        let setting = current_setting(executor).await;
        assert!(
            setting.as_deref().is_none_or(str::is_empty),
            "別利用者の値が残ってはいけない: {setting:?}"
        );
    }

    use backend::services::login_event::list_login_history;
    use backend::services::login_notify::{
        consume_token, get_setting, issue_token, peek_token, set_setting,
    };

    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");

    let user_a = create_test_user(&pool).await;
    let user_b = create_test_user(&pool).await;
    let user_c = create_test_user(&pool).await;

    dividend_svc::bulk_create(
        &pool,
        user_a,
        &[make_dividend_item("9931")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当の投入に失敗");
    dividend_svc::bulk_create(
        &pool,
        user_b,
        &[make_dividend_item("9932")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当の投入に失敗");
    dividend_svc::bulk_create(
        &pool,
        user_c,
        &[make_dividend_item("9933")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当の投入に失敗");
    asset_balance_svc::bulk_create(
        &pool,
        user_a,
        &[make_asset_item("9931")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("残高の投入に失敗");
    asset_balance_svc::bulk_create(
        &pool,
        user_b,
        &[make_asset_item("9901")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("残高の投入に失敗");

    for (user_id, device_byte) in [(&user_a, 61u8), (&user_b, 62u8)] {
        db::query(
            "INSERT INTO login_events (user_id, result, device_hash) VALUES ($1, 'success', $2)",
            vec![Bind::from(*user_id), Bind::Bytes(vec![device_byte; 32])],
        )
        .execute(&pool)
        .await
        .expect("ログイン記録の投入に失敗");
    }

    pool.batch_execute("CREATE ROLE rls_app WITH LOGIN PASSWORD 'rls-test-password'")
        .await
        .expect("制限ロールの作成に失敗");
    pool.batch_execute("GRANT CONNECT ON DATABASE postgres TO rls_app")
        .await
        .expect("CONNECT 付与に失敗");
    pool.batch_execute("GRANT USAGE ON SCHEMA public TO rls_app")
        .await
        .expect("USAGE 付与に失敗");
    for table in [
        "dividends",
        "domestic_stocks",
        "mutualfunds",
        "asset_balances",
        "fund_balances",
        "csv_import_history",
        "login_events",
        "admin_login_views",
        "login_notify_tokens",
    ] {
        pool.batch_execute(&format!(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON {table} TO rls_app"
        ))
        .await
        .expect("表権限の付与に失敗");
    }
    pool.batch_execute("GRANT SELECT, UPDATE ON users TO rls_app")
        .await
        .expect("users の参照・更新付与に失敗");
    pool.batch_execute("GRANT EXECUTE ON FUNCTION public.app_enter(uuid) TO rls_app")
        .await
        .expect("app_enter の実行付与に失敗");
    pool.batch_execute("GRANT app_restricted TO rls_app WITH SET TRUE, INHERIT FALSE")
        .await
        .expect("制限ロールへの切替付与に失敗");
    let restricted_url = format!("postgres://rls_app:rls-test-password@127.0.0.1:{port}/postgres");
    let restricted = connect_with_retry(&restricted_url).await;

    assert_no_user_setting(&restricted).await;

    let mut tx = restricted
        .begin_with_user(user_a)
        .await
        .expect("利用者付き tx の開始に失敗");
    let setting = db::query_scalar::<Option<String>>(
        "SELECT current_setting('app.current_user_id', true)",
        vec![],
    )
    .fetch_one(&mut tx)
    .await
    .expect("tx 内の設定値の取得に失敗");
    assert_eq!(
        setting,
        Some(user_a.to_string()),
        "tx 内では利用者 ID が設定されているはず"
    );
    tx.rollback().await.expect("rollback に失敗");
    assert_no_user_setting(&restricted).await;

    let uploaded = dividend_svc::DividendDomain::upload_csv(
        &restricted,
        user_a,
        make_dividend_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("制限ロールでの CSV 取り込みは通るはず");
    assert_eq!(uploaded.inserted, 1);
    backend::services::csv::record_import_success(&restricted, user_a, "dividend", &uploaded)
        .await
        .expect("制限ロールでの履歴記録は通るはず");
    let history_noset = db::query_scalar::<i64>(
        "SELECT COUNT(*) FROM csv_import_history WHERE user_id = $1",
        vec![Bind::from(user_a)],
    )
    .fetch_one(&restricted)
    .await
    .expect("履歴の件数取得に失敗");
    assert_eq!(
        history_noset, 0,
        "設定なしの制限ロールには履歴が見えてはいけない"
    );
    let mut tx = restricted
        .begin_with_user(user_a)
        .await
        .expect("利用者付き tx の開始に失敗");
    let history_a = db::query_scalar::<i64>(
        "SELECT COUNT(*) FROM csv_import_history WHERE user_id = $1",
        vec![Bind::from(user_a)],
    )
    .fetch_one(&mut tx)
    .await
    .expect("履歴の件数取得に失敗");
    assert_eq!(history_a, 1, "履歴は本人の行だけが見えるはず");
    tx.rollback().await.expect("rollback に失敗");

    let replaced = asset_balance_svc::bulk_create(
        &restricted,
        user_a,
        &[make_asset_item("9902")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("制限ロールでの残高置換は通るはず");
    assert_eq!(replaced.inserted, 1);
    let mut tx = restricted
        .begin_with_user(user_a)
        .await
        .expect("利用者付き tx の開始に失敗");
    let asset_codes = db::query("SELECT security_code FROM asset_balances", vec![])
        .fetch_all::<(String,), _>(&mut tx)
        .await
        .expect("残高の取得に失敗");
    assert_eq!(
        asset_codes,
        vec![("9902".to_string(),)],
        "置換後は本人の行だけが見えるはず"
    );
    tx.rollback().await.expect("rollback に失敗");

    let token = issue_token(&restricted, user_a)
        .await
        .expect("制限ロールでのトークン発行は通るはず");
    assert!(!token.is_empty(), "発行されたトークンが空ではいけない");
    for (user_id, expected) in [(user_a, 1i64), (user_b, 0i64)] {
        let mut tx = restricted
            .begin_with_user(user_id)
            .await
            .expect("利用者付き tx の開始に失敗");
        let sends = db::query_scalar::<i64>(
            "SELECT COUNT(*) FROM login_notify_tokens WHERE user_id = $1 \
             AND created_at > NOW() - INTERVAL '1 hour'",
            vec![Bind::from(user_id)],
        )
        .fetch_one(&mut tx)
        .await
        .expect("送信回数の取得に失敗");
        assert_eq!(sends, expected, "自分の発行分だけ数えるはず");
        tx.rollback().await.expect("rollback に失敗");
    }
    assert!(
        !set_setting(&restricted, user_a, false)
            .await
            .expect("制限ロールでの設定更新は通るはず"),
        "更新で返った設定値が変わっているはず"
    );
    assert!(
        !get_setting(&pool, user_a)
            .await
            .expect("設定取得は通るはず"),
        "更新後の設定値が読めるはず"
    );
    assert!(
        get_setting(&pool, user_b)
            .await
            .expect("設定取得は通るはず"),
        "別利用者の設定値に影響してはいけない"
    );
    set_setting(&restricted, user_a, true)
        .await
        .expect("設定値の復元に失敗");

    record_admin_view(&restricted, user_a)
        .await
        .expect("制限ロールでの閲覧記録は通るはず");
    let mut tx = restricted
        .begin_with_user(user_a)
        .await
        .expect("利用者付き tx の開始に失敗");
    let views_a = db::query_scalar::<i64>(
        "SELECT COUNT(*) FROM admin_login_views WHERE user_id = $1",
        vec![Bind::from(user_a)],
    )
    .fetch_one(&mut tx)
    .await
    .expect("閲覧記録の件数取得に失敗");
    assert_eq!(views_a, 1, "閲覧記録は本人の行だけが見えるはず");
    tx.rollback().await.expect("rollback に失敗");

    let deleted = bulk::delete_all::<dividend_svc::DividendDomain>(&restricted, user_c)
        .await
        .expect("制限ロールでの全削除は通るはず");
    assert_eq!(deleted, 1, "利用者 C の配当 1 件だけ消えるはず");
    for (user_id, expected) in [(user_a, 2i64), (user_b, 1i64)] {
        let count = db::query_scalar::<i64>(
            "SELECT COUNT(*) FROM dividends WHERE user_id = $1",
            vec![Bind::from(user_id)],
        )
        .fetch_one(&pool)
        .await
        .expect("所有者の件数取得に失敗");
        assert_eq!(count, expected, "別利用者の行が消えてはいけない");
    }

    let mut list_params = default_dividend_search_params();
    list_params.search.include_summary = Some(true);
    list_params.search.include_facets = Some(true);
    let dividends_a = domain_search::<dividend_svc::DividendDomain>(&pool, user_a, list_params)
        .await
        .expect("配当一覧は通るはず");
    assert_eq!(dividends_a.total, 2, "利用者 A の配当が見えるはず");
    assert!(dividends_a.summary.is_some(), "配当集計が返るはず");
    assert!(dividends_a.facets.is_some(), "配当 facets が返るはず");
    let asset_a = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_a,
        default_asset_balance_search_params(),
    )
    .await
    .expect("残高一覧は通るはず");
    assert_eq!(asset_a.total, 1, "利用者 A の残高が見えるはず");
    let history_a = list_login_history(&pool, user_a, 50, 0)
        .await
        .expect("ログイン履歴は通るはず");
    assert_eq!(history_a.len(), 1, "利用者 A の記録が見えるはず");

    assert_no_user_setting(&restricted).await;

    let preview = dividend_svc::DividendDomain::preview_csv(make_dividend_csv().as_bytes())
        .expect("未認証のプレビューは通るはず");
    assert_eq!(preview.valid_rows, 1);
    assert!(
        auth_svc::select_user_id_by_session(&pool, SessionToken::new())
            .await
            .expect("未知セッションの参照に失敗")
            .is_none(),
        "未知のセッションは利用者を返してはいけない"
    );
    assert!(
        peek_token(&pool, "存在しない平文トークン")
            .await
            .expect("未知トークンの参照に失敗")
            .is_none()
    );
    assert!(
        consume_token(&pool, "存在しない平文トークン")
            .await
            .expect("未知トークンの消費に失敗")
            .is_none()
    );
    let owner_setting = db::query_scalar::<Option<String>>(
        "SELECT current_setting('app.current_user_id', true)",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("設定値の取得に失敗");
    assert!(
        owner_setting.as_deref().is_none_or(str::is_empty),
        "未認証経路では利用者の値が残ってはいけない: {owner_setting:?}"
    );

    for (table, expected) in [("dividends", 3i64), ("login_events", 2i64)] {
        let count = db::query_scalar::<i64>(format!("SELECT COUNT(*) FROM {table}"), vec![])
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, expected, "所有者の挙動は変わらないはず: {table}");
    }
}

/// RLS 第四段階: `sessions`・`users` は所有者の挙動を変えず、
/// 権限を絞ったロールでは利用者ごとに分離される
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn sessions_users_rls_isolates_restricted_role() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");

    for table in ["sessions", "users"] {
        let rls = db::query_scalar::<bool>(
            "SELECT relrowsecurity FROM pg_class WHERE relname = $1",
            vec![Bind::from(table)],
        )
        .fetch_one(&pool)
        .await
        .expect("RLS 有効の取得に失敗");
        assert!(rls, "{table} に RLS が有効のはず");
        let forced = db::query_scalar::<bool>(
            "SELECT relforcerowsecurity FROM pg_class WHERE relname = $1",
            vec![Bind::from(table)],
        )
        .fetch_one(&pool)
        .await
        .expect("FORCE 状態の取得に失敗");
        assert!(
            !forced,
            "{table} に FORCE は付けない(所有者の挙動を変えない)"
        );
    }

    let info_a = GoogleUserInfo {
        sub: "stage4-google-a".to_string(),
        email: "stage4-a@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    let info_b = GoogleUserInfo {
        sub: "stage4-google-b".to_string(),
        email: "stage4-b@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    let token_a = auth_svc::upsert_user_and_rotate_session(&pool, &info_a)
        .await
        .expect("利用者 A のログインは通るはず");
    let token_b = auth_svc::upsert_user_and_rotate_session(&pool, &info_b)
        .await
        .expect("利用者 B のログインは通るはず");
    let user_a = auth_svc::select_user_id_by_session(&pool, token_a)
        .await
        .expect("セッション照会は通るはず")
        .expect("利用者 A のセッションがあるはず");
    let user_b = auth_svc::select_user_id_by_session(&pool, token_b)
        .await
        .expect("セッション照会は通るはず")
        .expect("利用者 B のセッションがあるはず");
    assert!(
        auth_svc::select_user_by_session(&pool, token_a)
            .await
            .expect("利用者照会は通るはず")
            .is_some(),
        "利用者 A の行が読めるはず"
    );
    auth_svc::delete_session(&pool, token_b)
        .await
        .expect("セッション削除は通るはず");
    assert!(
        auth_svc::select_user_id_by_session(&pool, token_b)
            .await
            .expect("削除後の照会は通るはず")
            .is_none(),
        "削除したセッションは失効するはず"
    );
    let token_b = auth_svc::upsert_user_and_rotate_session(&pool, &info_b)
        .await
        .expect("利用者 B の再ログインは通るはず");
    assert!(
        auth_svc::select_user_id_by_session(&pool, token_b)
            .await
            .expect("再ログイン後の照会は通るはず")
            .is_some(),
        "再発行したセッションが有効のはず"
    );

    let info_c = GoogleUserInfo {
        sub: "stage4-google-c".to_string(),
        email: "stage4-c@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    let token_c = auth_svc::upsert_user_and_rotate_session(&pool, &info_c)
        .await
        .expect("利用者 C のログインは通るはず");
    let user_c = auth_svc::select_user_id_by_session(&pool, token_c)
        .await
        .expect("セッション照会は通るはず")
        .expect("利用者 C のセッションがあるはず");
    auth_svc::delete_account(&pool, user_c)
        .await
        .expect("利用者削除は通るはず");
    assert!(
        auth_svc::select_user_by_session(&pool, token_c)
            .await
            .expect("削除後の照会は通るはず")
            .is_none(),
        "削除した利用者のセッションは失効するはず"
    );
    let remaining = db::query_scalar::<i64>(
        "SELECT COUNT(*) FROM users WHERE id = $1",
        vec![Bind::from(user_c)],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(remaining, 0, "削除した利用者の行はないはず");

    let user_count = db::query_scalar::<i64>("SELECT COUNT(*) FROM users", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user_count, 2, "所有者は両利用者の行を読めるはず");
    let session_count = db::query_scalar::<i64>("SELECT COUNT(*) FROM sessions", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(session_count, 2, "所有者は両利用者のセッションを読めるはず");

    pool.batch_execute("CREATE ROLE rls_app WITH LOGIN PASSWORD 'rls-test-password'")
        .await
        .expect("制限ロールの作成に失敗");
    pool.batch_execute("GRANT CONNECT ON DATABASE postgres TO rls_app")
        .await
        .expect("CONNECT 付与に失敗");
    pool.batch_execute("GRANT USAGE ON SCHEMA public TO rls_app")
        .await
        .expect("USAGE 付与に失敗");
    for table in ["sessions", "users"] {
        pool.batch_execute(&format!(
            "GRANT SELECT, INSERT, UPDATE, DELETE ON {table} TO rls_app"
        ))
        .await
        .expect("表権限の付与に失敗");
    }
    let restricted_url = format!("postgres://rls_app:rls-test-password@127.0.0.1:{port}/postgres");
    let restricted = connect_with_retry(&restricted_url).await;

    for table in ["sessions", "users"] {
        let count = db::query_scalar::<i64>(format!("SELECT COUNT(*) FROM {table}"), vec![])
            .fetch_one(&restricted)
            .await
            .unwrap();
        assert_eq!(
            count, 0,
            "利用者 ID 未設定の制限ロールは何も見えないはず: {table}"
        );
    }

    let mut tx = restricted.begin().await.expect("tx 開始に失敗");
    db::query(
        "SELECT set_config('app.current_user_id', $1, true)",
        vec![Bind::from(user_a.to_string())],
    )
    .execute(&mut tx)
    .await
    .expect("利用者 ID の設定に失敗");
    let owners = db::query("SELECT user_id::text FROM sessions", vec![])
        .fetch_all::<(String,), _>(&mut tx)
        .await
        .expect("セッションの取得に失敗");
    assert_eq!(
        owners,
        vec![(user_a.to_string(),)],
        "利用者 A のセッションだけが見えるはず"
    );
    let ids = db::query("SELECT id::text FROM users", vec![])
        .fetch_all::<(String,), _>(&mut tx)
        .await
        .expect("利用者の取得に失敗");
    assert_eq!(
        ids,
        vec![(user_a.to_string(),)],
        "利用者 A の行だけが見えるはず"
    );
    db::query(
        "INSERT INTO sessions (user_id, token_hash) VALUES ($1, $2)",
        vec![Bind::from(user_a), Bind::Bytes(vec![51u8; 32])],
    )
    .execute(&mut tx)
    .await
    .expect("自分のセッションの書き込みは通るはず");
    let updated = db::query(
        "UPDATE users SET email = $2 WHERE id = $1",
        vec![
            Bind::from(user_a),
            Bind::from("stage4-a-updated@example.com"),
        ],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の更新は通るはず");
    assert_eq!(updated, 1, "自分の行が1件更新されるはず");
    let err = db::query(
        "INSERT INTO sessions (user_id, token_hash) VALUES ($1, $2)",
        vec![Bind::from(user_b), Bind::Bytes(vec![52u8; 32])],
    )
    .execute(&mut tx)
    .await
    .expect_err("他人名義のセッション書き込みは失敗するはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "行共有違反のはず: {err:?}"
    );
    tx.rollback().await.expect("rollback に失敗");

    let mut tx = restricted.begin().await.expect("tx 開始に失敗");
    db::query(
        "SELECT set_config('app.current_user_id', $1, true)",
        vec![Bind::from(user_a.to_string())],
    )
    .execute(&mut tx)
    .await
    .expect("利用者 ID の設定に失敗");
    let other_id = uuid::Uuid::new_v4();
    let err = db::query(
        "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
        vec![
            Bind::from(other_id),
            Bind::from(format!("stage4-other-{other_id}")),
            Bind::from(format!("stage4-other-{other_id}@example.com")),
        ],
    )
    .execute(&mut tx)
    .await
    .expect_err("他人名義の利用者書き込みは失敗するはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "行共有違反のはず(users): {err:?}"
    );
    tx.rollback().await.expect("rollback に失敗");
}

/// RLS 第五段階 5a: migration 作成の制限ロール `app_restricted` は NOLOGIN で特権なし、
/// 本人表への読み書きと共有表・ビューへの読み取りだけを持ち、所有者は所属により切り替えできる
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn app_restricted_role_has_least_privilege() {
    async fn restricted_tx_with_user(pool: &Db, user: UserId) -> backend::db::Tx {
        let mut tx = pool.begin().await.expect("tx 開始に失敗");
        db::query("SELECT set_config('role', 'app_restricted', true)", vec![])
            .execute(&mut tx)
            .await
            .expect("ロールの切り替えに失敗");
        db::query(
            "SELECT set_config('app.current_user_id', $1, true)",
            vec![Bind::from(user.to_string())],
        )
        .execute(&mut tx)
        .await
        .expect("利用者 ID の設定に失敗");
        tx
    }
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");

    let can_login = db::query_scalar::<bool>(
        "SELECT rolcanlogin FROM pg_roles WHERE rolname = 'app_restricted'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("ロールの取得に失敗");
    assert!(!can_login, "app_restricted は NOLOGIN のはず");
    let bypass = db::query_scalar::<bool>(
        "SELECT rolbypassrls FROM pg_roles WHERE rolname = 'app_restricted'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("ロールの取得に失敗");
    assert!(!bypass, "app_restricted に BYPASSRLS はないはず");

    let is_member = db::query_scalar::<bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_auth_members m JOIN pg_roles r ON r.oid = m.roleid JOIN pg_roles u ON u.oid = m.member WHERE r.rolname = 'app_restricted' AND u.rolname = CURRENT_USER)",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("所属の取得に失敗");
    assert!(is_member, "所有者は app_restricted の所属のはず");

    let schema_usage = db::query_scalar::<bool>(
        "SELECT has_schema_privilege('app_restricted', 'public', 'USAGE')",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("スキーマ権限の取得に失敗");
    assert!(schema_usage, "app_restricted は public の USAGE を持つはず");

    for (signature, expected) in [
        ("public.login_notify_peek_v2(bytea)", true),
        ("public.login_notify_consume_v2(bytea)", true),
        (
            "public.admin_login_search_data_v2(text[],text[],text[],text,text,bigint,bigint)",
            false,
        ),
        (
            "public.admin_login_search_count_v2(text[],text[],text[])",
            false,
        ),
        (
            "public.scheduled_stock_price_targets_v2(date,bigint)",
            false,
        ),
        ("public.scheduled_fund_price_targets_v2(date,bigint)", false),
        ("public.prune_login_events_older_than_90_days_v2()", false),
    ] {
        let executable = db::query_scalar::<bool>(
            "SELECT has_function_privilege('app_restricted', $1, 'EXECUTE')",
            vec![Bind::from(signature)],
        )
        .fetch_one(&pool)
        .await
        .expect("関数権限の取得に失敗");
        assert_eq!(executable, expected, "{signature}");
    }

    let grants = db::query(
        "SELECT table_name, privilege_type FROM information_schema.role_table_grants WHERE grantee = 'app_restricted' AND table_schema = 'public'",
        vec![],
    )
    .fetch_all::<(String, String), _>(&pool)
    .await
    .expect("表権限の取得に失敗");
    let mut by_table: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for (table, privilege) in grants {
        by_table.entry(table).or_default().push(privilege);
    }
    for privileges in by_table.values_mut() {
        privileges.sort();
    }
    let full_tables = [
        "sessions",
        "dividends",
        "domestic_stocks",
        "mutualfunds",
        "asset_balances",
        "fund_balances",
        "csv_import_history",
    ];
    for table in full_tables {
        let expected: Vec<String> = ["DELETE", "INSERT", "SELECT", "UPDATE"]
            .iter()
            .map(|privilege| (*privilege).to_string())
            .collect();
        assert_eq!(
            by_table.get(table),
            Some(&expected),
            "{table} は本人表の読み書きだけのはず"
        );
    }
    for (table, expected) in [
        ("users", vec!["DELETE", "INSERT", "SELECT"]),
        ("login_events", vec!["DELETE", "INSERT", "SELECT"]),
        ("login_notify_tokens", vec!["INSERT", "SELECT"]),
        ("admin_login_views", vec!["INSERT", "SELECT"]),
    ] {
        let expected: Vec<String> = expected.into_iter().map(str::to_string).collect();
        assert_eq!(
            by_table.get(table),
            Some(&expected),
            "{table} は絞った操作だけのはず"
        );
    }
    for (column, expected) in [
        ("login_notify_enabled", true),
        ("updated_at", true),
        ("email", false),
        ("google_id", false),
        ("name", false),
        ("picture_url", false),
    ] {
        let updatable = db::query_scalar::<bool>(
            "SELECT has_column_privilege('app_restricted', 'public.users', $1, 'UPDATE')",
            vec![Bind::from(column)],
        )
        .fetch_one(&pool)
        .await
        .expect("列権限の取得に失敗");
        assert_eq!(
            updatable, expected,
            "users.{column} の UPDATE は {expected} のはず"
        );
    }
    let shared = [
        "stock",
        "security_prices",
        "dividend_per_share_cache",
        "market_data_provider_rate_control",
        "fund_codes",
        "fund_prices",
        "asset_balances_view",
        "fund_balances_view",
    ];
    for table in shared {
        let expected: Vec<String> = ["SELECT"]
            .iter()
            .map(|privilege| (*privilege).to_string())
            .collect();
        assert_eq!(
            by_table.get(table),
            Some(&expected),
            "{table} は読み取りだけのはず"
        );
    }
    for function in [
        "public.fund_name_normalize(text)",
        "public.fund_expected_nav_date(timestamptz)",
    ] {
        let executable = db::query_scalar::<bool>(
            format!("SELECT has_function_privilege('app_restricted', '{function}', 'EXECUTE')"),
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("関数権限の取得に失敗");
        assert!(executable, "{function} は実行できるはず");
    }

    let user_a = create_test_user(&pool).await;
    let user_b = create_test_user(&pool).await;
    dividend_svc::bulk_create(
        &pool,
        user_a,
        &[make_dividend_item("9931")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当の投入に失敗");
    dividend_svc::bulk_create(
        &pool,
        user_b,
        &[make_dividend_item("9932")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("配当の投入に失敗");
    db::query(
        "INSERT INTO stock (date, code, name, market_category) VALUES ($1, $2, $3, $4)",
        vec![
            Bind::from(
                Date::from_calendar_date(2025, time::Month::try_from(3).unwrap(), 24).unwrap(),
            ),
            Bind::from("9931"),
            Bind::from("テスト株式会社"),
            Bind::from("プライム"),
        ],
    )
    .execute(&pool)
    .await
    .expect("銘柄の投入に失敗");
    for user_id in [user_a, user_b] {
        db::query(
            "INSERT INTO csv_import_history (user_id, kind) VALUES ($1, 'dividend')",
            vec![Bind::from(user_id)],
        )
        .execute(&pool)
        .await
        .expect("履歴の投入に失敗");
    }

    let mut tx = pool.begin().await.expect("tx 開始に失敗");
    db::query("SELECT set_config('role', 'app_restricted', true)", vec![])
        .execute(&mut tx)
        .await
        .expect("ロールの切り替えに失敗");
    let current = db::query_scalar::<String>("SELECT current_user", vec![])
        .fetch_one(&mut tx)
        .await
        .expect("利用者名の取得に失敗");
    assert_eq!(
        current, "app_restricted",
        "切り替え後の利用者は制限ロールのはず"
    );

    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM dividends", vec![])
        .fetch_one(&mut tx)
        .await
        .unwrap();
    assert_eq!(count, 0, "利用者 ID 未設定の制限ロールは何も見えないはず");
    tx.rollback().await.expect("rollback に失敗");

    let mut tx = restricted_tx_with_user(&pool, user_a).await;
    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM dividends", vec![])
        .fetch_one(&mut tx)
        .await
        .unwrap();
    assert_eq!(count, 1, "利用者 A の行だけが見えるはず");
    let kinds = db::query("SELECT kind FROM csv_import_history", vec![])
        .fetch_all::<(String,), _>(&mut tx)
        .await
        .expect("履歴の取得に失敗");
    assert_eq!(
        kinds,
        vec![("dividend".to_string(),)],
        "利用者 A の履歴だけが見えるはず"
    );
    let stock_count = db::query_scalar::<i64>("SELECT COUNT(*) FROM stock", vec![])
        .fetch_one(&mut tx)
        .await
        .unwrap();
    assert_eq!(stock_count, 1, "共有表は読めるはず");
    tx.rollback().await.expect("rollback に失敗");

    let mut tx = restricted_tx_with_user(&pool, user_a).await;
    db::query(
        "INSERT INTO csv_import_history (user_id, kind) VALUES ($1, 'assetbalance')",
        vec![Bind::from(user_a)],
    )
    .execute(&mut tx)
    .await
    .expect("自分の行の書き込みは通るはず");
    db::query(
        "INSERT INTO sessions (user_id, token_hash) VALUES ($1, $2)",
        vec![Bind::from(user_a), Bind::Bytes(vec![61u8; 32])],
    )
    .execute(&mut tx)
    .await
    .expect("自分のセッションの書き込みは通るはず");
    tx.rollback().await.expect("rollback に失敗");

    let mut tx = restricted_tx_with_user(&pool, user_a).await;
    let err = db::query(
        "INSERT INTO csv_import_history (user_id, kind) VALUES ($1, 'assetbalance')",
        vec![Bind::from(user_b)],
    )
    .execute(&mut tx)
    .await
    .expect_err("他人名義の書き込みは失敗するはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "行共有違反のはず: {err:?}"
    );
    tx.rollback().await.expect("rollback に失敗");

    let mut tx = restricted_tx_with_user(&pool, user_a).await;
    let err = db::query(
        "INSERT INTO sessions (user_id, token_hash) VALUES ($1, $2)",
        vec![Bind::from(user_b), Bind::Bytes(vec![62u8; 32])],
    )
    .execute(&mut tx)
    .await
    .expect_err("他人名義のセッション書き込みは失敗するはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "行共有違反のはず(sessions): {err:?}"
    );
    tx.rollback().await.expect("rollback に失敗");

    let mut tx = restricted_tx_with_user(&pool, user_a).await;
    let err = db::query(
        "INSERT INTO stock (date, code, name, market_category) VALUES ($1, $2, $3, $4)",
        vec![
            Bind::from(
                Date::from_calendar_date(2025, time::Month::try_from(3).unwrap(), 25).unwrap(),
            ),
            Bind::from("9932"),
            Bind::from("テスト株式会社"),
            Bind::from("プライム"),
        ],
    )
    .execute(&mut tx)
    .await
    .expect_err("共有表の書き込みは失敗するはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "権限不足のはず(stock): {err:?}"
    );
    tx.rollback().await.expect("rollback に失敗");

    let mut tx = restricted_tx_with_user(&pool, user_a).await;
    let updated = db::query(
        "UPDATE users SET login_notify_enabled = NOT login_notify_enabled, updated_at = NOW() WHERE id = $1",
        vec![Bind::from(user_a)],
    )
    .execute(&mut tx)
    .await
    .expect("通知設定の列の更新は通るはず");
    assert_eq!(updated, 1, "自分の通知設定 1 件が更新されるはず");
    tx.rollback().await.expect("rollback に失敗");

    for sql in [
        "UPDATE users SET email = 'narrowed@example.com' WHERE id = '00000000-0000-0000-0000-000000000000'",
        "UPDATE users SET google_id = 'narrowed' WHERE id = '00000000-0000-0000-0000-000000000000'",
        "UPDATE login_events SET result = 'success' WHERE user_id = '00000000-0000-0000-0000-000000000000'",
        "UPDATE login_notify_tokens SET used_at = NOW() WHERE user_id = '00000000-0000-0000-0000-000000000000'",
        "UPDATE admin_login_views SET occurred_at = NOW() WHERE user_id = '00000000-0000-0000-0000-000000000000'",
        "DELETE FROM login_notify_tokens WHERE user_id = '00000000-0000-0000-0000-000000000000'",
        "DELETE FROM admin_login_views WHERE user_id = '00000000-0000-0000-0000-000000000000'",
    ] {
        let mut tx = restricted_tx_with_user(&pool, user_a).await;
        let err = db::query(sql, vec![])
            .execute(&mut tx)
            .await
            .expect_err("絞った権限の外の操作は失敗するはず");
        assert!(
            err.sql_state()
                .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
            "権限不足のはず({sql}): {err:?}"
        );
        tx.rollback().await.expect("rollback に失敗");
    }

    let current = db::query_scalar::<String>("SELECT current_user", vec![])
        .fetch_one(&pool)
        .await
        .expect("利用者名の取得に失敗");
    assert_eq!(current, "postgres", "終了後は所有者に戻るはず");
    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM dividends", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2, "所有者は両利用者の行を読めるはず");
}

/// RLS 第五段階 5b: app_enter(所有者専用)と照会用の DEFINER 関数は引数・戻り・権限が最小限で、
/// DEFINER 関数は app_restricted から呼べる・public からは呼べない・未知のハッシュでは他人の行を返さない。
/// app_enter のあとは本人の行だけが見え、COMMIT で所有者に戻る。
/// 回転は設定と違う利用者 ID では失敗する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stage5b_definer_functions_and_app_enter() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");
    assert_eq!(migration_versions(&pool).await, local_migration_versions());

    let info_a = GoogleUserInfo {
        sub: "stage5b-google-a".to_string(),
        email: "stage5b-a@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    let info_b = GoogleUserInfo {
        sub: "stage5b-google-b".to_string(),
        email: "stage5b-b@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    let token_a = auth_svc::upsert_user_and_rotate_session(&pool, &info_a)
        .await
        .expect("利用者 A のログインは通るはず");
    let token_b = auth_svc::upsert_user_and_rotate_session(&pool, &info_b)
        .await
        .expect("利用者 B のログインは通るはず");
    let user_a = auth_svc::select_user_id_by_session(&pool, token_a)
        .await
        .expect("セッション照会は通るはず")
        .expect("利用者 A のセッションがあるはず");
    let user_b = auth_svc::select_user_id_by_session(&pool, token_b)
        .await
        .expect("セッション照会は通るはず")
        .expect("利用者 B のセッションがあるはず");
    let hash_a = token_a.hash();
    let hash_b = token_b.hash();
    let plain_a = backend::services::login_notify::issue_token(&pool, user_a)
        .await
        .expect("通知トークンの発行は通るはず");
    let plain_b = backend::services::login_notify::issue_token(&pool, user_b)
        .await
        .expect("通知トークンの発行は通るはず");
    let hash_plain_a = backend::services::login_notify::token_hash(&plain_a);
    let hash_plain_b = backend::services::login_notify::token_hash(&plain_b);

    for signature in [
        "public.auth_lookup_user_id_by_session_with_idle_timeout(bytea,timestamptz)",
        "public.auth_lookup_user_id_by_session_v2(bytea)",
        "public.auth_delete_session_by_token(bytea)",
        "public.auth_rotate_session(uuid, bytea, bytea)",
        "public.auth_upsert_user(text, text, text, text)",
    ] {
        let executable = db::query_scalar::<bool>(
            format!("SELECT has_function_privilege('app_restricted', '{signature}', 'EXECUTE')"),
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("関数権限の取得に失敗");
        assert!(executable, "{signature} は app_restricted が実行できるはず");
        let public = db::query_scalar::<bool>(
            format!("SELECT has_function_privilege('public', '{signature}', 'EXECUTE')"),
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("関数権限の取得に失敗");
        assert!(!public, "{signature} は public から実行できないはず");
        let is_definer = db::query_scalar::<bool>(
            "SELECT prosecdef FROM pg_proc WHERE oid = $1::regprocedure",
            vec![Bind::from(signature)],
        )
        .fetch_one(&pool)
        .await
        .expect("関数定義の取得に失敗");
        assert!(is_definer, "{signature} は DEFINER のはず");
    }
    // 旧 login_notify_*_by_token は削除済みで存在しない
    for signature in [
        "public.login_notify_peek_by_token(bytea)",
        "public.login_notify_consume_by_token(bytea)",
    ] {
        let exists = db::query_scalar::<bool>(
            "SELECT to_regprocedure($1) IS NOT NULL",
            vec![Bind::from(signature)],
        )
        .fetch_one(&pool)
        .await
        .expect("関数定義の取得に失敗");
        assert!(!exists, "{signature} は存在しないはず");
    }
    for signature in [
        "public.auth_session_user_by_token(bytea)",
        "public.auth_lookup_user_id_by_session(bytea)",
    ] {
        let exists = db::query_scalar::<bool>(
            "SELECT to_regprocedure($1) IS NOT NULL",
            vec![Bind::from(signature)],
        )
        .fetch_one(&pool)
        .await
        .expect("関数定義の取得に失敗");
        assert!(!exists, "{signature} は存在しないはず");
    }

    let owner_enter = db::query_scalar::<bool>(
        "SELECT has_function_privilege(CURRENT_USER, 'public.app_enter(uuid)', 'EXECUTE')",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("関数権限の取得に失敗");
    assert!(owner_enter, "app_enter は所有者が実行できるはず");
    for role in ["app_restricted", "public"] {
        let executable = db::query_scalar::<bool>(
            format!("SELECT has_function_privilege('{role}', 'public.app_enter(uuid)', 'EXECUTE')"),
            vec![],
        )
        .fetch_one(&pool)
        .await
        .expect("関数権限の取得に失敗");
        assert!(!executable, "app_enter は {role} から実行できないはず");
    }
    let is_definer = db::query_scalar::<bool>(
        "SELECT prosecdef FROM pg_proc WHERE oid = 'public.app_enter(uuid)'::regprocedure",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("関数定義の取得に失敗");
    assert!(!is_definer, "app_enter は INVOKER のはず");

    for (signature, volatile) in [
        ("public.app_enter(uuid)", "v"),
        (
            "public.auth_lookup_user_id_by_session_with_idle_timeout(bytea,timestamp with time zone)",
            "s",
        ),
        ("public.auth_lookup_user_id_by_session_v2(bytea)", "s"),
        ("public.auth_session_user_by_token_v2(bytea)", "v"),
        ("public.auth_delete_session_by_token(bytea)", "v"),
        ("public.auth_rotate_session(uuid, bytea, bytea)", "v"),
        ("public.auth_upsert_user(text, text, text, text)", "v"),
    ] {
        let actual = db::query_scalar::<String>(
            "SELECT provolatile::text FROM pg_proc WHERE oid = $1::regprocedure",
            vec![Bind::from(signature)],
        )
        .fetch_one(&pool)
        .await
        .expect("揮発性の取得に失敗");
        assert_eq!(actual, volatile, "{signature} の揮発性のはず");
        let config = db::query(
            "SELECT unnest(proconfig) FROM pg_proc WHERE oid = $1::regprocedure",
            vec![Bind::from(signature)],
        )
        .fetch_all::<(String,), _>(&pool)
        .await
        .expect("設定の取得に失敗");
        let expected_search_path = "search_path=public, pg_temp";
        assert!(
            config.contains(&(expected_search_path.to_string(),)),
            "{signature} の search_path を変更しない"
        );
    }

    pool.batch_execute("CREATE ROLE fn_nogrant WITH LOGIN PASSWORD 'fn-nogrant-password'")
        .await
        .expect("権限なしロールの作成に失敗");
    pool.batch_execute("GRANT CONNECT ON DATABASE postgres TO fn_nogrant")
        .await
        .expect("CONNECT 付与に失敗");
    pool.batch_execute("GRANT USAGE ON SCHEMA public TO fn_nogrant")
        .await
        .expect("USAGE 付与に失敗");
    let denied_url = format!("postgres://fn_nogrant:fn-nogrant-password@127.0.0.1:{port}/postgres");
    let denied = connect_with_retry(&denied_url).await;
    let err = db::query(
        "SELECT auth_lookup_user_id_by_session_with_idle_timeout($1, $2)",
        vec![
            Bind::Bytes(hash_a.clone()),
            Bind::from(time::OffsetDateTime::now_utc() - time::Duration::minutes(30)),
        ],
    )
    .execute(&denied)
    .await
    .expect_err("権限なしロールは照会関数を呼べないはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "権限不足のはず: {err:?}"
    );
    let err = db::query("SELECT app_enter($1)", vec![Bind::from(user_a)])
        .execute(&denied)
        .await
        .expect_err("権限なしロールは app_enter を呼べないはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::INSUFFICIENT_PRIVILEGE),
        "権限不足のはず(app_enter): {err:?}"
    );

    let mut tx = pool.begin().await.expect("tx 開始に失敗");
    db::query("SELECT set_config('role', 'app_restricted', true)", vec![])
        .execute(&mut tx)
        .await
        .expect("ロールの切り替えに失敗");
    let found = db::query_scalar::<Option<UserId>>(
        "SELECT auth_lookup_user_id_by_session_with_idle_timeout($1, $2)",
        vec![
            Bind::Bytes(hash_a.clone()),
            Bind::from(time::OffsetDateTime::now_utc() - time::Duration::minutes(30)),
        ],
    )
    .fetch_one(&mut tx)
    .await
    .expect("照会関数の呼び出しに失敗");
    assert_eq!(
        found,
        Some(user_a),
        "所有するハッシュでは本人の利用者 ID が返るはず"
    );
    let unknown = db::query_scalar::<Option<UserId>>(
        "SELECT auth_lookup_user_id_by_session_with_idle_timeout($1, $2)",
        vec![
            Bind::Bytes(vec![91u8; 32]),
            Bind::from(time::OffsetDateTime::now_utc() - time::Duration::minutes(30)),
        ],
    )
    .fetch_one(&mut tx)
    .await
    .expect("照会関数の呼び出しに失敗");
    assert_eq!(unknown, None, "未知のハッシュでは他人の行を返さないはず");
    let peeked = db::query(
        "SELECT * FROM login_notify_peek_v2($1)",
        vec![Bind::Bytes(hash_plain_b.clone())],
    )
    .fetch_optional::<(UserId, OffsetDateTime), _>(&mut tx)
    .await
    .expect("参照関数の呼び出しに失敗");
    assert_eq!(
        peeked.map(|(user_id, _)| user_id),
        Some(user_b),
        "所有するトークンでは本人の利用者 ID が返るはず"
    );
    let missing = db::query(
        "SELECT * FROM login_notify_peek_v2($1)",
        vec![Bind::Bytes(vec![92u8; 32])],
    )
    .fetch_optional::<(UserId, OffsetDateTime), _>(&mut tx)
    .await
    .expect("参照関数の呼び出しに失敗");
    assert!(missing.is_none(), "未知のハッシュでは行を返さないはず");
    let consumed = db::query_scalar::<Option<UserId>>(
        "SELECT login_notify_consume_v2($1)",
        vec![Bind::Bytes(hash_plain_a.clone())],
    )
    .fetch_one(&mut tx)
    .await
    .expect("消費関数の呼び出しに失敗");
    assert_eq!(consumed, Some(user_a), "所有するトークンを消費できるはず");
    let consumed_again = db::query_scalar::<Option<UserId>>(
        "SELECT login_notify_consume_v2($1)",
        vec![Bind::Bytes(hash_plain_a.clone())],
    )
    .fetch_one(&mut tx)
    .await
    .expect("消費関数の呼び出しに失敗");
    assert_eq!(consumed_again, None, "消費済みトークンは NULL のはず");
    tx.rollback().await.expect("rollback に失敗");

    let mut tx = pool.begin().await.expect("tx 開始に失敗");
    db::query("SELECT app_enter($1)", vec![Bind::from(user_a)])
        .execute(&mut tx)
        .await
        .expect("app_enter に失敗");
    let current = db::query_scalar::<String>("SELECT current_user", vec![])
        .fetch_one(&mut tx)
        .await
        .expect("利用者名の取得に失敗");
    assert_eq!(
        current, "app_restricted",
        "切り替え後の利用者は制限ロールのはず"
    );
    let setting =
        db::query_scalar::<String>("SELECT current_setting('app.current_user_id')", vec![])
            .fetch_one(&mut tx)
            .await
            .expect("利用者 ID の取得に失敗");
    assert_eq!(setting, user_a.to_string(), "利用者 ID が設定されるはず");
    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM sessions", vec![])
        .fetch_one(&mut tx)
        .await
        .unwrap();
    assert_eq!(count, 1, "app_enter のあとは本人の行だけが見えるはず");
    tx.commit().await.expect("commit に失敗");
    let current = db::query_scalar::<String>("SELECT current_user", vec![])
        .fetch_one(&pool)
        .await
        .expect("利用者名の取得に失敗");
    assert_eq!(current, "postgres", "COMMIT で所有者に戻るはず");
    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM sessions", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2, "所有者は両利用者の行を読めるはず");

    let new_hash = vec![77u8; 32];
    let mut tx = pool.begin().await.expect("tx 開始に失敗");
    db::query("SELECT set_config('role', 'app_restricted', true)", vec![])
        .execute(&mut tx)
        .await
        .expect("ロールの切り替えに失敗");
    db::query(
        "SELECT set_config('app.current_user_id', $1, true)",
        vec![Bind::from(user_a.to_string())],
    )
    .execute(&mut tx)
    .await
    .expect("利用者 ID の設定に失敗");
    let err = db::query(
        "SELECT auth_rotate_session($1, $2, $3)",
        vec![
            Bind::from(user_b),
            Bind::Bytes(vec![79u8; 32]),
            Bind::Bytes(vec![80u8; 32]),
        ],
    )
    .execute(&mut tx)
    .await
    .expect_err("設定と違う利用者 ID の回転は失敗するはず");
    assert!(
        err.sql_state()
            .is_some_and(|s| s == db::SqlState::RAISE_EXCEPTION),
        "利用者不一致のはず: {err:?}"
    );
    tx.rollback().await.expect("rollback に失敗");

    let mut tx = pool.begin().await.expect("tx 開始に失敗");
    db::query("SELECT set_config('role', 'app_restricted', true)", vec![])
        .execute(&mut tx)
        .await
        .expect("ロールの切り替えに失敗");
    db::query(
        "SELECT set_config('app.current_user_id', $1, true)",
        vec![Bind::from(user_a.to_string())],
    )
    .execute(&mut tx)
    .await
    .expect("利用者 ID の設定に失敗");
    db::query(
        "SELECT auth_rotate_session($1, $2, $3)",
        vec![
            Bind::from(user_a),
            Bind::Bytes(new_hash.clone()),
            Bind::Bytes(vec![78u8; 32]),
        ],
    )
    .execute(&mut tx)
    .await
    .expect("回転関数の呼び出しに失敗");
    tx.commit().await.expect("commit に失敗");
    let stored = db::query(
        "SELECT token_hash FROM sessions WHERE user_id = $1",
        vec![Bind::from(user_a)],
    )
    .fetch_all::<(Vec<u8>,), _>(&pool)
    .await
    .expect("セッションの取得に失敗");
    assert_eq!(
        stored,
        vec![(new_hash.clone(),)],
        "利用者 A の行は新しい1件だけのはず"
    );
    assert!(
        auth_svc::select_user_id_by_session(&pool, token_b)
            .await
            .expect("セッション照会は通るはず")
            .is_some(),
        "利用者 B の行は残るはず"
    );

    let mut tx = pool.begin().await.expect("tx 開始に失敗");
    db::query("SELECT set_config('role', 'app_restricted', true)", vec![])
        .execute(&mut tx)
        .await
        .expect("ロールの切り替えに失敗");
    db::query(
        "SELECT auth_delete_session_by_token($1)",
        vec![Bind::Bytes(hash_b.clone())],
    )
    .execute(&mut tx)
    .await
    .expect("消去関数の呼び出しに失敗");
    tx.commit().await.expect("commit に失敗");
    assert!(
        auth_svc::select_user_id_by_session(&pool, token_b)
            .await
            .expect("セッション照会は通るはず")
            .is_none(),
        "削除したセッションは失効するはず"
    );

    let mut tx = pool.begin().await.expect("tx 開始に失敗");
    db::query("SELECT set_config('role', 'app_restricted', true)", vec![])
        .execute(&mut tx)
        .await
        .expect("ロールの切り替えに失敗");
    let new_id = db::query_scalar::<UserId>(
        "SELECT auth_upsert_user($1, $2, $3, $4)",
        vec![
            Bind::from("stage5b-google-c"),
            Bind::from("stage5b-c@example.com"),
            Bind::from(Some("stage5b C")),
            Bind::from(None::<&str>),
        ],
    )
    .fetch_one(&mut tx)
    .await
    .expect("upsert 関数の呼び出しに失敗");
    tx.commit().await.expect("commit に失敗");
    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM users", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 3, "新しい利用者が1件増えるはず");
    let same_id = db::query_scalar::<UserId>(
        "SELECT auth_upsert_user($1, $2, $3, $4)",
        vec![
            Bind::from("stage5b-google-c"),
            Bind::from("stage5b-c2@example.com"),
            Bind::from(Some("stage5b C")),
            Bind::from(None::<&str>),
        ],
    )
    .fetch_one(&pool)
    .await
    .expect("upsert 関数の呼び出しに失敗");
    assert_eq!(
        same_id, new_id,
        "同じ google_id では同じ利用者 ID が返るはず"
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn rls_5c_writes_via_app_enter() {
    async fn count_where(pool: &Db, table: &str, user: UserId) -> i64 {
        let sql = match table {
            "dividends" => "SELECT COUNT(*) FROM dividends WHERE user_id = $1",
            "csv_import_history" => "SELECT COUNT(*) FROM csv_import_history WHERE user_id = $1",
            "asset_balances" => "SELECT COUNT(*) FROM asset_balances WHERE user_id = $1",
            "login_events" => "SELECT COUNT(*) FROM login_events WHERE user_id = $1",
            "login_notify_tokens" => "SELECT COUNT(*) FROM login_notify_tokens WHERE user_id = $1",
            "admin_login_views" => "SELECT COUNT(*) FROM admin_login_views WHERE user_id = $1",
            "sessions" => "SELECT COUNT(*) FROM sessions WHERE user_id = $1",
            _ => panic!("対象外の表"),
        };
        db::query_scalar::<i64>(sql, vec![Bind::from(user)])
            .fetch_one(pool)
            .await
            .expect("件数取得に失敗")
    }
    use backend::services::login_event::admin::record_view as record_admin_view;
    use backend::services::login_event::{ClientInfo, LoginOutcome, insert_login_event_in_tx};
    use backend::services::login_notify::{issue_token, set_setting};

    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");
    assert_eq!(migration_versions(&pool).await, local_migration_versions());

    let user_a = create_test_user(&pool).await;
    let user_b = create_test_user(&pool).await;

    let mut tx = pool.begin_with_user(user_a).await.expect("tx 開始に失敗");
    let current = db::query_scalar::<String>("SELECT current_user", vec![])
        .fetch_one(&mut tx)
        .await
        .expect("利用者名の取得に失敗");
    assert_eq!(current, "app_restricted");
    let setting =
        db::query_scalar::<String>("SELECT current_setting('app.current_user_id')", vec![])
            .fetch_one(&mut tx)
            .await
            .expect("利用者 ID の取得に失敗");
    assert_eq!(setting, user_a.to_string());
    tx.rollback().await.expect("rollback に失敗");
    let current = db::query_scalar::<String>("SELECT current_user", vec![])
        .fetch_one(&pool)
        .await
        .expect("利用者名の取得に失敗");
    assert_eq!(current, "postgres");

    dividend_svc::bulk_create(
        &pool,
        user_a,
        &[make_dividend_item("9931")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("一括登録は通るはず");
    dividend_svc::bulk_create(
        &pool,
        user_b,
        &[make_dividend_item("9932")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("一括登録は通るはず");
    assert_eq!(count_where(&pool, "dividends", user_a).await, 1);
    assert_eq!(count_where(&pool, "dividends", user_b).await, 1);

    let uploaded = dividend_svc::DividendDomain::upload_csv(
        &pool,
        user_a,
        make_dividend_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("CSV 取り込みは通るはず");
    assert_eq!(uploaded.inserted, 1);
    backend::services::csv::record_import_success(&pool, user_a, "dividend", &uploaded)
        .await
        .expect("履歴記録は通るはず");
    assert_eq!(count_where(&pool, "dividends", user_a).await, 2);
    assert_eq!(count_where(&pool, "dividends", user_b).await, 1);
    assert_eq!(count_where(&pool, "csv_import_history", user_a).await, 1);
    assert_eq!(count_where(&pool, "csv_import_history", user_b).await, 0);

    asset_balance_svc::bulk_create(
        &pool,
        user_a,
        &[make_asset_item("9901")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("残高の一括登録は通るはず");
    assert_eq!(count_where(&pool, "asset_balances", user_a).await, 1);
    assert_eq!(count_where(&pool, "asset_balances", user_b).await, 0);

    let mut tx = pool.begin_with_user(user_a).await.expect("tx 開始に失敗");
    insert_login_event_in_tx(
        &mut tx,
        user_a,
        LoginOutcome::Success,
        vec![61u8; 32],
        &ClientInfo::default(),
    )
    .await
    .expect("ログイン記録は通るはず");
    tx.commit().await.expect("commit に失敗");
    let mut tx = pool.begin_with_user(user_b).await.expect("tx 開始に失敗");
    insert_login_event_in_tx(
        &mut tx,
        user_b,
        LoginOutcome::Success,
        vec![62u8; 32],
        &ClientInfo::default(),
    )
    .await
    .expect("ログイン記録は通るはず");
    tx.commit().await.expect("commit に失敗");
    assert_eq!(count_where(&pool, "login_events", user_a).await, 1);
    assert_eq!(count_where(&pool, "login_events", user_b).await, 1);

    let token = issue_token(&pool, user_a)
        .await
        .expect("通知トークンの発行は通るはず");
    assert_ne!(token, "");
    assert_eq!(count_where(&pool, "login_notify_tokens", user_a).await, 1);
    assert_eq!(count_where(&pool, "login_notify_tokens", user_b).await, 0);

    assert!(
        !set_setting(&pool, user_a, false)
            .await
            .expect("通知設定の更新は通るはず")
    );
    assert!(
        !backend::services::login_notify::get_setting(&pool, user_a)
            .await
            .expect("設定取得は通るはず")
    );
    assert!(
        backend::services::login_notify::get_setting(&pool, user_b)
            .await
            .expect("設定取得は通るはず")
    );
    set_setting(&pool, user_a, true)
        .await
        .expect("設定値の復元に失敗");

    record_admin_view(&pool, user_a)
        .await
        .expect("閲覧記録は通るはず");
    assert_eq!(count_where(&pool, "admin_login_views", user_a).await, 1);
    assert_eq!(count_where(&pool, "admin_login_views", user_b).await, 0);

    db::query(
        "INSERT INTO sessions (user_id, token_hash, device_hash) VALUES ($1, $2, $3), ($1, $4, $5)",
        vec![
            Bind::from(user_a),
            Bind::Bytes(vec![71u8; 32]),
            Bind::Bytes(vec![81u8; 32]),
            Bind::Bytes(vec![72u8; 32]),
            Bind::Bytes(vec![82u8; 32]),
        ],
    )
    .execute(&pool)
    .await
    .expect("セッションの投入に失敗");
    db::query(
        "INSERT INTO sessions (user_id, token_hash) VALUES ($1, $2)",
        vec![Bind::from(user_b), Bind::Bytes(vec![73u8; 32])],
    )
    .execute(&pool)
    .await
    .expect("セッションの投入に失敗");
    assert_eq!(count_where(&pool, "sessions", user_a).await, 2);
    assert_eq!(count_where(&pool, "sessions", user_b).await, 1);

    let device_id = backend::services::login_event::device_opaque_id(&user_a, &[81u8; 32]);
    let (removed, revoked_current) =
        backend::services::login_event::revoke_device(&pool, user_a, &device_id, &[82u8; 32])
            .await
            .expect("端末解除は通るはず")
            .expect("対象端末が見つかるはず");
    assert_eq!(removed, 1);
    assert!(!revoked_current);
    let device_log = backend::services::operation_log::list(&pool, user_a, 100, 0)
        .await
        .expect("端末解除の記録取得")
        .into_iter()
        .find(|entry| entry.operation == "device_revoke")
        .expect("端末解除の記録があるはず");
    assert_eq!(device_log.target, "device");
    assert_eq!(device_log.count, 1);
    let removed_again =
        backend::services::login_event::revoke_device(&pool, user_a, &device_id, &[82u8; 32])
            .await
            .expect("再解除");
    assert!(removed_again.is_none());
    assert_eq!(count_where(&pool, "sessions", user_a).await, 1);
    assert_eq!(count_where(&pool, "sessions", user_b).await, 1);

    let user_c = create_test_user(&pool).await;
    dividend_svc::bulk_create(
        &pool,
        user_c,
        &[make_dividend_item("9933")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("利用者 C の投入に失敗");
    let deleted = bulk::delete_all::<dividend_svc::DividendDomain>(&pool, user_c)
        .await
        .expect("全削除は通るはず");
    assert_eq!(deleted, 1);
    let audit_count: i64 = db::query_scalar(
        "SELECT count FROM operation_logs WHERE user_id = $1 AND operation = 'csv_delete' AND target = 'dividends' ORDER BY id DESC LIMIT 1",
        vec![Bind::from(user_c)],
    )
    .fetch_one_with_user(&pool, user_c)
    .await
    .expect("削除操作記録を取得");
    assert_eq!(audit_count, 1);
    let other_user_logs = backend::services::operation_log::list(&pool, user_a, 100, 0)
        .await
        .expect("利用者 A の操作記録取得");
    assert!(
        other_user_logs
            .iter()
            .all(|entry| entry.target != "dividends")
    );
    let before_failed_operation: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM operation_logs WHERE user_id = $1",
        vec![Bind::from(user_c)],
    )
    .fetch_one_with_user(&pool, user_c)
    .await
    .expect("操作記録件数取得");
    let mut failed_tx = pool.begin_with_user(user_c).await.expect("Tx 開始");
    backend::services::operation_log::record(&mut failed_tx, user_c, "csv_delete", "dividends", 1)
        .await
        .expect("失敗操作の記録投入");
    assert!(
        db::query("SELECT * FROM operation_log_missing_table", vec![])
            .fetch_all::<(i64,), _>(&mut failed_tx)
            .await
            .is_err()
    );
    drop(failed_tx);
    let after_failed_operation: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM operation_logs WHERE user_id = $1",
        vec![Bind::from(user_c)],
    )
    .fetch_one_with_user(&pool, user_c)
    .await
    .expect("失敗後の操作記録件数取得");
    assert_eq!(before_failed_operation, after_failed_operation);
    db::query(
        "UPDATE operation_logs SET occurred_at = NOW() - INTERVAL '2 years' WHERE user_id = $1",
        vec![Bind::from(user_c)],
    )
    .execute(&pool)
    .await
    .expect("古い操作記録の準備");
    assert_eq!(
        backend::services::operation_log::prune(&pool)
            .await
            .expect("古い操作記録の掃除"),
        u64::try_from(before_failed_operation).expect("操作記録件数"),
    );
    assert_eq!(count_where(&pool, "dividends", user_a).await, 2);
    assert_eq!(count_where(&pool, "dividends", user_b).await, 1);
    assert_eq!(count_where(&pool, "dividends", user_c).await, 0);

    let info_c = GoogleUserInfo {
        sub: "rls-5c-google-c".to_string(),
        email: "rls-5c-c@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    let token_c = auth_svc::upsert_user_and_rotate_session(&pool, &info_c)
        .await
        .expect("ログインは通るはず");
    let user_c_login = auth_svc::select_user_id_by_session(&pool, token_c)
        .await
        .expect("セッション照会は通るはず")
        .expect("新しい利用者のセッションがあるはず");
    let token_c2 = auth_svc::upsert_user_and_rotate_session(&pool, &info_c)
        .await
        .expect("再ログインは通るはず");
    assert!(
        auth_svc::select_user_id_by_session(&pool, token_c)
            .await
            .expect("旧セッション参照失敗")
            .is_none()
    );
    assert_eq!(
        auth_svc::select_user_id_by_session(&pool, token_c2)
            .await
            .expect("新セッション参照失敗"),
        Some(user_c_login)
    );
    assert_eq!(count_where(&pool, "sessions", user_a).await, 1);
    assert_eq!(count_where(&pool, "sessions", user_b).await, 1);

    let info_d = GoogleUserInfo {
        sub: "rls-5c-google-d".to_string(),
        email: "rls-5c-d@example.com".to_string(),
        name: None,
        picture_url: None,
    };
    let (token_d, user_d, outcome_d) = auth_svc::upsert_user_rotate_session_with_login_event(
        &pool,
        &info_d,
        vec![93u8; 32],
        &ClientInfo::default(),
    )
    .await
    .expect("端末付きログインは通るはず");
    assert_eq!(outcome_d, LoginOutcome::NewDevice);
    assert_eq!(
        auth_svc::select_user_id_by_session(&pool, token_d)
            .await
            .expect("セッション照会は通るはず"),
        Some(user_d.id)
    );
    assert_eq!(count_where(&pool, "login_events", user_d.id).await, 1);
    assert_eq!(count_where(&pool, "login_events", user_a).await, 1);
    assert_eq!(count_where(&pool, "login_events", user_b).await, 1);
    assert_eq!(count_where(&pool, "sessions", user_a).await, 1);
    assert_eq!(count_where(&pool, "sessions", user_b).await, 1);

    let cleared = auth_svc::delete_all_sessions(&pool, user_a)
        .await
        .expect("全セッション削除は通るはず");
    assert_eq!(cleared, 1);
    let logout_logs = backend::services::operation_log::list(&pool, user_a, 100, 0)
        .await
        .expect("全端末ログアウトの記録一覧");
    assert!(logout_logs.iter().any(|entry| {
        entry.operation == "logout_all" && entry.target == "sessions" && entry.count == 1
    }));
    assert_eq!(count_where(&pool, "sessions", user_a).await, 0);
    assert_eq!(count_where(&pool, "sessions", user_b).await, 1);

    auth_svc::delete_account(&pool, user_c)
        .await
        .expect("利用者削除は通るはず");
    let remaining: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM users WHERE id = $1",
        vec![Bind::from(user_c)],
    )
    .fetch_one(&pool)
    .await
    .expect("件数取得に失敗");
    assert_eq!(remaining, 0);
    assert_eq!(count_where(&pool, "dividends", user_a).await, 2);
    assert_eq!(count_where(&pool, "dividends", user_b).await, 1);

    let setting = db::query_scalar::<Option<String>>(
        "SELECT current_setting('app.current_user_id', true)",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("設定値の取得に失敗");
    assert!(setting.as_deref().is_none_or(str::is_empty));
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn operation_log_services_record_list_and_prune() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let pool = connect_with_retry(&format!(
        "postgres://postgres:postgres@127.0.0.1:{port}/postgres"
    ))
    .await;
    run_migrations(&pool).await.expect("migrations failed");
    let user = create_test_user(&pool).await;
    let mut tx = pool.begin_with_user(user).await.expect("Tx 開始");
    backend::services::operation_log::record(&mut tx, user, "logout_all", "sessions", 3)
        .await
        .expect("記録追加");
    tx.commit().await.expect("Tx commit");

    let entries = backend::services::operation_log::list(&pool, user, 10, 0)
        .await
        .expect("記録一覧");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].operation, "logout_all");
    assert_eq!(entries[0].target, "sessions");
    assert_eq!(entries[0].count, 3);

    pool.batch_execute("ALTER ROLE app_worker PASSWORD 'operation-log-test'")
        .await
        .expect("app_worker のテスト用パスワード設定");
    let worker_url = format!("postgres://app_worker:operation-log-test@127.0.0.1:{port}/postgres");
    let worker_pool = connect_with_retry(&worker_url).await;
    assert!(
        db::query("SELECT id FROM operation_logs LIMIT 1", vec![])
            .fetch_all::<(i64,), _>(&worker_pool)
            .await
            .is_err()
    );
    assert!(
        db::query(
            "INSERT INTO operation_logs (user_id, operation, target, count) VALUES ($1, 'logout_all', 'sessions', 1)",
            vec![Bind::from(user)],
        )
        .execute(&worker_pool)
        .await
        .is_err()
    );

    db::query(
        "UPDATE operation_logs SET occurred_at = NOW() - INTERVAL '2 years' WHERE user_id = $1",
        vec![Bind::from(user)],
    )
    .execute(&pool)
    .await
    .expect("古い記録の準備");
    let pruned: i64 = db::query_scalar(
        "SELECT public.prune_operation_logs_older_than_one_year_v1()",
        vec![],
    )
    .fetch_one(&worker_pool)
    .await
    .expect("worker から掃除関数を実行");
    assert_eq!(pruned, 1);
    assert!(
        backend::services::operation_log::list(&pool, user, 10, 0)
            .await
            .expect("掃除後一覧")
            .is_empty()
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn rls_5d_reads_via_app_enter() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");

    let user_a = create_test_user(&pool).await;
    let user_b = create_test_user(&pool).await;
    dividend_svc::bulk_create(
        &pool,
        user_a,
        &[make_dividend_item("9931")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("利用者 A の配当を登録できるはず");
    dividend_svc::bulk_create(
        &pool,
        user_b,
        &[make_dividend_item("9932")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("利用者 B の配当を登録できるはず");

    let mut params = dividend_search_params_with_pagination(PaginationParams::default());
    params.search.include_summary = Some(true);
    params.search.include_facets = Some(true);
    let result = domain_search::<dividend_svc::DividendDomain>(&pool, user_a, params)
        .await
        .expect("制限ロールでの一覧読み取りは成功するはず");
    assert_eq!(result.total, 1, "利用者 A の件数だけ返るはず");
    assert_eq!(result.data.len(), 1, "利用者 A のデータだけ返るはず");
    assert!(result.data.iter().all(|row| row.user_id == user_a));

    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM dividends", vec![])
        .fetch_one_with_user(&pool, user_a)
        .await
        .expect("RLS 読み取りは成功するはず");
    assert_eq!(count, 1, "RLS が別利用者の行を隠すはず");

    let user = db::query_scalar::<String>("SELECT current_user", vec![])
        .fetch_one(&pool)
        .await
        .expect("利用者名の取得に失敗");
    assert_eq!(user, "postgres", "Tx 終了後は所有者に戻るはず");
    let setting = db::query_scalar::<Option<String>>(
        "SELECT NULLIF(current_setting('app.current_user_id', true), '')",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .expect("利用者 ID 設定の取得に失敗");
    assert!(setting.is_none(), "Tx 終了後に利用者 ID が残らないはず");

    let token = "5d-test-token";
    let token_hash = backend::services::login_notify::token_hash(token);
    db::query(
        "INSERT INTO login_notify_tokens (user_id, token_hash) VALUES ($1, $2)",
        vec![Bind::from(user_a), Bind::Bytes(token_hash.clone())],
    )
    .execute(&pool)
    .await
    .expect("通知 token を登録できるはず");
    let peeked = backend::services::login_notify::peek_token(&pool, token)
        .await
        .expect("通知 token の照会は成功するはず");
    assert_eq!(peeked.map(|(id, _)| id), Some(user_a));
    let consumed = backend::services::login_notify::consume_token(&pool, token)
        .await
        .expect("通知 token の消費は成功するはず");
    assert_eq!(consumed, Some(user_a));
}
