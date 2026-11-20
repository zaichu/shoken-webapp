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
async fn service_coverage_all_domains() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    assert_eq!(
        domain_search::<mutualfund_svc::MutualfundDomain>(
            &pool,
            user_id,
            default_mutualfund_search_params(),
        )
        .await
        .expect("initial mutualfund list failed")
        .data,
        [] as [shared::domain::Mutualfund; 0]
    );
    let mutualfund_empty =
        mutualfund_svc::bulk_create(&pool, user_id, &[], RowLimit::new(i64::MAX))
            .await
            .expect("empty mutualfund bulk_create failed");
    assert_eq!(mutualfund_empty.inserted, 0);
    assert_eq!(mutualfund_empty.skipped, 0);

    let mutualfund_items = vec![
        make_mutualfund_item("テスト投信A"),
        make_mutualfund_item("テスト投信B"),
    ];
    let mutualfund_created =
        mutualfund_svc::bulk_create(&pool, user_id, &mutualfund_items, RowLimit::new(i64::MAX))
            .await
            .expect("mutualfund bulk_create failed");
    assert_eq!(mutualfund_created.inserted, 2);
    assert_eq!(mutualfund_created.skipped, 0);

    let mutualfund_uploaded = mutualfund_svc::MutualfundDomain::upload_csv(
        &pool,
        user_id,
        make_mutualfund_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("mutualfund upload_csv failed");
    assert_eq!(mutualfund_uploaded.inserted, 1);
    assert_eq!(mutualfund_uploaded.skipped, 0);
    assert_eq!(
        mutualfund_uploaded.errors,
        [] as [shared::csv_import::CsvRowError; 0]
    );

    assert_eq!(
        domain_search::<mutualfund_svc::MutualfundDomain>(
            &pool,
            user_id,
            default_mutualfund_search_params(),
        )
        .await
        .expect("mutualfund list failed")
        .data
        .len(),
        3
    );
    assert_eq!(
        bulk::delete_all::<mutualfund_svc::MutualfundDomain>(&pool, user_id)
            .await
            .expect("mutualfund delete_all failed"),
        3
    );
    assert_eq!(
        domain_search::<mutualfund_svc::MutualfundDomain>(
            &pool,
            user_id,
            default_mutualfund_search_params(),
        )
        .await
        .expect("mutualfund list after delete failed")
        .data,
        [] as [shared::domain::Mutualfund; 0]
    );

    assert_eq!(
        domain_search::<dividend_svc::DividendDomain>(
            &pool,
            user_id,
            default_dividend_search_params()
        )
        .await
        .expect("initial dividend list failed")
        .data,
        [] as [shared::domain::Dividend; 0]
    );
    let dividend_empty = dividend_svc::bulk_create(&pool, user_id, &[], RowLimit::new(i64::MAX))
        .await
        .expect("empty dividend bulk_create failed");
    assert_eq!(dividend_empty.inserted, 0);
    assert_eq!(dividend_empty.skipped, 0);

    let dividend_items = vec![make_dividend_item("9931"), make_dividend_item("9932")];
    let dividend_created =
        dividend_svc::bulk_create(&pool, user_id, &dividend_items, RowLimit::new(i64::MAX))
            .await
            .expect("dividend bulk_create failed");
    assert_eq!(dividend_created.inserted, 2);
    assert_eq!(dividend_created.skipped, 0);

    let dividend_uploaded = dividend_svc::DividendDomain::upload_csv(
        &pool,
        user_id,
        make_dividend_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("dividend upload_csv failed");
    assert_eq!(dividend_uploaded.inserted, 1);
    assert_eq!(dividend_uploaded.skipped, 0);
    assert_eq!(
        dividend_uploaded.errors,
        [] as [shared::csv_import::CsvRowError; 0]
    );

    assert_eq!(
        domain_search::<dividend_svc::DividendDomain>(
            &pool,
            user_id,
            default_dividend_search_params()
        )
        .await
        .expect("dividend list failed")
        .data
        .len(),
        3
    );
    assert_eq!(
        bulk::delete_all::<dividend_svc::DividendDomain>(&pool, user_id)
            .await
            .expect("dividend delete_all failed"),
        3
    );
    assert_eq!(
        domain_search::<dividend_svc::DividendDomain>(
            &pool,
            user_id,
            default_dividend_search_params()
        )
        .await
        .expect("dividend list after delete failed")
        .data,
        [] as [shared::domain::Dividend; 0]
    );

    assert_eq!(
        domain_search::<domestic_stock_svc::DomesticStockDomain>(
            &pool,
            user_id,
            default_domestic_stock_search_params(),
        )
        .await
        .expect("initial domestic_stock list failed")
        .data,
        [] as [shared::domain::DomesticStock; 0]
    );
    let domestic_stock_empty =
        domestic_stock_svc::bulk_create(&pool, user_id, &[], RowLimit::new(i64::MAX))
            .await
            .expect("empty domestic_stock bulk_create failed");
    assert_eq!(domestic_stock_empty.inserted, 0);
    assert_eq!(domestic_stock_empty.skipped, 0);

    let domestic_stock_items = vec![
        make_domestic_stock_item("9935"),
        make_domestic_stock_item("9936"),
    ];
    let domestic_stock_created = domestic_stock_svc::bulk_create(
        &pool,
        user_id,
        &domestic_stock_items,
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("domestic_stock bulk_create failed");
    assert_eq!(domestic_stock_created.inserted, 2);
    assert_eq!(domestic_stock_created.skipped, 0);

    let domestic_stock_uploaded = domestic_stock_svc::DomesticStockDomain::upload_csv(
        &pool,
        user_id,
        make_domestic_stock_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("domestic_stock upload_csv failed");
    assert_eq!(domestic_stock_uploaded.inserted, 1);
    assert_eq!(domestic_stock_uploaded.skipped, 0);
    assert_eq!(
        domestic_stock_uploaded.errors,
        [] as [shared::csv_import::CsvRowError; 0]
    );

    assert_eq!(
        domain_search::<domestic_stock_svc::DomesticStockDomain>(
            &pool,
            user_id,
            default_domestic_stock_search_params(),
        )
        .await
        .expect("domestic_stock list failed")
        .data
        .len(),
        3
    );
    assert_eq!(
        bulk::delete_all::<domestic_stock_svc::DomesticStockDomain>(&pool, user_id)
            .await
            .expect("domestic_stock delete_all failed"),
        3
    );
    assert_eq!(
        domain_search::<domestic_stock_svc::DomesticStockDomain>(
            &pool,
            user_id,
            default_domestic_stock_search_params(),
        )
        .await
        .expect("domestic_stock list after delete failed")
        .data,
        [] as [shared::domain::DomesticStock; 0]
    );

    assert_eq!(
        domain_search::<asset_balance_svc::AssetBalanceDomain>(
            &pool,
            user_id,
            default_asset_balance_search_params(),
        )
        .await
        .expect("initial asset_balance list failed")
        .data,
        [] as [shared::domain::AssetBalance; 0]
    );
    let asset_balance_empty =
        asset_balance_svc::bulk_create(&pool, user_id, &[], RowLimit::new(i64::MAX))
            .await
            .expect("empty asset_balance bulk_create failed");
    assert_eq!(asset_balance_empty.inserted, 0);
    assert_eq!(asset_balance_empty.skipped, 0);

    let asset_balance_items = vec![make_asset_item("9931"), make_asset_item("9901")];
    let asset_balance_created = asset_balance_svc::bulk_create(
        &pool,
        user_id,
        &asset_balance_items,
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("asset_balance bulk_create failed");
    assert_eq!(asset_balance_created.inserted, 2);
    assert_eq!(asset_balance_created.skipped, 0);
    assert_eq!(
        domain_search::<asset_balance_svc::AssetBalanceDomain>(
            &pool,
            user_id,
            default_asset_balance_search_params(),
        )
        .await
        .expect("asset_balance list after bulk_create failed")
        .data
        .len(),
        2
    );

    let asset_balance_uploaded = asset_balance_svc::AssetBalanceDomain::upload_csv(
        &pool,
        user_id,
        make_asset_balance_csv().as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("asset_balance upload_csv failed");
    assert_eq!(asset_balance_uploaded.inserted, 1);
    assert_eq!(asset_balance_uploaded.skipped, 0);
    assert_eq!(
        asset_balance_uploaded.errors,
        [] as [shared::csv_import::CsvRowError; 0]
    );

    let asset_balance_rows = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("asset_balance list failed")
    .data;
    assert_eq!(asset_balance_rows.len(), 1);
    assert_eq!(asset_balance_rows[0].security_code.as_str(), "9904");

    assert_eq!(
        bulk::delete_all::<asset_balance_svc::AssetBalanceDomain>(&pool, user_id)
            .await
            .expect("asset_balance delete_all failed"),
        1
    );
    assert_eq!(
        domain_search::<asset_balance_svc::AssetBalanceDomain>(
            &pool,
            user_id,
            default_asset_balance_search_params(),
        )
        .await
        .expect("asset_balance list after delete failed")
        .data,
        [] as [shared::domain::AssetBalance; 0]
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn asset_balance_bulk_create_replaces_previous_snapshot() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 1回目: 2銘柄を登録
    let items_a = vec![make_asset_item("9931"), make_asset_item("9932")];
    asset_balance_svc::bulk_create(&pool, user_id, &items_a, RowLimit::new(i64::MAX))
        .await
        .expect("1回目 bulk_create 失敗");

    // 2回目: 3銘柄を登録（スナップショット置き換えなので合計3件になるはず）
    let items_b = vec![
        make_asset_item("9941"),
        make_asset_item("9942"),
        make_asset_item("9943"),
    ];
    asset_balance_svc::bulk_create(&pool, user_id, &items_b, RowLimit::new(i64::MAX))
        .await
        .expect("2回目 bulk_create 失敗");

    let rows = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("list 失敗")
    .data;
    assert_eq!(
        rows.len(),
        3,
        "2回目のスナップショットが3件のはずが{}件",
        rows.len()
    );
    let codes: Vec<&str> = rows.iter().map(|r| r.security_code.as_str()).collect();
    assert!(codes.contains(&"9941"), "9941 が存在しない");
    assert!(codes.contains(&"9942"), "9942 が存在しない");
    assert!(codes.contains(&"9943"), "9943 が存在しない");
    assert!(!codes.contains(&"9931"), "9931 が残存している（削除漏れ）");
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn asset_balance_same_code_in_different_accounts() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 同一銘柄を口座別(特定/NISA/口座なし)に持てる
    let mut tokutei = make_asset_item("9904");
    tokutei.account = Some("特定".parse().unwrap());
    let mut nisa = make_asset_item("9904");
    nisa.account = Some("NISA".parse().unwrap());
    let items = vec![tokutei, nisa, make_asset_item("9904")];
    asset_balance_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("口座別の同一銘柄は別行で登録できる");

    let rows = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("list 失敗")
    .data;
    let accounts: Vec<Option<String>> = rows
        .iter()
        .filter(|row| row.security_code.as_str() == "9904")
        .map(|row| row.account.as_ref().map(|a| a.as_str().to_string()))
        .collect();
    let accounts: Vec<Option<&str>> = accounts.iter().map(|a| a.as_deref()).collect();
    assert_eq!(accounts.len(), 3, "口座別の行が揃っていない: {accounts:?}");
    for expected in [Some("特定"), Some("NISA"), None] {
        assert!(accounts.contains(&expected), "{expected:?} がない");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires Docker to run Postgres container"]
async fn asset_balance_bulk_create_concurrent_same_user_no_mix() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let items_a = vec![make_asset_item("A001"), make_asset_item("A002")];
    let items_b = vec![
        make_asset_item("B001"),
        make_asset_item("B002"),
        make_asset_item("B003"),
    ];

    let pool_a = pool.clone();
    let pool_b = pool.clone();

    // 2タスクを同時に起動して advisory lock による直列化を確認
    let (res_a, res_b) = tokio::join!(
        tokio::spawn(async move {
            asset_balance_svc::bulk_create(&pool_a, user_id, &items_a, RowLimit::new(i64::MAX))
                .await
        }),
        tokio::spawn(async move {
            asset_balance_svc::bulk_create(&pool_b, user_id, &items_b, RowLimit::new(i64::MAX))
                .await
        }),
    );
    res_a.unwrap().expect("task_a 失敗");
    res_b.unwrap().expect("task_b 失敗");

    let rows = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("list 失敗")
    .data;

    // advisory lock で直列化されるため、A(2件) か B(3件) のいずれかのみ存在する
    assert!(
        rows.len() == 2 || rows.len() == 3,
        "AとBのデータが混在している可能性（{}件）",
        rows.len()
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_bulk_create_and_list() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let items = vec![
        make_dividend_item("9931"),
        make_dividend_item("9932"),
        make_dividend_item("9933"),
    ];
    let created = dividend_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("dividend bulk_create failed");
    assert_eq!(created.inserted, 3);
    assert_eq!(created.skipped, 0);

    let rows = domain_search::<dividend_svc::DividendDomain>(
        &pool,
        user_id,
        default_dividend_search_params(),
    )
    .await
    .expect("dividend list failed")
    .data;
    assert_eq!(rows.len(), 3);

    let first_page = domain_search::<dividend_svc::DividendDomain>(
        &pool,
        user_id,
        dividend_search_params_with_pagination(PaginationParams {
            page: Some(1),
            per_page: Some(2),
        }),
    )
    .await
    .expect("dividend first page list failed");
    assert_eq!(first_page.total, 3);
    assert_eq!(first_page.page, 1);
    assert_eq!(first_page.per_page, 2);
    assert_eq!(first_page.data.len(), 2);
    assert!(first_page.summary.is_none());
    assert!(first_page.facets.is_none());

    let second_page = domain_search::<dividend_svc::DividendDomain>(
        &pool,
        user_id,
        dividend_search_params_with_pagination(PaginationParams {
            page: Some(2),
            per_page: Some(2),
        }),
    )
    .await
    .expect("dividend second page list failed");
    assert_eq!(second_page.total, 3);
    assert_eq!(second_page.page, 2);
    assert_eq!(second_page.per_page, 2);
    assert_eq!(second_page.data.len(), 1);
    assert!(second_page.summary.is_none());
    assert!(second_page.facets.is_none());

    let clamped_page = domain_search::<dividend_svc::DividendDomain>(
        &pool,
        user_id,
        dividend_search_params_with_pagination(PaginationParams {
            page: Some(1),
            per_page: Some(5000),
        }),
    )
    .await
    .expect("dividend clamped page list failed");
    assert_eq!(clamped_page.total, 3);
    assert_eq!(clamped_page.per_page, 1000);

    let deleted = bulk::delete_all::<dividend_svc::DividendDomain>(&pool, user_id)
        .await
        .expect("dividend delete_all failed");
    assert_eq!(deleted, 3);

    let rows = domain_search::<dividend_svc::DividendDomain>(
        &pool,
        user_id,
        default_dividend_search_params(),
    )
    .await
    .expect("dividend list after delete failed")
    .data;
    assert_eq!(rows.len(), 0);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_bulk_create_skips_duplicates() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let items = vec![make_dividend_item("9934")];

    let first = dividend_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("first dividend bulk_create failed");
    assert_eq!(first.inserted, 1);
    assert_eq!(first.skipped, 0);

    let second = dividend_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("second dividend bulk_create failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 1);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_bulk_create_keeps_duplicate_rows_in_same_batch() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    // 同じ内容の行が CSV に2行あると2行とも入る(occurrence_index で区別)
    let items = vec![make_dividend_item("9934"), make_dividend_item("9934")];

    let first = dividend_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("duplicate dividend bulk_create failed");
    assert_eq!(first.inserted, 2);
    assert_eq!(first.skipped, 0);

    // 同じ2行の再取込では増えない
    let second = dividend_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("duplicate dividend re-upload failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 2);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn domestic_stock_bulk_create_and_list() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let items = vec![
        make_domestic_stock_item("9935"),
        make_domestic_stock_item("9936"),
        make_domestic_stock_item("9937"),
    ];
    let created = domestic_stock_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("domestic_stock bulk_create failed");
    assert_eq!(created.inserted, 3);
    assert_eq!(created.skipped, 0);

    let rows = domain_search::<domestic_stock_svc::DomesticStockDomain>(
        &pool,
        user_id,
        default_domestic_stock_search_params(),
    )
    .await
    .expect("domestic_stock list failed")
    .data;
    assert_eq!(rows.len(), 3);

    let deleted = bulk::delete_all::<domestic_stock_svc::DomesticStockDomain>(&pool, user_id)
        .await
        .expect("domestic_stock delete_all failed");
    assert_eq!(deleted, 3);

    let rows = domain_search::<domestic_stock_svc::DomesticStockDomain>(
        &pool,
        user_id,
        default_domestic_stock_search_params(),
    )
    .await
    .expect("domestic_stock list after delete failed")
    .data;
    assert_eq!(rows.len(), 0);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn domestic_stock_bulk_create_skips_duplicates() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let items = vec![make_domestic_stock_item("9938")];

    let first = domestic_stock_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("first domestic_stock bulk_create failed");
    assert_eq!(first.inserted, 1);
    assert_eq!(first.skipped, 0);

    let second = domestic_stock_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("second domestic_stock bulk_create failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 1);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn mutualfund_bulk_create_and_list() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let items = vec![
        make_mutualfund_item("テスト投信A"),
        make_mutualfund_item("テスト投信B"),
    ];
    let created = mutualfund_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("mutualfund bulk_create failed");
    assert_eq!(created.inserted, 2);
    assert_eq!(created.skipped, 0);

    let rows = domain_search::<mutualfund_svc::MutualfundDomain>(
        &pool,
        user_id,
        default_mutualfund_search_params(),
    )
    .await
    .expect("mutualfund list failed")
    .data;
    assert_eq!(rows.len(), 2);

    let deleted = bulk::delete_all::<mutualfund_svc::MutualfundDomain>(&pool, user_id)
        .await
        .expect("mutualfund delete_all failed");
    assert_eq!(deleted, 2);

    let rows = domain_search::<mutualfund_svc::MutualfundDomain>(
        &pool,
        user_id,
        default_mutualfund_search_params(),
    )
    .await
    .expect("mutualfund list after delete failed")
    .data;
    assert_eq!(rows.len(), 0);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn mutualfund_bulk_create_skips_duplicates() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let items = vec![make_mutualfund_item("テスト投信C")];

    let first = mutualfund_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("first mutualfund bulk_create failed");
    assert_eq!(first.inserted, 1);
    assert_eq!(first.skipped, 0);

    let second = mutualfund_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("second mutualfund bulk_create failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 1);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn mutualfund_bulk_create_keeps_duplicate_rows_in_same_batch() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    // 同じ内容の行が CSV に2行あると2行とも入る(occurrence_index で区別)
    let items = vec![
        make_mutualfund_item("テスト投信C"),
        make_mutualfund_item("テスト投信C"),
    ];

    let first = mutualfund_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("duplicate mutualfund bulk_create failed");
    assert_eq!(first.inserted, 2);
    assert_eq!(first.skipped, 0);

    // 同じ2行の再取込では増えない
    let second = mutualfund_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
        .await
        .expect("duplicate mutualfund re-upload failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 2);
}

/// 0116・0117 の backfill(md5 式)と取り込み(bulk_create 側の md5 式)が一致すること。
/// 0115 相当に戻して旧い経路で端の値の行を入れ、migration を当て直して backfill し、
/// 同じ行を bulk_create で取り込むと 0 件挿入になる(式が1文字でも違うと別物と見なされ二重に入る)
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividends_mutualfunds_backfill_hash_matches_bulk_create() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 0116・0117 を取り消して旧い形に戻す(旧一意キーも戻す)
    pool.batch_execute(
        "DROP INDEX IF EXISTS public.idx_dividends_hash_occurrence; \
         DROP INDEX IF EXISTS public.idx_mutualfunds_hash_occurrence; \
         ALTER TABLE public.dividends DROP COLUMN IF EXISTS content_hash, DROP COLUMN IF EXISTS occurrence_index; \
         ALTER TABLE public.mutualfunds DROP COLUMN IF EXISTS content_hash, DROP COLUMN IF EXISTS occurrence_index;",
    )
    .await
    .expect("0116・0117 の取り消しに失敗");
    pool.batch_execute(
        "CREATE UNIQUE INDEX idx_dividends_unique ON public.dividends USING btree (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax, product, account); \
         CREATE UNIQUE INDEX idx_mutualfunds_unique ON public.mutualfunds USING btree (user_id, trade_date, fund_name, shares, cancellation_amount_yen, account);",
    )
    .await
    .expect("旧一意キーの復元に失敗");
    db::query(
        "DELETE FROM _sqlx_migrations WHERE version IN (116, 117)",
        vec![],
    )
    .execute(&pool)
    .await
    .expect("migration 記録の取り消しに失敗");

    // 旧い経路で端の値の行を入れる(小数・0円・NULL・全角名)
    let div_items = vec![
        CreateDividendRequest {
            settlement_date: Date::from_calendar_date(2024, time::Month::March, 25).unwrap(),
            product: "国内株式".to_string(),
            account: "特定".parse().unwrap(),
            security_code: "9934".to_string(),
            security_name: "銘柄_9934".to_string(),
            unit_price: dec!(12.5),
            shares: dec!(100.0),
            dividends_before_tax: dec!(1000.0),
            taxes: dec!(0),
            net_amount_received: dec!(1000.0),
        },
        CreateDividendRequest {
            settlement_date: Date::from_calendar_date(2024, time::Month::June, 10).unwrap(),
            product: "投資信託".to_string(),
            account: "NISA".parse().unwrap(),
            security_code: String::new(),
            security_name: "ＡＢＣ　投信".to_string(),
            unit_price: dec!(1),
            shares: dec!(0.5),
            dividends_before_tax: dec!(2500.75),
            taxes: dec!(508.25),
            net_amount_received: dec!(1992.5),
        },
    ];
    for item in &div_items {
        let row = db::query(
            "INSERT INTO dividends (user_id, settlement_date, product, account, security_code, \
             security_name, unit_price, shares, dividends_before_tax, taxes, net_amount_received) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
             ON CONFLICT (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax, product, account) \
             DO NOTHING RETURNING 1",
            vec![
                Bind::from(user_id),
                Bind::from(item.settlement_date),
                Bind::from(item.product.as_str()),
                Bind::from(item.account.as_str()),
                Bind::from(item.security_code.as_str()),
                Bind::from(item.security_name.as_str()),
                Bind::from(item.unit_price),
                Bind::from(item.shares),
                Bind::from(item.dividends_before_tax),
                Bind::from(item.taxes),
                Bind::from(item.net_amount_received),
            ],
        )
        .fetch_optional::<(i32,), _>(&pool)
        .await
        .expect("旧い経路の配当挿入に失敗");
        assert!(row.is_some(), "旧い経路で配当が入るはず");
    }
    let mf_items = vec![
        CreateMutualfundRequest {
            trade_date: Date::from_calendar_date(2024, time::Month::March, 20).unwrap(),
            settlement_date: Date::from_calendar_date(2024, time::Month::March, 25).unwrap(),
            fund_name: "ＭＵＦＧ　日本株オープン".to_string(),
            dividends: None,
            account: "特定".parse().unwrap(),
            shares: dec!(1000.0),
            exchange_rate: dec!(1),
            cancellation_unit_price_yen: dec!(12000.0),
            cancellation_amount_yen: dec!(12000000.0),
            average_acquisition_price_yen: dec!(10000.0),
            realized_profit_and_loss: dec!(2000000.0),
            taxes: dec!(0),
            realized_profit_and_loss_after_tax: dec!(2000000.0),
        },
        CreateMutualfundRequest {
            trade_date: Date::from_calendar_date(2024, time::Month::May, 1).unwrap(),
            settlement_date: Date::from_calendar_date(2024, time::Month::May, 8).unwrap(),
            fund_name: "架空投信D".to_string(),
            dividends: Some("再投資型".to_string()),
            account: "NISA".parse().unwrap(),
            shares: dec!(1500.5),
            exchange_rate: dec!(150.25),
            cancellation_unit_price_yen: dec!(12500.5),
            cancellation_amount_yen: dec!(18750750.25),
            average_acquisition_price_yen: dec!(10000.0),
            realized_profit_and_loss: dec!(600000.0),
            taxes: dec!(121818.0),
            realized_profit_and_loss_after_tax: dec!(478182.0),
        },
    ];
    for item in &mf_items {
        let row = db::query(
            "INSERT INTO mutualfunds (user_id, trade_date, settlement_date, fund_name, dividends, \
             account, shares, exchange_rate, cancellation_unit_price_yen, cancellation_amount_yen, \
             average_acquisition_price_yen, realized_profit_and_loss, taxes, realized_profit_and_loss_after_tax) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14) \
             ON CONFLICT (user_id, trade_date, fund_name, shares, cancellation_amount_yen, account) \
             DO NOTHING RETURNING 1",
            vec![
                Bind::from(user_id),
                Bind::from(item.trade_date),
                Bind::from(item.settlement_date),
                Bind::from(item.fund_name.as_str()),
                Bind::from(item.dividends.clone()),
                Bind::from(item.account.as_str()),
                Bind::from(item.shares),
                Bind::from(item.exchange_rate),
                Bind::from(item.cancellation_unit_price_yen),
                Bind::from(item.cancellation_amount_yen),
                Bind::from(item.average_acquisition_price_yen),
                Bind::from(item.realized_profit_and_loss),
                Bind::from(item.taxes),
                Bind::from(item.realized_profit_and_loss_after_tax),
            ],
        )
        .fetch_optional::<(i32,), _>(&pool)
        .await
        .expect("旧い経路の投信挿入に失敗");
        assert!(row.is_some(), "旧い経路で投信が入るはず");
    }

    // migration を当て直して backfill する
    run_migrations(&pool).await.expect("migrations failed");

    // backfill の中身を確認する
    let backfilled = db::query(
        "SELECT occurrence_index FROM dividends WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_all::<(i32,), _>(&pool)
    .await
    .expect("backfill の確認に失敗");
    assert_eq!(backfilled.len(), 2);
    assert!(
        backfilled.iter().all(|(index,)| *index == 0),
        "既存行は occurrence_index=0: {backfilled:?}"
    );
    let null_hash: i64 = db::query_scalar(
        "SELECT COUNT(*) FROM dividends WHERE user_id = $1 AND content_hash IS NULL",
        vec![Bind::from(user_id)],
    )
    .fetch_one(&pool)
    .await
    .expect("backfill の NULL 確認に失敗");
    assert_eq!(null_hash, 0, "backfill で content_hash が埋まるはず");

    // 同じ行を bulk_create で取り込むと 0 件挿入になる
    let div_res = dividend_svc::bulk_create(&pool, user_id, &div_items, RowLimit::new(i64::MAX))
        .await
        .expect("dividend re-upload failed");
    assert_eq!(
        (div_res.inserted, div_res.skipped),
        (0, 2),
        "backfill と式が一致すれば再取込は増えない"
    );
    let mf_res = mutualfund_svc::bulk_create(&pool, user_id, &mf_items, RowLimit::new(i64::MAX))
        .await
        .expect("mutualfund re-upload failed");
    assert_eq!(
        (mf_res.inserted, mf_res.skipped),
        (0, 2),
        "backfill と式が一致すれば再取込は増えない"
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn nullable_scalar_optional_flattens_sql_null() {
    let (pool, _node) = start_test_pool().await;
    assert_eq!(
        db::query_scalar::<i32>("SELECT NULL::integer", vec![])
            .fetch_nullable_optional(&pool)
            .await
            .expect("NULL scalar を読み取れる"),
        None
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn search_facets_group_by_domain_fields() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let mut older = make_dividend_item("9938");
    older.security_name = "旧社名".to_string();
    older.settlement_date =
        Date::from_calendar_date(2024, time::Month::try_from(3).unwrap(), 25).unwrap();
    let mut newer = make_dividend_item("9938");
    newer.security_name = "新社名".to_string();
    newer.settlement_date =
        Date::from_calendar_date(2024, time::Month::try_from(6).unwrap(), 10).unwrap();
    let mut other = make_dividend_item("9939");
    other.product = "投資信託".to_string();
    other.account = "NISA".parse().unwrap();
    other.settlement_date =
        Date::from_calendar_date(2023, time::Month::try_from(1).unwrap(), 10).unwrap();
    dividend_svc::bulk_create(
        &pool,
        user_id,
        &[older, newer, other],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("dividend bulk_create");

    let mut dividend_params = default_dividend_search_params();
    dividend_params.search.include_facets = Some(true);
    let facets = domain_search::<dividend_svc::DividendDomain>(&pool, user_id, dividend_params)
        .await
        .expect("dividend search")
        .facets
        .expect("facets 必須");
    assert_eq!(
        facets
            .products
            .expect("products")
            .iter()
            .map(|f| (f.value.as_str(), f.count))
            .collect::<Vec<_>>(),
        vec![("国内株式", Some(2)), ("投資信託", Some(1))]
    );
    assert_eq!(
        facets
            .accounts
            .expect("accounts")
            .iter()
            .map(|f| (f.value.as_str(), f.count))
            .collect::<Vec<_>>(),
        vec![("NISA", Some(1)), ("特定", Some(2))]
    );
    let securities = facets.securities.expect("securities");
    assert_eq!(securities.len(), 2);
    assert_eq!(securities[0].value, "9938");
    assert_eq!(securities[0].label, "新社名");
    assert_eq!(securities[0].count, Some(2));
    assert_eq!(securities[1].value, "9939");
    assert_eq!(securities[1].count, Some(1));
    assert_eq!(
        facets
            .years
            .expect("years")
            .iter()
            .map(|f| (f.value.as_str(), f.count))
            .collect::<Vec<_>>(),
        vec![("2024", Some(2)), ("2023", Some(1))]
    );
    assert_eq!(
        facets
            .year_months
            .expect("year_months")
            .iter()
            .map(|f| f.value.as_str())
            .collect::<Vec<_>>(),
        vec!["2024-06", "2024-03", "2023-01"]
    );

    let mut ds_older = make_domestic_stock_item("9935");
    ds_older.account = "NISA".parse().unwrap();
    ds_older.trade_date =
        Date::from_calendar_date(2023, time::Month::try_from(5).unwrap(), 15).unwrap();
    let ds_newer = make_domestic_stock_item("9936");
    domestic_stock_svc::bulk_create(
        &pool,
        user_id,
        &[ds_older, ds_newer],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("domestic_stock bulk_create");

    let mut ds_params = default_domestic_stock_search_params();
    ds_params.search.include_facets = Some(true);
    let facets =
        domain_search::<domestic_stock_svc::DomesticStockDomain>(&pool, user_id, ds_params)
            .await
            .expect("domestic_stock search")
            .facets
            .expect("facets 必須");
    assert_eq!(
        facets
            .accounts
            .expect("accounts")
            .iter()
            .map(|f| f.value.as_str())
            .collect::<Vec<_>>(),
        vec!["NISA", "特定"]
    );
    let securities = facets.securities.expect("securities");
    assert_eq!(
        securities
            .iter()
            .map(|f| f.value.as_str())
            .collect::<Vec<_>>(),
        vec!["9935", "9936"]
    );
    assert_eq!(
        facets
            .years
            .expect("years")
            .iter()
            .map(|f| f.value.as_str())
            .collect::<Vec<_>>(),
        vec!["2024", "2023"]
    );
    assert_eq!(
        facets
            .year_months
            .expect("year_months")
            .iter()
            .map(|f| f.value.as_str())
            .collect::<Vec<_>>(),
        vec!["2024-03", "2023-05"]
    );

    let mf_older = make_mutualfund_item("テスト投信A");
    let mut mf_newer = make_mutualfund_item("テスト投信B");
    mf_newer.trade_date =
        Date::from_calendar_date(2023, time::Month::try_from(8).unwrap(), 1).unwrap();
    mutualfund_svc::bulk_create(
        &pool,
        user_id,
        &[mf_older, mf_newer],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("mutualfund bulk_create");

    let mut mf_params = default_mutualfund_search_params();
    mf_params.search.include_facets = Some(true);
    let facets = domain_search::<mutualfund_svc::MutualfundDomain>(&pool, user_id, mf_params)
        .await
        .expect("mutualfund search")
        .facets
        .expect("facets 必須");
    assert!(facets.products.is_none());
    assert!(facets.securities.is_none());
    assert_eq!(
        facets
            .funds
            .expect("funds")
            .iter()
            .map(|f| f.value.as_str())
            .collect::<Vec<_>>(),
        vec!["テスト投信A", "テスト投信B"]
    );
    assert_eq!(
        facets
            .years
            .expect("years")
            .iter()
            .map(|f| f.value.as_str())
            .collect::<Vec<_>>(),
        vec!["2024", "2023"]
    );

    let asset_items = vec![make_asset_item("9931"), make_asset_item("9901")];
    asset_balance_svc::bulk_create(&pool, user_id, &asset_items, RowLimit::new(i64::MAX))
        .await
        .expect("asset_balance bulk_create");

    let mut ab_params = default_asset_balance_search_params();
    ab_params.search.include_facets = Some(true);
    let facets = domain_search::<asset_balance_svc::AssetBalanceDomain>(&pool, user_id, ab_params)
        .await
        .expect("asset_balance search")
        .facets
        .expect("facets 必須");
    let securities = facets.securities.expect("securities");
    assert_eq!(
        securities
            .iter()
            .map(|f| (f.value.as_str(), f.count))
            .collect::<Vec<_>>(),
        vec![("9901", Some(1)), ("9931", Some(1))]
    );
    assert!(facets.products.is_none());
    assert!(facets.accounts.is_none());
    assert!(facets.funds.is_none());
    assert!(facets.years.is_none());
    assert!(facets.year_months.is_none());
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn user_row_limit_rejects_over_limit_inserts() {
    use backend::services::asset_balance::AssetBalanceDomain;
    use backend::services::dividend::DividendDomain;
    use backend::services::domain::bulk::ensure_user_row_limit_with;
    use backend::services::domestic_stock::DomesticStockDomain;
    use backend::services::mutualfund::MutualfundDomain;
    const LIMIT: RowLimit = RowLimit::new(3);

    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 追記型(dividends): 既存行数 + 追加分が上限を超えると拒否
    let items = vec![make_dividend_item("9931"), make_dividend_item("9932")];
    dividend_svc::bulk_create(&pool, user_id, &items, LIMIT)
        .await
        .expect("初回 bulk_create は成功");

    // 上限ちょうど(既存2+追加1=3)は許可、超過(既存2+追加2=4)は拒否
    assert!(
        ensure_user_row_limit_with::<DividendDomain, _>(&pool, user_id, 1, LIMIT)
            .await
            .is_ok()
    );
    let err = ensure_user_row_limit_with::<DividendDomain, _>(&pool, user_id, 2, LIMIT)
        .await
        .expect_err("既存2+追加2=4 > 3 で拒否");
    assert!(
        matches!(err, backend::errors::ApiError::Validation(_)),
        "期待しないエラー: {err:?}"
    );

    let domestic_items = vec![
        make_domestic_stock_item("9940"),
        make_domestic_stock_item("9914"),
    ];
    domestic_stock_svc::bulk_create(&pool, user_id, &domestic_items, LIMIT)
        .await
        .expect("初回 bulk_create は成功");
    assert!(
        ensure_user_row_limit_with::<DomesticStockDomain, _>(&pool, user_id, 1, LIMIT)
            .await
            .is_ok()
    );
    let err = ensure_user_row_limit_with::<DomesticStockDomain, _>(&pool, user_id, 2, LIMIT)
        .await
        .expect_err("既存2+追加2=4 > 3 で拒否");
    assert!(
        matches!(err, backend::errors::ApiError::Validation(_)),
        "期待しないエラー: {err:?}"
    );

    let mutualfund_items = vec![
        make_mutualfund_item("ファンドA"),
        make_mutualfund_item("ファンドB"),
    ];
    mutualfund_svc::bulk_create(&pool, user_id, &mutualfund_items, LIMIT)
        .await
        .expect("初回 bulk_create は成功");
    assert!(
        ensure_user_row_limit_with::<MutualfundDomain, _>(&pool, user_id, 1, LIMIT)
            .await
            .is_ok()
    );
    let err = ensure_user_row_limit_with::<MutualfundDomain, _>(&pool, user_id, 2, LIMIT)
        .await
        .expect_err("既存2+追加2=4 > 3 で拒否");
    assert!(
        matches!(err, backend::errors::ApiError::Validation(_)),
        "期待しないエラー: {err:?}"
    );

    // 置換型(asset_balances): 追加分のみで上限判定(既存行数を見ない)
    let asset_items = vec![make_asset_item("9931"), make_asset_item("9901")];
    asset_balance_svc::bulk_create(&pool, user_id, &asset_items, LIMIT)
        .await
        .expect("asset_balances は置換のため上限内");
    // 既存2件あっても追加分4件 > 上限3 で拒否(既存行数を見ない)
    let err = ensure_user_row_limit_with::<AssetBalanceDomain, _>(&pool, user_id, 4, LIMIT)
        .await
        .expect_err("追加4件 > 上限3 で拒否");
    assert!(
        matches!(err, backend::errors::ApiError::Validation(_)),
        "期待しないエラー: {err:?}"
    );
    assert!(
        ensure_user_row_limit_with::<AssetBalanceDomain, _>(&pool, user_id, 3, LIMIT)
            .await
            .is_ok()
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn bulk_create_respects_user_row_limit() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let items = vec![make_dividend_item("9931")];

    // 上限値は引数で渡す(プロセス全体の環境変数を書き換えると並行テストに影響する)
    let err = dividend_svc::bulk_create(&pool, user_id, &items, RowLimit::new(0))
        .await
        .expect_err("上限0で追加1件でも拒否");
    assert!(
        matches!(err, backend::errors::ApiError::Validation(_)),
        "期待しないエラー: {err:?}"
    );
    dividend_svc::bulk_create(&pool, user_id, &items, RowLimit::new(100))
        .await
        .expect("上限内なら成功");
}
