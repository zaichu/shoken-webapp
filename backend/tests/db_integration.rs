use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header::ACCESS_CONTROL_ALLOW_ORIGIN},
    middleware,
    routing::get,
};
use backend::services::csv::import::CsvImport;
use backend::{
    config::{self, Config},
    db::{self, Bind, Db, Executor, connect_pool_lazy, run_migrations},
    middleware::validate_origin,
    models::asset_balance::{AssetBalanceSearchQueryParams, CreateAssetBalanceRequest},
    models::common::{PaginationParams, SearchQueryParams},
    models::dividend::{CreateDividendRequest, DividendSearchQueryParams},
    models::domestic_stock::{CreateDomesticStockRequest, DomesticStockSearchQueryParams},
    models::mutualfund::{CreateMutualfundRequest, MutualfundSearchQueryParams},
    models::user::GoogleUserInfo,
    services::asset_balance as asset_balance_svc,
    services::auth::{self as auth_svc, SessionToken},
    services::dividend as dividend_svc,
    services::dividend_cache,
    services::domain::bulk::{self, RowLimit},
    services::domain::search::search as domain_search,
    services::domestic_stock as domestic_stock_svc,
    services::jquants::JQuantsClient,
    services::mutualfund as mutualfund_svc,
    services::stock_price::{self, ClosePrice},
    state::{AppState, Secrets},
};

fn default_asset_balance_search_params() -> AssetBalanceSearchQueryParams {
    AssetBalanceSearchQueryParams::default()
}

fn default_dividend_search_params() -> DividendSearchQueryParams {
    DividendSearchQueryParams::default()
}

fn default_domestic_stock_search_params() -> DomesticStockSearchQueryParams {
    DomesticStockSearchQueryParams::default()
}

fn default_mutualfund_search_params() -> MutualfundSearchQueryParams {
    MutualfundSearchQueryParams::default()
}

fn dividend_search_params_with_pagination(
    pagination: PaginationParams,
) -> DividendSearchQueryParams {
    DividendSearchQueryParams {
        search: SearchQueryParams {
            pagination,
            ..Default::default()
        },
        ..Default::default()
    }
}
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use shared::dividend_per_share::DividendCacheStatus;
use shared::value::{SecurityCode, UserId};
use std::{sync::Arc, time::Duration};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tokio::time::{sleep, timeout};
use tower::ServiceExt;
use uuid::Uuid;

async fn connect_with_retry(database_url: &str) -> Db {
    let pool = connect_pool_lazy(database_url, 4).expect("Failed to build pool");
    let mut last_error = None;
    for _ in 0..20 {
        let attempt = timeout(
            Duration::from_secs(2),
            db::query("SELECT 1", vec![]).execute(&pool),
        )
        .await;
        match attempt {
            Ok(Ok(_)) => return pool,
            Ok(Err(err)) => last_error = Some(err),
            Err(_) => {}
        }
        sleep(Duration::from_millis(500)).await;
    }

    panic!("Failed to connect to Postgres: {last_error:?}");
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn db_integration_with_docker_and_migrations() {
    let node = Postgres::default().start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    db::query(
        r#"
        INSERT INTO stock
          (date, code, name, market_category, industry_code_33, industry_category_33,
           industry_code_17, industry_category_17, size_code, size_category)
        VALUES
          ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
        vec![
            Bind::from(NaiveDate::from_ymd_opt(2025, 3, 24).unwrap()),
            Bind::from("1234"),
            Bind::from("テスト株式会社"),
            Bind::from("プライム"),
            Bind::from(Some("123".to_string())),
            Bind::from(Some("情報・通信業".to_string())),
            Bind::from(Some("12".to_string())),
            Bind::from(Some("情報通信".to_string())),
            Bind::from(Some("10".to_string())),
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
        .uri("/api/v1/stocks?query=1234")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = to_bytes(resp.into_body(), 1024 * 1024).await.unwrap();
    let stock: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(stock["code"], "1234");

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

/// テスト用の CreateAssetBalanceRequest を生成するヘルパー
fn make_asset_item(code: &str) -> CreateAssetBalanceRequest {
    CreateAssetBalanceRequest {
        security_code: code.parse().unwrap(),
        security_name: format!("テスト銘柄{code}"),
        shares: dec!(100),
        executing_shares: dec!(0),
        average_purchase_price: dec!(1000),
        total_purchase_amount: dec!(100000),
        current_price: dec!(1100),
        daily_change: dec!(10),
        market_value: dec!(110000),
        profit_loss_rate: dec!(10),
    }
}

fn make_dividend_item(security_code: &str) -> CreateDividendRequest {
    CreateDividendRequest {
        settlement_date: NaiveDate::from_ymd_opt(2024, 3, 25).unwrap(),
        product: "国内株式".to_string(),
        account: "特定".parse().unwrap(),
        security_code: security_code.to_string(),
        security_name: format!("銘柄_{security_code}"),
        unit_price: dec!(100.0),
        shares: dec!(100.0),
        dividends_before_tax: dec!(1000.0),
        taxes: dec!(203.0),
        net_amount_received: dec!(797.0),
    }
}

fn make_domestic_stock_item(security_code: &str) -> CreateDomesticStockRequest {
    CreateDomesticStockRequest {
        trade_date: NaiveDate::from_ymd_opt(2024, 3, 20).unwrap(),
        settlement_date: NaiveDate::from_ymd_opt(2024, 3, 25).unwrap(),
        security_code: security_code.parse().unwrap(),
        security_name: format!("銘柄_{security_code}"),
        account: "特定".parse().unwrap(),
        shares: dec!(100.0),
        asked_price: dec!(1000.0),
        proceeds: dec!(100000.0),
        purchase_price: dec!(90000.0),
        realized_profit_and_loss: dec!(10000.0),
        taxes: dec!(2030.0),
        realized_profit_and_loss_after_tax: dec!(7970.0),
    }
}

fn make_mutualfund_item(fund_name: &str) -> CreateMutualfundRequest {
    CreateMutualfundRequest {
        trade_date: NaiveDate::from_ymd_opt(2024, 3, 20).unwrap(),
        settlement_date: NaiveDate::from_ymd_opt(2024, 3, 25).unwrap(),
        fund_name: fund_name.to_string(),
        dividends: None,
        account: "特定".parse().unwrap(),
        shares: dec!(1000.0),
        exchange_rate: dec!(1.0),
        cancellation_unit_price_yen: dec!(12000.0),
        cancellation_amount_yen: dec!(12000000.0),
        average_acquisition_price_yen: dec!(10000.0),
        realized_profit_and_loss: dec!(2000000.0),
        taxes: dec!(406060.0),
        realized_profit_and_loss_after_tax: dec!(1593940.0),
    }
}

fn make_mutualfund_csv() -> &'static str {
    concat!(
        "約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n",
        "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim 米国株式(S&P500)\",\"\",\"特定\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"615849\""
    )
}

fn make_dividend_csv() -> &'static str {
    concat!(
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]\n",
        "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8592\",\"テスト配当\",\"円\",\"93.76\",\"200\",\"18752\",\"3808\",\"14944\""
    )
}

fn make_domestic_stock_csv() -> &'static str {
    concat!(
        "約定日,受渡日,銘柄コード,銘柄名,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]\n",
        "\"2026/02/09\",\"2026/02/12\",\"5020\",\"ＥＮＥＯＳ\",\"特定\",\"-\",\"売付\",\"100\",\"1441.0\",\"144100\",\"1350.00\",\"9100\""
    )
}

fn asset_balance_csv(rows: &[&str]) -> String {
    format!(
        "■現在の評価額合計［円］,,\"1,100,000\"\n\
         ■評価損益合計,前日比［円］,\"10,000\"\n\
         ,前月比［円］,\"5,000\"\n\
         ,評価損益［円］,\"100,000\"\n\
         ■特定口座\n\
         \n\
         銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］\n\
         {}",
        rows.join("\n")
    )
}

fn make_asset_balance_csv() -> String {
    asset_balance_csv(&[
        "\"7203\",\"トヨタ自動車\",\"100\",\"-\",\"100\",\"0\",\"2500\",\"250000\",\"2650\",\"15\",\"265000\",\"6.0\"",
    ])
}

/// Docker が必要なテスト用の Postgres コンテナ起動ヘルパー
async fn start_test_pool() -> (Db, impl Drop) {
    let node = Postgres::default().start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");
    (pool, node)
}

async fn create_test_user(pool: &Db) -> UserId {
    let user_id = UserId::from(Uuid::new_v4());
    db::query(
        "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
        vec![
            Bind::from(user_id),
            Bind::from(format!("test_google_{user_id}")),
            Bind::from(format!("test_{user_id}@example.com")),
        ],
    )
    .execute(pool)
    .await
    .expect("failed to insert test user");
    user_id
}

#[tokio::test]
async fn service_coverage_all_domains() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    assert!(
        domain_search::<mutualfund_svc::MutualfundDomain>(
            &pool,
            user_id,
            default_mutualfund_search_params(),
        )
        .await
        .expect("initial mutualfund list failed")
        .data
        .is_empty()
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
    assert!(mutualfund_uploaded.errors.is_empty());

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
    assert!(
        domain_search::<mutualfund_svc::MutualfundDomain>(
            &pool,
            user_id,
            default_mutualfund_search_params(),
        )
        .await
        .expect("mutualfund list after delete failed")
        .data
        .is_empty()
    );

    assert!(
        domain_search::<dividend_svc::DividendDomain>(
            &pool,
            user_id,
            default_dividend_search_params()
        )
        .await
        .expect("initial dividend list failed")
        .data
        .is_empty()
    );
    let dividend_empty = dividend_svc::bulk_create(&pool, user_id, &[], RowLimit::new(i64::MAX))
        .await
        .expect("empty dividend bulk_create failed");
    assert_eq!(dividend_empty.inserted, 0);
    assert_eq!(dividend_empty.skipped, 0);

    let dividend_items = vec![make_dividend_item("1001"), make_dividend_item("1002")];
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
    assert!(dividend_uploaded.errors.is_empty());

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
    assert!(
        domain_search::<dividend_svc::DividendDomain>(
            &pool,
            user_id,
            default_dividend_search_params()
        )
        .await
        .expect("dividend list after delete failed")
        .data
        .is_empty()
    );

    assert!(
        domain_search::<domestic_stock_svc::DomesticStockDomain>(
            &pool,
            user_id,
            default_domestic_stock_search_params(),
        )
        .await
        .expect("initial domestic_stock list failed")
        .data
        .is_empty()
    );
    let domestic_stock_empty =
        domestic_stock_svc::bulk_create(&pool, user_id, &[], RowLimit::new(i64::MAX))
            .await
            .expect("empty domestic_stock bulk_create failed");
    assert_eq!(domestic_stock_empty.inserted, 0);
    assert_eq!(domestic_stock_empty.skipped, 0);

    let domestic_stock_items = vec![
        make_domestic_stock_item("3001"),
        make_domestic_stock_item("3002"),
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
    assert!(domestic_stock_uploaded.errors.is_empty());

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
    assert!(
        domain_search::<domestic_stock_svc::DomesticStockDomain>(
            &pool,
            user_id,
            default_domestic_stock_search_params(),
        )
        .await
        .expect("domestic_stock list after delete failed")
        .data
        .is_empty()
    );

    assert!(
        domain_search::<asset_balance_svc::AssetBalanceDomain>(
            &pool,
            user_id,
            default_asset_balance_search_params(),
        )
        .await
        .expect("initial asset_balance list failed")
        .data
        .is_empty()
    );
    let asset_balance_empty =
        asset_balance_svc::bulk_create(&pool, user_id, &[], RowLimit::new(i64::MAX))
            .await
            .expect("empty asset_balance bulk_create failed");
    assert_eq!(asset_balance_empty.inserted, 0);
    assert_eq!(asset_balance_empty.skipped, 0);

    let asset_balance_items = vec![make_asset_item("1301"), make_asset_item("1605")];
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
    assert!(asset_balance_uploaded.errors.is_empty());

    let asset_balance_rows = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("asset_balance list failed")
    .data;
    assert_eq!(asset_balance_rows.len(), 1);
    assert_eq!(asset_balance_rows[0].security_code.as_str(), "7203");

    assert_eq!(
        bulk::delete_all::<asset_balance_svc::AssetBalanceDomain>(&pool, user_id)
            .await
            .expect("asset_balance delete_all failed"),
        1
    );
    assert!(
        domain_search::<asset_balance_svc::AssetBalanceDomain>(
            &pool,
            user_id,
            default_asset_balance_search_params(),
        )
        .await
        .expect("asset_balance list after delete failed")
        .data
        .is_empty()
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn asset_balance_bulk_create_replaces_previous_snapshot() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 1回目: 2銘柄を登録
    let items_a = vec![make_asset_item("1001"), make_asset_item("1002")];
    asset_balance_svc::bulk_create(&pool, user_id, &items_a, RowLimit::new(i64::MAX))
        .await
        .expect("1回目 bulk_create 失敗");

    // 2回目: 3銘柄を登録（スナップショット置き換えなので合計3件になるはず）
    let items_b = vec![
        make_asset_item("2001"),
        make_asset_item("2002"),
        make_asset_item("2003"),
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
    assert!(codes.contains(&"2001"), "2001 が存在しない");
    assert!(codes.contains(&"2002"), "2002 が存在しない");
    assert!(codes.contains(&"2003"), "2003 が存在しない");
    assert!(!codes.contains(&"1001"), "1001 が残存している（削除漏れ）");
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_price_apply_close_price_keeps_newer_row() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    asset_balance_svc::bulk_create(
        &pool,
        user_id,
        &[make_asset_item("1301")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("bulk_create 失敗");

    let code: SecurityCode = "1301".parse().unwrap();

    // 取込直後は基準日が未確定(NULL)なので、最初の取得値がそのまま入る
    let first = ClosePrice {
        close: dec!(900),
        as_of: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        prev_close: Some(dec!(890)),
    };
    let updated = stock_price::apply_close_price(&pool, &code, &first)
        .await
        .expect("apply(first) 失敗");
    assert_eq!(updated, 1);

    // 保存済みの基準日より古い取得値は反映しない（日付の新しいほうを残す）
    let stale = ClosePrice {
        close: dec!(800),
        as_of: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
        prev_close: Some(dec!(790)),
    };
    let updated = stock_price::apply_close_price(&pool, &code, &stale)
        .await
        .expect("apply(stale) 失敗");
    assert_eq!(updated, 0);

    // 新しい取得値が current_price/market_value/daily_change に反映されることを確認
    let fresh = ClosePrice {
        close: dec!(2500.5),
        as_of: NaiveDate::from_ymd_opt(2020, 1, 2).unwrap(),
        prev_close: Some(dec!(2400)),
    };
    let updated = stock_price::apply_close_price(&pool, &code, &fresh)
        .await
        .expect("apply(fresh) 失敗");
    assert_eq!(updated, 1);

    let row = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("list 失敗")
    .data
    .into_iter()
    .next()
    .expect("1件のはず");
    assert_eq!(row.current_price, dec!(2500.5));
    // daily_change = close - prev_close、一覧には共有テーブルの値が出る
    assert_eq!(row.daily_change, dec!(100.5));
    assert_eq!(row.price_as_of, NaiveDate::from_ymd_opt(2020, 1, 2));
}

/// 同一銘柄を持つ2人の利用者で、確定値の反映は共有テーブル1行だけを更新し、
/// 両方の一覧に同じ価格が出る
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn stock_price_applies_once_to_shared_row_and_visible_to_all_holders() {
    let (pool, _node) = start_test_pool().await;
    let user_a = create_test_user(&pool).await;
    let user_b = create_test_user(&pool).await;

    let mut shares_b = make_asset_item("7203");
    shares_b.shares = dec!(200);
    for (user_id, items) in [
        (user_a, vec![make_asset_item("7203")]),
        (user_b, vec![shares_b]),
    ] {
        asset_balance_svc::bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
            .await
            .expect("bulk_create 失敗");
    }

    // CSV 取込時点の共有行は未確認(基準日なし)で、両者の CSV 値が入っている
    let seeded: Option<(db::Numeric, Option<NaiveDate>)> = db::query_as(
        "SELECT current_price, price_as_of FROM security_prices WHERE security_code = '7203'",
        vec![],
    )
    .fetch_optional(&pool)
    .await
    .expect("共有価格行の取得");
    let (seeded_price, seeded_as_of) = seeded.expect("取込直後は共有行がある");
    assert_eq!(seeded_price.0, dec!(1100));
    assert_eq!(seeded_as_of, None, "CSV 取込値は未確認として入る");

    let code: SecurityCode = "7203".parse().unwrap();
    let price = ClosePrice {
        close: dec!(2700),
        as_of: NaiveDate::from_ymd_opt(2026, 10, 14).unwrap(),
        prev_close: Some(dec!(2650)),
    };
    let updated = stock_price::apply_close_price(&pool, &code, &price)
        .await
        .expect("apply_close_price 失敗");
    assert_eq!(updated, 1, "共有テーブルでは1銘柄1行だけ更新される");

    for (user_id, shares) in [(user_a, dec!(100)), (user_b, dec!(200))] {
        let mut params = default_asset_balance_search_params();
        params.search.include_summary = Some(true);
        let res = domain_search::<asset_balance_svc::AssetBalanceDomain>(&pool, user_id, params)
            .await
            .expect("list 失敗");
        let row = res.data.into_iter().next().expect("1件のはず");
        assert_eq!(row.current_price, dec!(2700));
        assert_eq!(row.daily_change, dec!(50));
        assert_eq!(row.price_as_of, NaiveDate::from_ymd_opt(2026, 10, 14));

        // 前日比の合計・評価額は 共有価格×その利用者の数量 で計算される
        let summary = res.summary.expect("summary 必須");
        assert_eq!(summary.total_daily_change, dec!(50) * shares);
        assert_eq!(summary.total_market_value, dec!(2700) * shares);
    }
}

/// 確定値が入った銘柄は CSV 再取込で上書きされない。
/// 未確認(price_as_of NULL)の共有行だけが CSV の値に更新される
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn asset_balance_csv_reimport_keeps_confirmed_price() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    asset_balance_svc::bulk_create(
        &pool,
        user_id,
        &[make_asset_item("7203")],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("初回 bulk_create 失敗");

    let code: SecurityCode = "7203".parse().unwrap();
    let confirmed = ClosePrice {
        close: dec!(2700),
        as_of: NaiveDate::from_ymd_opt(2026, 10, 14).unwrap(),
        prev_close: Some(dec!(2650)),
    };
    stock_price::apply_close_price(&pool, &code, &confirmed)
        .await
        .expect("確定値の適用");

    // 別価格を載せた再取込でも確定値は残り、確定値のない銘柄だけ CSV 値が入る
    let mut reimported = make_asset_item("7203");
    reimported.current_price = dec!(9999);
    reimported.daily_change = dec!(-999);
    let mut fresh = make_asset_item("6758");
    fresh.current_price = dec!(1234.5);
    asset_balance_svc::bulk_create(
        &pool,
        user_id,
        &[reimported, fresh],
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("再取込");

    let rows = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("list 失敗")
    .data;
    assert_eq!(rows.len(), 2);
    let by_code = |code: &str| {
        rows.iter()
            .find(|r| r.security_code.as_str() == code)
            .expect("行がある")
    };
    let confirmed_row = by_code("7203");
    assert_eq!(confirmed_row.current_price, dec!(2700), "確定値は消えない");
    assert_eq!(confirmed_row.daily_change, dec!(50));
    assert_eq!(
        confirmed_row.price_as_of,
        NaiveDate::from_ymd_opt(2026, 10, 14)
    );
    let seeded_row = by_code("6758");
    assert_eq!(seeded_row.current_price, dec!(1234.5));
    assert_eq!(seeded_row.price_as_of, None, "CSV 取込値は未確認");

    // 未確認の共有行は次の CSV 取込(別利用者でも)の値で更新される
    let other_user = create_test_user(&pool).await;
    let mut other_import = make_asset_item("6758");
    other_import.current_price = dec!(2000);
    asset_balance_svc::bulk_create(&pool, other_user, &[other_import], RowLimit::new(i64::MAX))
        .await
        .expect("別利用者の取込");

    let rows = domain_search::<asset_balance_svc::AssetBalanceDomain>(
        &pool,
        user_id,
        default_asset_balance_search_params(),
    )
    .await
    .expect("list 失敗")
    .data;
    let updated_row = rows
        .iter()
        .find(|r| r.security_code.as_str() == "6758")
        .expect("行がある");
    assert_eq!(updated_row.current_price, dec!(2000));
    assert_eq!(updated_row.price_as_of, None);
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
        make_dividend_item("1001"),
        make_dividend_item("1002"),
        make_dividend_item("1003"),
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
    let items = vec![make_dividend_item("2001")];

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
async fn dividend_csv_keeps_same_detail_in_different_accounts() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 同一入金日・同一銘柄・同一数量・同一額で、口座または商品だけ違う明細は別データ
    let csv = concat!(
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]\n",
        "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8591\",\"オリックス\",\"円\",\"93.76\",\"200\",\"18752\",\"3808\",\"14944\"\n",
        "\"2025/12/09\",\"国内株式\",\"NISA\",\"8591\",\"オリックス\",\"円\",\"93.76\",\"200\",\"18752\",\"3808\",\"14944\"\n",
        "\"2025/12/09\",\"投資信託\",\"特定・一般\",\"8591\",\"オリックス\",\"円\",\"93.76\",\"200\",\"18752\",\"3808\",\"14944\""
    );

    let first = dividend_svc::DividendDomain::upload_csv(
        &pool,
        user_id,
        csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("dividend upload_csv failed");
    assert_eq!(first.inserted, 3);
    assert_eq!(first.skipped, 0);
    assert!(first.errors.is_empty());

    // 同じ CSV の再取込では増えない
    let second = dividend_svc::DividendDomain::upload_csv(
        &pool,
        user_id,
        csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("dividend re-upload_csv failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 3);
    assert!(second.errors.is_empty());
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn domestic_stock_bulk_create_and_list() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let items = vec![
        make_domestic_stock_item("3001"),
        make_domestic_stock_item("3002"),
        make_domestic_stock_item("3003"),
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
    let items = vec![make_domestic_stock_item("4001")];

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
async fn mutualfund_csv_keeps_same_trade_in_different_accounts() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    // 同一約定日・同一ファンド・同一数量・同一解約額で、口座だけ違う明細は別データ
    let csv = concat!(
        "約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n",
        "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim 米国株式(S&P500)\",\"再投資型\",\"特定\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"615849\"\n",
        "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim 米国株式(S&P500)\",\"再投資型\",\"NISA\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"615849\""
    );

    let first = mutualfund_svc::MutualfundDomain::upload_csv(
        &pool,
        user_id,
        csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("mutualfund upload_csv failed");
    assert_eq!(first.inserted, 2);
    assert_eq!(first.skipped, 0);
    assert!(first.errors.is_empty());

    // 同じ CSV の再取込では増えない
    let second = mutualfund_svc::MutualfundDomain::upload_csv(
        &pool,
        user_id,
        csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("mutualfund re-upload_csv failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 2);
    assert!(second.errors.is_empty());
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn auth_session_upsert_rotate_and_delete() {
    let (pool, _node) = start_test_pool().await;

    let info = GoogleUserInfo {
        sub: "google-sub-0001".to_string(),
        email: "first@example.com".to_string(),
        name: Some("ユーザー".to_string()),
        picture: Some("https://example.com/p.png".to_string()),
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

/// id にもトークンを書く旧版と同時に動いても、双方が発行したセッションを互いに照合・失効できること。
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn session_token_hash_rolling_deploy_compat() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let legacy: SessionToken = Uuid::new_v4().to_string().parse().unwrap();
    db::query(
        "INSERT INTO sessions (user_id, token_hash) VALUES ($2, sha256(convert_to($1::text, 'UTF8')))",
        vec![Bind::from(legacy.to_string()), Bind::from(user_id)],
    )
    .execute(&pool)
    .await
    .expect("移行済みセッションの挿入");
    assert_eq!(
        auth_svc::select_user_id_by_session(&pool, legacy)
            .await
            .expect("移行済みセッションの照合"),
        Some(user_id),
        "移行後も旧 Cookie が有効であること"
    );

    // 旧版は id にもトークンを書く
    let old_version_issued = Uuid::new_v4();
    let old_version_issued_token: SessionToken = old_version_issued.to_string().parse().unwrap();
    db::query(
        "INSERT INTO sessions (id, user_id, token_hash) VALUES ($1, $2, sha256(convert_to($1::uuid::text, 'UTF8')))",
        vec![Bind::from(old_version_issued), Bind::from(user_id)],
    )
    .execute(&pool)
    .await
    .expect("旧版発行セッションの挿入");
    assert_eq!(
        auth_svc::select_user_id_by_session(&pool, old_version_issued_token)
            .await
            .expect("旧版発行セッションの照合"),
        Some(user_id),
        "旧版発行セッションが有効であること"
    );

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
        picture: None,
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

    // 同一ユーザーの旧セッションは rotate で失効する
    assert!(
        auth_svc::select_user_id_by_session(&pool, legacy)
            .await
            .expect("rotate 後の旧セッション照合")
            .is_none(),
        "rotate で移行済みの旧セッションが失効すること"
    );
    assert!(
        auth_svc::select_user_id_by_session(&pool, old_version_issued_token)
            .await
            .expect("rotate 後の旧版発行セッション照合")
            .is_none(),
        "rotate で旧版発行セッションも失効すること"
    );
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

    // 新版は id にトークンを書かず、旧版の照合クエリでも解決できる
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
    let old_version_view: Option<(UserId,)> = db::query_as(
        "SELECT user_id FROM sessions WHERE (token_hash = sha256(convert_to($1::text, 'UTF8')) OR (token_hash IS NULL AND id = $1::text::uuid)) AND expires_at > NOW()",
        vec![Bind::from(new_session.to_string())],
    )
    .fetch_optional(&pool)
    .await
    .expect("旧版の照合クエリ");
    assert_eq!(
        old_version_view.map(|r| r.0),
        Some(user_id),
        "旧版が新版発行セッションを照合できること"
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
        picture: None,
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
async fn search_facets_group_by_domain_fields() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let mut older = make_dividend_item("4001");
    older.security_name = "旧社名".to_string();
    older.settlement_date = NaiveDate::from_ymd_opt(2024, 3, 25).unwrap();
    let mut newer = make_dividend_item("4001");
    newer.security_name = "新社名".to_string();
    newer.settlement_date = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
    let mut other = make_dividend_item("4002");
    other.product = "投資信託".to_string();
    other.account = "NISA".parse().unwrap();
    other.settlement_date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
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
    assert_eq!(securities[0].value, "4001");
    assert_eq!(securities[0].label, "新社名");
    assert_eq!(securities[0].count, Some(2));
    assert_eq!(securities[1].value, "4002");
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

    let mut ds_older = make_domestic_stock_item("3001");
    ds_older.account = "NISA".parse().unwrap();
    ds_older.trade_date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
    let ds_newer = make_domestic_stock_item("3002");
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
        vec!["3001", "3002"]
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
    mf_newer.trade_date = NaiveDate::from_ymd_opt(2023, 8, 1).unwrap();
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

    let asset_items = vec![make_asset_item("1301"), make_asset_item("1605")];
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
        vec![("1301", Some(1)), ("1605", Some(1))]
    );
    assert!(facets.products.is_none());
    assert!(facets.accounts.is_none());
    assert!(facets.funds.is_none());
    assert!(facets.years.is_none());
    assert!(facets.year_months.is_none());
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_cache_persistence_and_rate_slot() {
    let (pool, _node) = start_test_pool().await;

    assert!(
        dividend_cache::get_batch(&pool, None, &[])
            .await
            .unwrap()
            .is_empty()
    );

    dividend_cache::persistence::update_cache_error(
        &pool,
        &"1234".parse::<SecurityCode>().unwrap(),
        "fetch failed",
        0,
    )
    .await
    .expect("エラー記録");
    let items = dividend_cache::get_batch(&pool, None, &["1234".parse().unwrap()])
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].security_code.as_str(), "1234");
    assert_eq!(items[0].status, DividendCacheStatus::Error);
    assert_eq!(items[0].dividend_per_share, None);
    assert!(
        items[0].is_stale,
        "再試行間隔を経過した error 記録は再取得対象"
    );

    dividend_cache::persistence::update_cache_error(
        &pool,
        &"5678".parse::<SecurityCode>().unwrap(),
        "429",
        3600,
    )
    .await
    .expect("cooldown 付きエラー記録");
    let items = dividend_cache::get_batch(&pool, None, &["5678".parse().unwrap()])
        .await
        .unwrap();
    assert_eq!(items[0].status, DividendCacheStatus::Error);
    assert!(!items[0].is_stale, "cooldown 中は再取得対象外");

    // 既存行への upsert: stale_at が cooldown で未来に更新される
    dividend_cache::persistence::update_cache_error(
        &pool,
        &"1234".parse::<SecurityCode>().unwrap(),
        "429",
        3600,
    )
    .await
    .expect("既存行の上書き");
    let items = dividend_cache::get_batch(&pool, None, &["1234".parse().unwrap()])
        .await
        .unwrap();
    assert!(!items[0].is_stale);
    let row: (String,) = db::query_as(
        "SELECT error_message FROM dividend_per_share_cache WHERE security_code = '1234'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "429");

    dividend_cache::acquire_rate_slot(&pool)
        .await
        .expect("スロット予約");
    let row: (chrono::DateTime<Utc>,) = db::query_as(
        "SELECT next_available_at FROM market_data_provider_rate_control \
         WHERE provider = 'jquants'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        row.0 > Utc::now(),
        "予約後の next_available_at は未来: {:?}",
        row.0
    );
}

/// drain_refresh_queue の消化対象選定(pending / 再試行間隔経過済みの error は対象、
/// 間隔内(未来の stale_at)は対象外)と、取得失敗時の error 記録を検証する。
/// J-Quants への実取得経路は Worker 側にしかないため、ホスト側の取得は必ず失敗し、
/// 選定済み行が error として記録されることまでを確認する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn dividend_cache_drain_refresh_queue() {
    let (pool, _node) = start_test_pool().await;
    let client = JQuantsClient::new("test-api-key".to_string());

    // pending 行(get_batch のエンキュー相当)と即時再取得対象の error 行を積む
    for code in ["7203", "8306"] {
        db::query(
            "INSERT INTO dividend_per_share_cache (security_code, status, provider, updated_at)
             VALUES ($1, 'pending', 'jquants', NOW())",
            vec![Bind::from(code)],
        )
        .execute(&pool)
        .await
        .expect("pending 行の投入");
    }
    // 8306 は再試行間隔 0 のエラー行に戻して「stale_at 経過済み = 即時再取得対象」の経路を確認
    dividend_cache::persistence::update_cache_error(
        &pool,
        &"8306".parse::<SecurityCode>().unwrap(),
        "previous fetch failed",
        0,
    )
    .await
    .expect("error 行の更新");
    // 429 cooldown 中の error 行は消化対象外
    dividend_cache::persistence::update_cache_error(
        &pool,
        &"9107".parse::<SecurityCode>().unwrap(),
        "429",
        3600,
    )
    .await
    .expect("cooldown 行の投入");

    let processed = dividend_cache::drain_refresh_queue(&pool, &client)
        .await
        .expect("drain 失敗");
    assert_eq!(processed, 2, "cooldown 中の銘柄を除く 2 件を消化する");

    // 選定された行は fetch 失敗により error として記録される
    for code in ["7203", "8306"] {
        let row: (String,) = db::query_as(
            "SELECT status FROM dividend_per_share_cache WHERE security_code = $1",
            vec![Bind::from(code)],
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, "error", "{code} は失敗記録される");
    }

    // cooldown 中の行は消化されず、記録内容も変わらない
    let row: (String, Option<String>) = db::query_as(
        "SELECT status, error_message FROM dividend_per_share_cache
         WHERE security_code = '9107'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "error");
    assert_eq!(row.1.as_deref(), Some("429"));

    // 失敗した行は再試行間隔分だけ stale_at が未来になり、即時には再消化されない
    let row: (chrono::DateTime<Utc>,) = db::query_as(
        "SELECT stale_at FROM dividend_per_share_cache WHERE security_code = '7203'",
        vec![],
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        row.0 > Utc::now() + chrono::Duration::hours(23),
        "失敗行の stale_at は再試行間隔の先: {:?}",
        row.0
    );
    let processed = dividend_cache::drain_refresh_queue(&pool, &client)
        .await
        .expect("2回目の drain 失敗");
    assert_eq!(processed, 0, "再試行間隔が経過するまで消化しない");
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn user_row_limit_rejects_over_limit_inserts() {
    use backend::services::asset_balance::AssetBalanceDomain;
    use backend::services::dividend::DividendDomain;
    use backend::services::domain::bulk::ensure_user_row_limit_with;
    use backend::services::domestic_stock::DomesticStockDomain;
    use backend::services::mutualfund::MutualfundDomain;
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    const LIMIT: RowLimit = RowLimit::new(3);

    // 追記型(dividends): 既存行数 + 追加分が上限を超えると拒否
    let items = vec![make_dividend_item("1001"), make_dividend_item("1002")];
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
        make_domestic_stock_item("6501"),
        make_domestic_stock_item("6758"),
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
    let asset_items = vec![make_asset_item("1301"), make_asset_item("1605")];
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
    let items = vec![make_dividend_item("1001")];

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

/// migrate.rs が作る _sqlx_migrations と同じ定義。履歴の置き換えを確かめるため
/// ランナーを介さず手動で履歴を作るのに使う
const SQLX_MIGRATIONS_DDL: &str = "CREATE TABLE _sqlx_migrations (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL,
    checksum BYTEA NOT NULL,
    execution_time BIGINT NOT NULL
)";

/// 旧ランナーで version 1〜=max_version まで適用済みの状態を再現する
async fn seed_migration_history(pool: &Db, max_version: i64) {
    pool.batch_execute(SQLX_MIGRATIONS_DDL)
        .await
        .expect("履歴テーブルの作成に失敗");
    for version in 1..=max_version {
        db::query(
            "INSERT INTO _sqlx_migrations
                (version, description, success, checksum, execution_time)
            VALUES ($1, 'legacy', true, '\\x00'::bytea, 0)",
            vec![Bind::from(version)],
        )
        .execute(pool)
        .await
        .expect("履歴行の作成に失敗");
    }
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn partial_legacy_history_stops_migration() {
    let node = Postgres::default().start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let pool = connect_with_retry(&database_url).await;
    seed_migration_history(&pool, 10).await;

    let err = run_migrations(&pool)
        .await
        .expect_err("旧履歴が途中までの DB はエラーで止まるはず");
    assert!(err.contains("揃っていません"), "unexpected error: {err}");
}

async fn insert_price_row(
    pool: &Db,
    user_id: UserId,
    code: &str,
    current_price: Decimal,
    daily_change: Decimal,
    price_as_of: Option<NaiveDate>,
) {
    db::query(
        r#"
        INSERT INTO asset_balances
            (user_id, security_code, security_name, shares,
             average_purchase_price, total_purchase_amount,
             current_price, daily_change, price_as_of)
        VALUES ($1, $2, $3, 100, 1000, 100000, $4, $5, $6)
        "#,
        vec![
            Bind::from(user_id),
            Bind::from(code),
            Bind::from(format!("テスト銘柄{code}")),
            Bind::from(current_price),
            Bind::from(daily_change),
            Bind::opt(price_as_of),
        ],
    )
    .execute(pool)
    .await
    .expect("asset_balances への投入に失敗");

    // 保有行を直接 INSERT するフィクスチャでも、バックフィル後の状態に合わせて
    // 共有テーブルへ同じ価格を入れておく
    db::query(
        r#"
        INSERT INTO security_prices
            (security_code, current_price, daily_change, price_as_of)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (security_code) DO UPDATE SET
            current_price = EXCLUDED.current_price,
            daily_change  = EXCLUDED.daily_change,
            price_as_of   = EXCLUDED.price_as_of
        "#,
        vec![
            Bind::from(code),
            Bind::from(current_price),
            Bind::from(daily_change),
            Bind::opt(price_as_of),
        ],
    )
    .execute(pool)
    .await
    .expect("security_prices への投入に失敗");
}

/// 丸め導入前に保存された残差つきの価格を持つ行だけ price_as_of が NULL に戻り、
/// 次の cron 取得で取り直される対象になることを確かめる。
/// start_test_pool は空テーブルに全マイグレーションを適用するため、
/// データ投入後にマイグレーションファイルの UPDATE を直接流して検証する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn residual_price_rows_get_price_as_of_cleared() {
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    let as_of = NaiveDate::from_ymd_opt(2026, 10, 14);

    // 現在値・前日比のどちらかが小数第2位で割り切れない行は取り直し対象
    insert_price_row(
        &pool,
        user_id,
        "9831",
        dec!(716.5999755859375),
        dec!(16.5999755859375),
        as_of,
    )
    .await;
    insert_price_row(
        &pool,
        user_id,
        "9501",
        dec!(517.6),
        dec!(2.5999755859375),
        as_of,
    )
    .await;
    // 正常な行は基準日を維持する
    insert_price_row(&pool, user_id, "7203", dec!(2650), dec!(15), as_of).await;
    insert_price_row(&pool, user_id, "6758", dec!(2500.5), dec!(100.5), as_of).await;
    // すでに未確定の行はそのまま
    insert_price_row(
        &pool,
        user_id,
        "1301",
        dec!(716.5999755859375),
        dec!(0),
        None,
    )
    .await;

    pool.batch_execute(include_str!(
        "../migrations/0010_clear_price_as_of_on_residual_prices.sql"
    ))
    .await
    .expect("マイグレーションの実行に失敗");

    let rows = db::query(
        "SELECT security_code, price_as_of FROM asset_balances \
         WHERE user_id = $1 ORDER BY security_code",
        vec![Bind::from(user_id)],
    )
    .fetch_all::<(String, Option<NaiveDate>), _>(&pool)
    .await
    .expect("price_as_of の取得に失敗");
    assert_eq!(
        rows,
        vec![
            ("1301".to_string(), None),
            ("6758".to_string(), as_of),
            ("7203".to_string(), as_of),
            ("9501".to_string(), None),
            ("9831".to_string(), None),
        ]
    );

    // 価格の値自体は更新しない(price_as_of だけを NULL に戻す)
    let (residue_count,): (i64,) = db::query(
        "SELECT COUNT(*) FROM asset_balances \
         WHERE user_id = $1 AND current_price = 716.5999755859375::numeric",
        vec![Bind::from(user_id)],
    )
    .fetch_one::<(i64,), _>(&pool)
    .await
    .expect("集計に失敗");
    assert_eq!(residue_count, 2);
}

/// (銘柄コード, 数量) の一覧。置換ガードの前後比較用
async fn asset_balance_rows(pool: &Db, user_id: UserId) -> Vec<(String, String)> {
    db::query(
        "SELECT security_code, shares::text FROM asset_balances \
         WHERE user_id = $1 ORDER BY security_code",
        vec![Bind::from(user_id)],
    )
    .fetch_all::<(String, String), _>(pool)
    .await
    .expect("asset_balances の取得に失敗")
}

/// (銘柄コード, 価格内容) の一覧。共有価格が巻き込まれていないかの比較用
async fn security_price_rows(pool: &Db) -> Vec<(String, String)> {
    db::query(
        "SELECT security_code, current_price::text || '/' || daily_change::text \
         || '/' || COALESCE(price_as_of::text, '-') \
         FROM security_prices ORDER BY security_code",
        vec![],
    )
    .fetch_all::<(String, String), _>(pool)
    .await
    .expect("security_prices の取得に失敗")
}

/// 置換型の資産CSVは、行エラー・有効行0件のとき保存前に拒否され、
/// 既存残高と共有価格が変わらないことを確かめる(#1336)。
/// 追記型CSVの部分成功は従来通り維持する
#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn asset_balance_csv_upload_rejects_invalid_or_empty() {
    use backend::services::asset_balance::AssetBalanceDomain;
    use backend::services::dividend::DividendDomain;

    const VALID_ROW_1301: &str = "\"1301\",\"極洋\",\"100\",\"0\",\"100\",\"0\",\"500\",\"50000\",\"600\",\"5\",\"60000\",\"20.0\"";
    const VALID_ROW_1605: &str = "\"1605\",\"ＩＮＰＥＸ\",\"200\",\"0\",\"200\",\"0\",\"2355\",\"471000\",\"3685\",\"65\",\"737000\",\"56.47\"";
    const BAD_SHARES_ROW: &str = "\"7974\",\"任天堂\",\"N/A\",\"0\",\"1,000\",\"0\",\"5,997.60\",\"5,997,600\",\"8,737.0\",\"223.0\",\"8,737,000\",\"45.67\"";
    const BAD_PRICE_ROW: &str = "\"1605\",\"ＩＮＰＥＸ\",\"200\",\"0\",\"200\",\"0\",\"2,355.00\",\"471,000\",\"不明\",\"65.0\",\"737,000\",\"56.47\"";
    const SUMMARY_ONLY_ROW: &str = ",,,,,,特定口座合計,\"11,245,249\",,,\"14,517,240\",\"29.09\"";

    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;

    let seed = vec![make_asset_item("7203"), make_asset_item("7974")];
    asset_balance_svc::bulk_create(&pool, user_id, &seed, RowLimit::new(i64::MAX))
        .await
        .expect("seed bulk_create failed");

    // 正常CSVは従来通り全置換される
    let ok = AssetBalanceDomain::upload_csv(
        &pool,
        user_id,
        asset_balance_csv(&[VALID_ROW_1301, VALID_ROW_1605]).as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("正常CSVは取り込める");
    assert_eq!((ok.inserted, ok.errors.is_empty()), (2, true));
    assert_eq!(
        asset_balance_rows(&pool, user_id).await,
        vec![
            ("1301".to_string(), "100".to_string()),
            ("1605".to_string(), "200".to_string()),
        ]
    );

    // 拒否ケースの比較基準になる状態へ戻す
    asset_balance_svc::bulk_create(&pool, user_id, &seed, RowLimit::new(i64::MAX))
        .await
        .expect("re-seed bulk_create failed");
    let balances_before = asset_balance_rows(&pool, user_id).await;
    let prices_before = security_price_rows(&pool).await;

    for (label, csv) in [
        (
            "1行不正",
            asset_balance_csv(&[VALID_ROW_1301, BAD_SHARES_ROW]),
        ),
        (
            "全行不正",
            asset_balance_csv(&[BAD_SHARES_ROW, BAD_PRICE_ROW]),
        ),
        ("有効行0件", asset_balance_csv(&[SUMMARY_ONLY_ROW])),
        ("データ行なし", asset_balance_csv(&[])),
    ] {
        let err =
            AssetBalanceDomain::upload_csv(&pool, user_id, csv.as_bytes(), RowLimit::new(i64::MAX))
                .await
                .expect_err(label);
        assert!(
            matches!(err, backend::errors::ApiError::Validation(_)),
            "{label}: {err:?}"
        );
        assert_eq!(
            asset_balance_rows(&pool, user_id).await,
            balances_before,
            "{label}"
        );
        assert_eq!(security_price_rows(&pool).await, prices_before, "{label}");
    }

    // 追記型(配当)は行エラーがあっても有効行だけ取り込む従来仕様を維持する
    let dividend_csv = concat!(
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]\n",
        "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8592\",\"テスト配当\",\"円\",\"93.76\",\"200\",\"18752\",\"3808\",\"14944\"\n",
        "\"N/A\",\"国内株式\",\"特定\",\"8593\",\"不正行\",\"円\",\"1\",\"1\",\"1\",\"1\",\"1\""
    );
    let res = DividendDomain::upload_csv(
        &pool,
        user_id,
        dividend_csv.as_bytes(),
        RowLimit::new(i64::MAX),
    )
    .await
    .expect("追記型CSVは部分成功する");
    assert_eq!((res.inserted, res.errors.len()), (1, 1));

    let (dividend_count,): (i64,) = db::query(
        "SELECT COUNT(*) FROM dividends WHERE user_id = $1",
        vec![Bind::from(user_id)],
    )
    .fetch_one::<(i64,), _>(&pool)
    .await
    .expect("dividends の件数取得に失敗");
    assert_eq!(dividend_count, 1);
}
