use axum::{
    body::{to_bytes, Body},
    http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, Method, Request, StatusCode},
};
use backend::{
    config::Config,
    db::run_migrations,
    models::asset_balance::CreateAssetBalanceRequest,
    models::common::{PaginationParams, SearchQueryParams},
    models::dividend::{CreateDividendRequest, DividendSearchQueryParams},
    models::domestic_stock::{CreateDomesticStockRequest, DomesticStockSearchQueryParams},
    models::mutualfund::{CreateMutualfundRequest, MutualfundSearchQueryParams},
    routes::app_router,
    services::asset_balance as asset_balance_svc,
    services::dividend as dividend_svc,
    services::domestic_stock as domestic_stock_svc,
    services::mutualfund as mutualfund_svc,
    state::{AppState, Secrets},
};

fn default_pagination() -> PaginationParams {
    PaginationParams {
        page: None,
        per_page: None,
    }
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
use chrono::NaiveDate;
use reqwest::Client;
use rust_decimal_macros::dec;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, sync::Arc, time::Duration};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;
use tokio::time::{sleep, timeout};
use tower::ServiceExt;
use uuid::Uuid;

struct EnvGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvGuard {
    fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = env::var(key).ok();
        match value {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.previous {
            Some(value) => env::set_var(self.key, value),
            None => env::remove_var(self.key),
        }
    }
}

async fn connect_with_retry(database_url: &str) -> PgPool {
    let mut last_error = None;
    for _ in 0..20 {
        let attempt = timeout(
            Duration::from_secs(2),
            PgPoolOptions::new().connect(database_url),
        )
        .await;
        match attempt {
            Ok(Ok(pool)) => return pool,
            Ok(Err(err)) => last_error = Some(err),
            Err(_) => {}
        }
        sleep(Duration::from_millis(500)).await;
    }

    panic!("Failed to connect to Postgres: {:?}", last_error);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn db_integration_with_docker_and_migrations() {
    let _app_env = EnvGuard::set("APP_ENV", Some("production"));
    let _cors_origins = EnvGuard::set("CORS_ORIGINS", Some("https://shoken-webapp.vercel.app"));

    let node = Postgres::default().start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");

    sqlx::query(
        r#"
        INSERT INTO stock
          (date, code, name, market_category, industry_code_33, industry_category_33,
           industry_code_17, industry_category_17, size_code, size_category)
        VALUES
          ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(NaiveDate::from_ymd_opt(2025, 3, 24).unwrap())
    .bind("1234")
    .bind("テスト株式会社")
    .bind("プライム")
    .bind(Option::<String>::Some("123".to_string()))
    .bind(Option::<String>::Some("情報・通信業".to_string()))
    .bind(Option::<String>::Some("12".to_string()))
    .bind(Option::<String>::Some("情報通信".to_string()))
    .bind(Option::<String>::Some("10".to_string()))
    .bind(Option::<String>::Some("大型株".to_string()))
    .execute(&pool)
    .await
    .expect("Failed to insert test stock");

    let secrets = Arc::new(Secrets {
        database_url: database_url.clone(),
        jquants_api_key: None,
        google_client_id: None,
        google_client_secret: None,
        frontend_url: "http://localhost:8080".to_string(),
    });
    let client = Client::new();
    let state = AppState {
        pool,
        secrets,
        client,
        dividend_cache: backend::state::DividendCacheState::default(),
    };

    let config = Config::from_env();
    let app = app_router(
        state,
        &config,
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
    );

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
        .header("origin", "https://shoken-webapp.vercel.app")
        .header("access-control-request-method", "GET")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let allowed_origin = resp
        .headers()
        .get(ACCESS_CONTROL_ALLOW_ORIGIN)
        .and_then(|value| value.to_str().ok());
    assert_eq!(allowed_origin, Some("https://shoken-webapp.vercel.app"));
}

/// テスト用の CreateAssetBalanceRequest を生成するヘルパー
fn make_asset_item(code: &str) -> CreateAssetBalanceRequest {
    CreateAssetBalanceRequest {
        security_code: code.to_string(),
        security_name: format!("テスト銘柄{}", code),
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
        account: "特定".to_string(),
        security_code: security_code.to_string(),
        security_name: format!("銘柄_{}", security_code),
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
        security_code: security_code.to_string(),
        security_name: format!("銘柄_{}", security_code),
        account: "特定".to_string(),
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
        account: "特定".to_string(),
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

fn make_asset_balance_csv() -> &'static str {
    concat!(
        "■現在の評価額合計［円］,,\"1,100,000\"\n",
        "■評価損益合計,前日比［円］,\"10,000\"\n",
        ",前月比［円］,\"5,000\"\n",
        ",評価損益［円］,\"100,000\"\n",
        "■特定口座\n",
        "\n",
        "銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］\n",
        "\"7203\",\"トヨタ自動車\",\"100\",\"-\",\"100\",\"0\",\"2500\",\"250000\",\"2650\",\"15\",\"265000\",\"6.0\""
    )
}

/// Docker が必要なテスト用の Postgres コンテナ起動ヘルパー
async fn start_test_pool() -> (PgPool, impl Drop) {
    let node = Postgres::default().start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);
    let pool = connect_with_retry(&database_url).await;
    run_migrations(&pool).await.expect("migrations failed");
    (pool, node)
}

async fn create_test_user(pool: &PgPool) -> Uuid {
    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind(format!("test_google_{user_id}"))
        .bind(format!("test_{user_id}@example.com"))
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
        mutualfund_svc::search(&pool, user_id, &default_mutualfund_search_params())
            .await
            .expect("initial mutualfund list failed")
            .data
            .is_empty()
    );
    let mutualfund_empty = mutualfund_svc::bulk_create(&pool, user_id, &[])
        .await
        .expect("empty mutualfund bulk_create failed");
    assert_eq!(mutualfund_empty.inserted, 0);
    assert_eq!(mutualfund_empty.skipped, 0);

    let mutualfund_items = vec![
        make_mutualfund_item("テスト投信A"),
        make_mutualfund_item("テスト投信B"),
    ];
    let mutualfund_created = mutualfund_svc::bulk_create(&pool, user_id, &mutualfund_items)
        .await
        .expect("mutualfund bulk_create failed");
    assert_eq!(mutualfund_created.inserted, 2);
    assert_eq!(mutualfund_created.skipped, 0);

    let mutualfund_uploaded =
        mutualfund_svc::upload_csv(&pool, user_id, make_mutualfund_csv().as_bytes())
            .await
            .expect("mutualfund upload_csv failed");
    assert_eq!(mutualfund_uploaded.inserted, 1);
    assert_eq!(mutualfund_uploaded.skipped, 0);
    assert!(mutualfund_uploaded.errors.is_empty());

    assert_eq!(
        mutualfund_svc::search(&pool, user_id, &default_mutualfund_search_params())
            .await
            .expect("mutualfund list failed")
            .data
            .len(),
        3
    );
    assert_eq!(
        mutualfund_svc::delete_all(&pool, user_id)
            .await
            .expect("mutualfund delete_all failed"),
        3
    );
    assert!(
        mutualfund_svc::search(&pool, user_id, &default_mutualfund_search_params())
            .await
            .expect("mutualfund list after delete failed")
            .data
            .is_empty()
    );

    assert!(
        dividend_svc::search(&pool, user_id, &default_dividend_search_params())
            .await
            .expect("initial dividend list failed")
            .data
            .is_empty()
    );
    let dividend_empty = dividend_svc::bulk_create(&pool, user_id, &[])
        .await
        .expect("empty dividend bulk_create failed");
    assert_eq!(dividend_empty.inserted, 0);
    assert_eq!(dividend_empty.skipped, 0);

    let dividend_items = vec![make_dividend_item("1001"), make_dividend_item("1002")];
    let dividend_created = dividend_svc::bulk_create(&pool, user_id, &dividend_items)
        .await
        .expect("dividend bulk_create failed");
    assert_eq!(dividend_created.inserted, 2);
    assert_eq!(dividend_created.skipped, 0);

    let dividend_uploaded =
        dividend_svc::upload_csv(&pool, user_id, make_dividend_csv().as_bytes())
            .await
            .expect("dividend upload_csv failed");
    assert_eq!(dividend_uploaded.inserted, 1);
    assert_eq!(dividend_uploaded.skipped, 0);
    assert!(dividend_uploaded.errors.is_empty());

    assert_eq!(
        dividend_svc::search(&pool, user_id, &default_dividend_search_params())
            .await
            .expect("dividend list failed")
            .data
            .len(),
        3
    );
    assert_eq!(
        dividend_svc::delete_all(&pool, user_id)
            .await
            .expect("dividend delete_all failed"),
        3
    );
    assert!(
        dividend_svc::search(&pool, user_id, &default_dividend_search_params())
            .await
            .expect("dividend list after delete failed")
            .data
            .is_empty()
    );

    assert!(
        domestic_stock_svc::search(&pool, user_id, &default_domestic_stock_search_params())
            .await
            .expect("initial domestic_stock list failed")
            .data
            .is_empty()
    );
    let domestic_stock_empty = domestic_stock_svc::bulk_create(&pool, user_id, &[])
        .await
        .expect("empty domestic_stock bulk_create failed");
    assert_eq!(domestic_stock_empty.inserted, 0);
    assert_eq!(domestic_stock_empty.skipped, 0);

    let domestic_stock_items = vec![
        make_domestic_stock_item("3001"),
        make_domestic_stock_item("3002"),
    ];
    let domestic_stock_created =
        domestic_stock_svc::bulk_create(&pool, user_id, &domestic_stock_items)
            .await
            .expect("domestic_stock bulk_create failed");
    assert_eq!(domestic_stock_created.inserted, 2);
    assert_eq!(domestic_stock_created.skipped, 0);

    let domestic_stock_uploaded =
        domestic_stock_svc::upload_csv(&pool, user_id, make_domestic_stock_csv().as_bytes())
            .await
            .expect("domestic_stock upload_csv failed");
    assert_eq!(domestic_stock_uploaded.inserted, 1);
    assert_eq!(domestic_stock_uploaded.skipped, 0);
    assert!(domestic_stock_uploaded.errors.is_empty());

    assert_eq!(
        domestic_stock_svc::search(&pool, user_id, &default_domestic_stock_search_params())
            .await
            .expect("domestic_stock list failed")
            .data
            .len(),
        3
    );
    assert_eq!(
        domestic_stock_svc::delete_all(&pool, user_id)
            .await
            .expect("domestic_stock delete_all failed"),
        3
    );
    assert!(
        domestic_stock_svc::search(&pool, user_id, &default_domestic_stock_search_params())
            .await
            .expect("domestic_stock list after delete failed")
            .data
            .is_empty()
    );

    assert!(
        asset_balance_svc::list(&pool, user_id, &default_pagination())
            .await
            .expect("initial asset_balance list failed")
            .data
            .is_empty()
    );
    let asset_balance_empty = asset_balance_svc::bulk_create(&pool, user_id, &[])
        .await
        .expect("empty asset_balance bulk_create failed");
    assert_eq!(asset_balance_empty.inserted, 0);
    assert_eq!(asset_balance_empty.skipped, 0);

    let asset_balance_items = vec![make_asset_item("1301"), make_asset_item("1605")];
    let asset_balance_created =
        asset_balance_svc::bulk_create(&pool, user_id, &asset_balance_items)
            .await
            .expect("asset_balance bulk_create failed");
    assert_eq!(asset_balance_created.inserted, 2);
    assert_eq!(asset_balance_created.skipped, 0);
    assert_eq!(
        asset_balance_svc::list(&pool, user_id, &default_pagination())
            .await
            .expect("asset_balance list after bulk_create failed")
            .data
            .len(),
        2
    );

    let asset_balance_uploaded =
        asset_balance_svc::upload_csv(&pool, user_id, make_asset_balance_csv().as_bytes())
            .await
            .expect("asset_balance upload_csv failed");
    assert_eq!(asset_balance_uploaded.inserted, 1);
    assert_eq!(asset_balance_uploaded.skipped, 0);
    assert!(asset_balance_uploaded.errors.is_empty());

    let asset_balance_rows = asset_balance_svc::list(&pool, user_id, &default_pagination())
        .await
        .expect("asset_balance list failed")
        .data;
    assert_eq!(asset_balance_rows.len(), 1);
    assert_eq!(asset_balance_rows[0].security_code, "7203");

    assert_eq!(
        asset_balance_svc::delete_all(&pool, user_id)
            .await
            .expect("asset_balance delete_all failed"),
        1
    );
    assert!(
        asset_balance_svc::list(&pool, user_id, &default_pagination())
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
    let user_id = Uuid::new_v4();

    // 1回目: 2銘柄を登録
    let items_a = vec![make_asset_item("1001"), make_asset_item("1002")];
    asset_balance_svc::bulk_create(&pool, user_id, &items_a)
        .await
        .expect("1回目 bulk_create 失敗");

    // 2回目: 3銘柄を登録（スナップショット置き換えなので合計3件になるはず）
    let items_b = vec![
        make_asset_item("2001"),
        make_asset_item("2002"),
        make_asset_item("2003"),
    ];
    asset_balance_svc::bulk_create(&pool, user_id, &items_b)
        .await
        .expect("2回目 bulk_create 失敗");

    let rows = asset_balance_svc::list(&pool, user_id, &default_pagination())
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

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires Docker to run Postgres container"]
async fn asset_balance_bulk_create_concurrent_same_user_no_mix() {
    let (pool, _node) = start_test_pool().await;
    let user_id = Uuid::new_v4();

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
        tokio::spawn(
            async move { asset_balance_svc::bulk_create(&pool_a, user_id, &items_a).await }
        ),
        tokio::spawn(
            async move { asset_balance_svc::bulk_create(&pool_b, user_id, &items_b).await }
        ),
    );
    res_a.unwrap().expect("task_a 失敗");
    res_b.unwrap().expect("task_b 失敗");

    let rows = asset_balance_svc::list(&pool, user_id, &default_pagination())
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
    let created = dividend_svc::bulk_create(&pool, user_id, &items)
        .await
        .expect("dividend bulk_create failed");
    assert_eq!(created.inserted, 3);
    assert_eq!(created.skipped, 0);

    let rows = dividend_svc::search(&pool, user_id, &default_dividend_search_params())
        .await
        .expect("dividend list failed")
        .data;
    assert_eq!(rows.len(), 3);

    let first_page = dividend_svc::search(
        &pool,
        user_id,
        &dividend_search_params_with_pagination(PaginationParams {
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

    let second_page = dividend_svc::search(
        &pool,
        user_id,
        &dividend_search_params_with_pagination(PaginationParams {
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

    let clamped_page = dividend_svc::search(
        &pool,
        user_id,
        &dividend_search_params_with_pagination(PaginationParams {
            page: Some(1),
            per_page: Some(5000),
        }),
    )
    .await
    .expect("dividend clamped page list failed");
    assert_eq!(clamped_page.total, 3);
    assert_eq!(clamped_page.per_page, 1000);

    let deleted = dividend_svc::delete_all(&pool, user_id)
        .await
        .expect("dividend delete_all failed");
    assert_eq!(deleted, 3);

    let rows = dividend_svc::search(&pool, user_id, &default_dividend_search_params())
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

    let first = dividend_svc::bulk_create(&pool, user_id, &items)
        .await
        .expect("first dividend bulk_create failed");
    assert_eq!(first.inserted, 1);
    assert_eq!(first.skipped, 0);

    let second = dividend_svc::bulk_create(&pool, user_id, &items)
        .await
        .expect("second dividend bulk_create failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 1);
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
    let created = domestic_stock_svc::bulk_create(&pool, user_id, &items)
        .await
        .expect("domestic_stock bulk_create failed");
    assert_eq!(created.inserted, 3);
    assert_eq!(created.skipped, 0);

    let rows = domestic_stock_svc::search(&pool, user_id, &default_domestic_stock_search_params())
        .await
        .expect("domestic_stock list failed")
        .data;
    assert_eq!(rows.len(), 3);

    let deleted = domestic_stock_svc::delete_all(&pool, user_id)
        .await
        .expect("domestic_stock delete_all failed");
    assert_eq!(deleted, 3);

    let rows = domestic_stock_svc::search(&pool, user_id, &default_domestic_stock_search_params())
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

    let first = domestic_stock_svc::bulk_create(&pool, user_id, &items)
        .await
        .expect("first domestic_stock bulk_create failed");
    assert_eq!(first.inserted, 1);
    assert_eq!(first.skipped, 0);

    let second = domestic_stock_svc::bulk_create(&pool, user_id, &items)
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
    let created = mutualfund_svc::bulk_create(&pool, user_id, &items)
        .await
        .expect("mutualfund bulk_create failed");
    assert_eq!(created.inserted, 2);
    assert_eq!(created.skipped, 0);

    let rows = mutualfund_svc::search(&pool, user_id, &default_mutualfund_search_params())
        .await
        .expect("mutualfund list failed")
        .data;
    assert_eq!(rows.len(), 2);

    let deleted = mutualfund_svc::delete_all(&pool, user_id)
        .await
        .expect("mutualfund delete_all failed");
    assert_eq!(deleted, 2);

    let rows = mutualfund_svc::search(&pool, user_id, &default_mutualfund_search_params())
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

    let first = mutualfund_svc::bulk_create(&pool, user_id, &items)
        .await
        .expect("first mutualfund bulk_create failed");
    assert_eq!(first.inserted, 1);
    assert_eq!(first.skipped, 0);

    let second = mutualfund_svc::bulk_create(&pool, user_id, &items)
        .await
        .expect("second mutualfund bulk_create failed");
    assert_eq!(second.inserted, 0);
    assert_eq!(second.skipped, 1);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn unauthenticated_requests_return_401() {
    let (pool, _node) = start_test_pool().await;
    let config = Config::default();
    let state = AppState {
        pool,
        secrets: Arc::new(Secrets {
            database_url: "postgresql://postgres:postgres@localhost/postgres".to_string(),
            jquants_api_key: None,
            google_client_id: None,
            google_client_secret: None,
            frontend_url: "http://localhost:8080".to_string(),
        }),
        client: Client::new(),
        dividend_cache: backend::state::DividendCacheState::default(),
    };
    let app = app_router(
        state,
        &config,
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
    );

    for path in [
        "/dividends",
        "/domestic-stocks",
        "/mutualfunds",
        "/asset-balances",
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("request failed");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{path}");
    }
}
