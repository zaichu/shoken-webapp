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
async fn stock_master_replace_master_swaps_all_rows() {
    let (pool, _node) = start_test_pool().await;

    db::query(
        "INSERT INTO stock (date, code, name, market_category) VALUES ($1, $2, $3, $4)",
        vec![
            Bind::from(
                Date::from_calendar_date(2025, time::Month::try_from(3).unwrap(), 24).unwrap(),
            ),
            Bind::from("9990"),
            Bind::from("旧銘柄"),
            Bind::from("スタンダード"),
        ],
    )
    .execute(&pool)
    .await
    .expect("Failed to seed stock");

    let rows = vec![
        stock_master::StockRow {
            date: Date::from_calendar_date(2026, time::Month::try_from(10).unwrap(), 15).unwrap(),
            code: "9931".to_string(),
            name: "架空水産".to_string(),
            market_category: "プライム".to_string(),
            industry_category_33: Some("水産・農林業".to_string()),
            industry_category_17: Some("食品".to_string()),
            size_category: Some("TOPIX Small 1".to_string()),
        },
        stock_master::StockRow {
            date: Date::from_calendar_date(2026, time::Month::try_from(10).unwrap(), 15).unwrap(),
            code: "9904".to_string(),
            name: "架空自動車".to_string(),
            market_category: "プライム".to_string(),
            industry_category_33: None,
            industry_category_17: None,
            size_category: None,
        },
    ];

    let inserted = stock_master::replace_master(&pool, &rows)
        .await
        .expect("replace_master failed");
    assert_eq!(inserted, 2);

    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM stock", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 2, "1銘柄1行かつ旧行が全置換されているはず");

    let name = db::query_scalar::<String>(
        "SELECT name::text FROM stock WHERE code = $1",
        vec![Bind::from("9931")],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(name, "架空水産");
    assert!(
        db::query_scalar::<i64>("SELECT COUNT(*) FROM stock WHERE code = '9990'", vec![])
            .fetch_one(&pool)
            .await
            .unwrap()
            == 0,
        "旧行が残っている"
    );

    // code は主キー: 同じ code の行は挿入できない
    let err = db::query(
        "INSERT INTO stock (date, code, name, market_category) VALUES ($1, $2, $3, $4)",
        vec![
            Bind::from(
                Date::from_calendar_date(2026, time::Month::try_from(10).unwrap(), 16).unwrap(),
            ),
            Bind::from("9931"),
            Bind::from("重複銘柄"),
            Bind::from("プライム"),
        ],
    )
    .execute(&pool)
    .await
    .expect_err("code 重複の INSERT は失敗するはず");
    assert!(err.is_unique_violation(), "期待するのは PK 違反: {err:?}");
}

/// 受け入れ条件①: 枠の行が無い初回は、枠を取って更新を実行する。
/// ホスト側テストでは J-Quants 呼び出し自体は必ず失敗するため、
/// Err = 枠を取って更新を試みた、Ok(None) = 枠が取れずスキップ、を示す
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_master_refresh_runs_when_gate_row_missing() {
    let (pool, _node) = start_test_pool().await;
    seed_stock_row(&pool).await;
    assert_eq!(master_refresh_gate(&pool).await, None);
    let client = JQuantsClient::new("test-api-key".to_string());

    let result = stock_master::maybe_refresh(&pool, &client, scheduled_now_ms()).await;
    assert!(
        result.is_err(),
        "枠の行が無い初回は枠を取って更新を試みるはず(取得自体はホストでは失敗): {result:?}"
    );
    assert!(
        master_refresh_gate(&pool).await.is_some(),
        "枠の行が作られているはず"
    );
}

/// 受け入れ条件②: 枠を取った直後の実行は、枠が取れず更新しない
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_master_refresh_skips_when_slot_taken_recently() {
    let (pool, _node) = start_test_pool().await;
    seed_stock_row(&pool).await;
    let client = JQuantsClient::new("test-api-key".to_string());

    // 1回目: 枠を取って更新を試みる(失敗しても再試行は1時間後に押される)
    let first = stock_master::maybe_refresh(&pool, &client, scheduled_now_ms()).await;
    assert!(
        first.is_err(),
        "1回目は枠を取って更新を試みるはず: {first:?}"
    );

    // 直後の2回目: next_available_at が未来なので枠が取れず更新しない
    let second = stock_master::maybe_refresh(&pool, &client, scheduled_now_ms()).await;
    assert_eq!(
        second.unwrap(),
        None,
        "直後の実行は枠が取れずスキップするはず"
    );
    let count = db::query_scalar::<i64>("SELECT COUNT(*) FROM stock", vec![])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "stock の行は変わらないはず");
}

/// 受け入れ条件③: next_available_at が過去なら枠を取って更新を実行する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_master_refresh_runs_when_slot_is_due() {
    let (pool, _node) = start_test_pool().await;
    seed_stock_row(&pool).await;
    let client = JQuantsClient::new("test-api-key".to_string());
    db::query(
        "INSERT INTO market_data_provider_rate_control (provider, next_available_at)
         VALUES ('jquants-equities-master', NOW() - INTERVAL '1 day')",
        vec![],
    )
    .execute(&pool)
    .await
    .expect("枠の行の投入");

    let result = stock_master::maybe_refresh(&pool, &client, scheduled_now_ms()).await;
    assert!(
        result.is_err(),
        "過去の next_available_at は枠を取って更新を試みるはず: {result:?}"
    );
}

/// stock が空でも、失敗待ち・実行中の枠(未来の next_available_at)は待つ。
/// 枠の行が無い初回は初期投入を実行する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_master_refresh_empty_stock_waits_for_slot_but_runs_without_row() {
    let (pool, _node) = start_test_pool().await;
    let client = JQuantsClient::new("test-api-key".to_string());

    // 失敗待ち・実行中の枠(未来)は、stock が空でも強制取得しない
    db::query(
        "INSERT INTO market_data_provider_rate_control (provider, next_available_at)
         VALUES ('jquants-equities-master', NOW() + INTERVAL '1 hour')",
        vec![],
    )
    .execute(&pool)
    .await
    .expect("枠の行の投入");
    let skipped = stock_master::maybe_refresh(&pool, &client, scheduled_now_ms()).await;
    assert_eq!(skipped.unwrap(), None, "失敗待ちの枠は空でも実行しない");

    // 枠の行が無い初回は初期投入を試みる
    db::query(
        "DELETE FROM market_data_provider_rate_control WHERE provider = 'jquants-equities-master'",
        vec![],
    )
    .execute(&pool)
    .await
    .expect("枠の行の削除");
    let result = stock_master::maybe_refresh(&pool, &client, scheduled_now_ms()).await;
    assert!(
        result.is_err(),
        "枠の行が無ければ空でも初期投入を試みるはず: {result:?}"
    );
}

/// 受け入れ条件④: 更新に失敗したら再試行は1時間後になる(毎分のやり直しをしない)
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_master_refresh_postpones_slot_one_hour_on_failure() {
    let (pool, _node) = start_test_pool().await;
    seed_stock_row(&pool).await;
    let client = JQuantsClient::new("test-api-key".to_string());

    let result = stock_master::maybe_refresh(&pool, &client, scheduled_now_ms()).await;
    assert!(result.is_err(), "ホストでは取得が失敗するはず: {result:?}");

    let next = master_refresh_gate(&pool).await.expect("枠の行があるはず");
    let now = OffsetDateTime::now_utc();
    assert!(
        next > now + time::Duration::minutes(55) && next < now + time::Duration::minutes(65),
        "失敗後の再試行は1時間後になるはず: {next:?}"
    );
}
