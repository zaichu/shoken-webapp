#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header::ACCESS_CONTROL_ALLOW_ORIGIN},
    middleware,
    routing::get,
};
use backend::services::csv::pipeline::CsvImport;
use backend::{
    config::{self, Config},
    db::{self, Bind, Db, Executor, connect_pool_lazy, run_migrations},
    middleware::validate_origin,
    models::asset_balance::{AssetBalanceSearchQueryParams, CreateAssetBalanceRequest},
    models::common::{PaginationParams, SearchQueryParams},
    models::csv_import::CsvUploadResponse,
    models::dividend::{CreateDividendRequest, DividendSearchQueryParams},
    models::domestic_stock::{CreateDomesticStockRequest, DomesticStockSearchQueryParams},
    models::fund_balance::{FundBalance, FundBalanceSearchQueryParams},
    models::mutualfund::{CreateMutualfundRequest, MutualfundSearchQueryParams},
    models::user::GoogleUserInfo,
    services::asset_balance as asset_balance_svc,
    services::auth::{self as auth_svc, SessionToken},
    services::dividend as dividend_svc,
    services::dividend_cache,
    services::domain::bulk::{self, RowLimit},
    services::domain::search::search as domain_search,
    services::domestic_stock as domestic_stock_svc,
    services::fund_balance::FundBalanceDomain,
    services::fund_price::{self, FundNav},
    services::jquants::JQuantsClient,
    services::mutualfund as mutualfund_svc,
    services::price_cron,
    services::stock_master,
    services::stock_price::{self, ClosePrice},
    state::{AppState, Secrets},
};

use backend::test_constants::POSTGRES_TAG;
use rust_decimal_macros::dec;
use shared::dividend_per_share::DividendCacheStatus;
use shared::value::{SecurityCode, UserId};
use std::{sync::Arc, time::Duration};
use testcontainers::{ImageExt, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres;
use time::format_description::well_known::Rfc3339;
use time::{Date, OffsetDateTime};
use tokio::time::{sleep, timeout};
use tower::ServiceExt;
use uuid::Uuid;

mod common;

use common::*;

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn db_integration_with_docker_and_migrations() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    db::query(
        r#"
        INSERT INTO stock
          (date, code, name, market_category, industry_category_33,
           industry_category_17, size_category)
        VALUES
          ($1, $2, $3, $4, $5, $6, $7)
        "#,
        vec![
            Bind::from(
                Date::from_calendar_date(2025, time::Month::try_from(3).unwrap(), 24).unwrap(),
            ),
            Bind::from("9931"),
            Bind::from("テスト株式会社"),
            Bind::from("プライム"),
            Bind::from(Some("情報・通信業".to_string())),
            Bind::from(Some("情報通信".to_string())),
            Bind::from(Some("大型株".to_string())),
        ],
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test stock");

    let secrets = Arc::new(Secrets {
        jquants_api_key: None,
        google_client_id: None,
        google_client_secret: None,
        frontend_url: "http://localhost:8080".to_string(),
        gmail_user: None,
        gmail_app_password: None,
    });
    let state = AppState {
        pool,
        secrets,
        config: Arc::new(Config {
            cors_origins: vec!["https://shoken-webapp.pages.dev".to_string()],
            ..Config::default()
        }),
        google_oauth: None,
        jquants_client: None,
    };

    // worker_entry と同じレイヤ順（origin 検証 → CORS）を再現する
    let allowed_origins = Arc::new(state.config.cors_origins.clone());
    let strict_origin_check = state.config.is_production();
    let app = backend::handlers::v1::stock_search_routes()
        .route("/health", get(|| async { "OK" }))
        .layer(middleware::from_fn(move |req, next| {
            let origins = allowed_origins.clone();
            async move { validate_origin(origins, strict_origin_check, req, next).await }
        }))
        .layer(config::build_cors_layer(
            &state.config.cors_origins,
            !state.config.is_production(),
        ))
        .with_state(state);

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/stocks?query=9931")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let stock: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(stock["code"], "9931");

    let req = Request::builder()
        .method(Method::OPTIONS)
        .uri("/health")
        .header("origin", "https://shoken-webapp.pages.dev")
        .header("access-control-request-method", "GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let allowed_origin = resp
        .headers()
        .get(ACCESS_CONTROL_ALLOW_ORIGIN)
        .and_then(|value| value.to_str().ok());
    assert_eq!(allowed_origin, Some("https://shoken-webapp.pages.dev"));
}

/// 空の DB には土台が適用され、再実行は履歴を変えない
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn fresh_database_applies_all_migrations() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool)
        .await
        .expect("空 DB への全適用は成功するはず");
    assert_eq!(migration_versions(&pool).await, local_migration_versions());

    run_migrations(&pool)
        .await
        .expect("適用済み DB の再実行は成功するはず");
    assert_eq!(migration_versions(&pool).await, local_migration_versions());
}

/// 旧 migration を全て適用済みの DB(本番)では、土台の SQL を流さず履歴だけを足し、
/// 土台の後ろの migration だけを当てる。結果は空の DB から作った DB と同じスキーマになり、
/// 既存の行は残り、再実行も履歴を変えない
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn legacy_database_records_baseline_without_applying() {
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
    let fresh_node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let fresh_port = fresh_node.get_host_port_ipv4(5432).await.unwrap();
    let fresh = connect_with_retry(&format!(
        "postgres://postgres:postgres@127.0.0.1:{fresh_port}/postgres"
    ))
    .await;
    run_migrations(&fresh).await.expect("空 DB への全適用");

    apply_legacy_migrations(&pool, legacy_last_version()).await;
    create_test_user(&pool).await;

    run_migrations(&pool)
        .await
        .expect("旧 migration 適用済み DB では土台の記録だけで成功するはず");
    let expected: Vec<i64> = legacy_migration_files()
        .into_iter()
        .map(|(version, _, _)| version)
        .chain(local_migration_versions())
        .collect();
    assert_eq!(migration_versions(&pool).await, expected);
    assert_eq!(
        schema_fingerprint(&pool).await,
        schema_fingerprint(&fresh).await,
        "空の DB から作った DB と同じスキーマになる"
    );
    let users = db::query_scalar::<i64>("SELECT count(*) FROM users", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(users, 1, "既存の行は残る");
    let baseline_count = local_migration_versions()
        .into_iter()
        .filter(|version| *version <= baseline_last_version())
        .count();
    let recorded_only = db::query_scalar::<i64>(
        "SELECT count(*) FROM _sqlx_migrations
         WHERE version > $1 AND version <= $2 AND success AND execution_time = 0",
        vec![
            Bind::from(legacy_last_version()),
            Bind::from(baseline_last_version()),
        ],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        usize::try_from(recorded_only).unwrap(),
        baseline_count,
        "土台は実行せずに記録だけされる"
    );

    run_migrations(&pool)
        .await
        .expect("記録後の再実行は成功するはず(土台の checksum が一致する)");
    assert_eq!(migration_versions(&pool).await, expected);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn session_migration_keeps_existing_old_sessions_valid_immediately() {
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

    apply_legacy_migrations(&pool, legacy_last_version()).await;
    let user_id = create_test_user(&pool).await;
    let token = SessionToken::new();
    db::query(
        "INSERT INTO sessions (user_id, token_hash, created_at, expires_at)
         VALUES ($1, $2, NOW() - INTERVAL '1 day', NOW() + INTERVAL '6 days')",
        vec![Bind::from(user_id), Bind::Bytes(token.hash())],
    )
    .execute(&pool)
    .await
    .expect("旧形式のセッションを作成");

    run_migrations(&pool).await.expect("0107 を適用");

    let last_used_at = db::query_scalar::<time::OffsetDateTime>(
        "SELECT last_used_at FROM sessions WHERE token_hash = $1",
        vec![Bind::Bytes(token.hash())],
    )
    .fetch_one(&pool)
    .await
    .expect("migration後の最終利用時刻を取得");
    let migration_recent = db::query_scalar::<bool>(
        "SELECT last_used_at > NOW() - INTERVAL '30 minutes'
         FROM sessions WHERE token_hash = $1",
        vec![Bind::Bytes(token.hash())],
    )
    .fetch_one(&pool)
    .await
    .expect("最終利用時刻を確認");
    assert!(migration_recent, "既存行は migration 時刻で初期化される");
    assert!(
        last_used_at > time::OffsetDateTime::now_utc() - time::Duration::minutes(30),
        "作成から1日経った既存セッションでも migration 直後は有効"
    );
    assert!(
        auth_svc::select_user_by_session(&pool, token)
            .await
            .expect("migration後の認証照会")
            .is_some(),
        "migration直後は既存セッションで認証できる"
    );
}

/// 旧 migration が途中までの DB は、続きを当てられないため状態と対処を示して止める
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn partially_applied_legacy_database_stops() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let pool = connect_with_retry(&database_url).await;
    apply_legacy_migrations(&pool, 20).await;

    let err = run_migrations(&pool)
        .await
        .expect_err("途中までの旧 migration はエラーで止まるはず");
    assert!(err.contains("0020 までしか適用されていません"), "{err}");
    assert!(err.contains("make db-reset"), "{err}");
    assert_eq!(
        migration_versions(&pool).await,
        (0..=20).collect::<Vec<_>>(),
        "土台は記録しない"
    );
}

/// 想定外の履歴は状態と対処を示して止める: 履歴に対応するファイルが無い場合
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn unknown_history_version_stops_migration() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let pool = connect_with_retry(&database_url).await;
    seed_migration_history(&pool, &[99]).await;

    let err = run_migrations(&pool)
        .await
        .expect_err("未知の履歴はエラーで止まるはず");
    assert!(
        err.contains("対応するファイルがありません"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("make db-reset"),
        "対処法(db-reset)への言及があるはず: {err}"
    );
}

/// 想定外の履歴は状態と対処を示して止める: 適用済み内容が現行ファイルと異なる場合
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn mismatched_history_stops_migration() {
    let node = Postgres::default()
        .with_tag(POSTGRES_TAG)
        .start()
        .await
        .unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let pool = connect_with_retry(&database_url).await;
    seed_migration_history(&pool, &local_migration_versions()).await;

    let err = run_migrations(&pool)
        .await
        .expect_err("内容の異なる履歴はエラーで止まるはず");
    assert!(
        err.contains("現行ファイルと一致しません"),
        "unexpected error: {err}"
    );
    assert!(
        err.contains("make db-reset"),
        "対処法(db-reset)への言及があるはず: {err}"
    );
}
