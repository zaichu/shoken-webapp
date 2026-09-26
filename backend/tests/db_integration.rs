use axum::{
    body::{to_bytes, Body},
    http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, Method, Request, StatusCode},
};
use backend::{
    config::Config,
    db::{connect_pool_lazy, run_migrations, wait_for_pool_with_retry},
    models::asset_balance::{AssetBalanceSearchQueryParams, CreateAssetBalanceRequest},
    models::common::{PaginationParams, SearchQueryParams},
    models::dividend::{CreateDividendRequest, DividendSearchQueryParams},
    models::domestic_stock::{CreateDomesticStockRequest, DomesticStockSearchQueryParams},
    models::mutualfund::{CreateMutualfundRequest, MutualfundSearchQueryParams},
    models::user::GoogleUserInfo,
    routes::app_router,
    services::asset_balance as asset_balance_svc,
    services::auth as auth_svc,
    services::dividend as dividend_svc,
    services::dividend_cache,
    services::domestic_stock as domestic_stock_svc,
    services::mutualfund as mutualfund_svc,
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
use reqwest::Client;
use rust_decimal_macros::dec;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, sync::atomic::AtomicBool, sync::Arc, time::Duration};
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
        asset_balance_svc::search(&pool, user_id, &default_asset_balance_search_params())
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
        asset_balance_svc::search(&pool, user_id, &default_asset_balance_search_params())
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

    let asset_balance_rows =
        asset_balance_svc::search(&pool, user_id, &default_asset_balance_search_params())
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
        asset_balance_svc::search(&pool, user_id, &default_asset_balance_search_params())
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

    let rows = asset_balance_svc::search(&pool, user_id, &default_asset_balance_search_params())
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
        tokio::spawn(
            async move { asset_balance_svc::bulk_create(&pool_a, user_id, &items_a).await }
        ),
        tokio::spawn(
            async move { asset_balance_svc::bulk_create(&pool_b, user_id, &items_b).await }
        ),
    );
    res_a.unwrap().expect("task_a 失敗");
    res_b.unwrap().expect("task_b 失敗");

    let rows = asset_balance_svc::search(&pool, user_id, &default_asset_balance_search_params())
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
    let session1 = Uuid::parse_str(&token1).expect("セッションIDはUUID");

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
    let session2 = Uuid::parse_str(&token2).unwrap();
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
    sqlx::query("UPDATE sessions SET expires_at = NOW() - INTERVAL '1 hour' WHERE id = $1")
        .bind(session2)
        .execute(&pool)
        .await
        .unwrap();
    assert!(auth_svc::select_user_id_by_session(&pool, session2)
        .await
        .unwrap()
        .is_none());
    assert!(auth_svc::select_user_by_session(&pool, session2)
        .await
        .unwrap()
        .is_none());

    // delete_session で明示失効
    sqlx::query("UPDATE sessions SET expires_at = NOW() + INTERVAL '1 hour' WHERE id = $1")
        .bind(session2)
        .execute(&pool)
        .await
        .unwrap();
    auth_svc::delete_session(&pool, session2)
        .await
        .expect("セッション削除");
    assert!(auth_svc::select_user_id_by_session(&pool, session2)
        .await
        .unwrap()
        .is_none());

    // delete_account はセッションごとユーザーを削除する
    let token3 = auth_svc::upsert_user_and_rotate_session(&pool, &info2)
        .await
        .expect("3回目のセッション発行");
    let session3 = Uuid::parse_str(&token3).unwrap();
    auth_svc::delete_account(&pool, user.id)
        .await
        .expect("アカウント削除");
    assert!(auth_svc::select_user_by_session(&pool, session3)
        .await
        .unwrap()
        .is_none());
    let remaining: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE id = $1")
        .bind(user.id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(remaining.is_none());
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
    other.account = "NISA".to_string();
    other.settlement_date = NaiveDate::from_ymd_opt(2023, 1, 10).unwrap();
    dividend_svc::bulk_create(&pool, user_id, &[older, newer, other])
        .await
        .expect("dividend bulk_create");

    let mut dividend_params = default_dividend_search_params();
    dividend_params.search.include_facets = Some(true);
    let facets = dividend_svc::search(&pool, user_id, &dividend_params)
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
    ds_older.account = "NISA".to_string();
    ds_older.trade_date = NaiveDate::from_ymd_opt(2023, 5, 15).unwrap();
    let ds_newer = make_domestic_stock_item("3002");
    domestic_stock_svc::bulk_create(&pool, user_id, &[ds_older, ds_newer])
        .await
        .expect("domestic_stock bulk_create");

    let mut ds_params = default_domestic_stock_search_params();
    ds_params.search.include_facets = Some(true);
    let facets = domestic_stock_svc::search(&pool, user_id, &ds_params)
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
    mutualfund_svc::bulk_create(&pool, user_id, &[mf_older, mf_newer])
        .await
        .expect("mutualfund bulk_create");

    let mut mf_params = default_mutualfund_search_params();
    mf_params.search.include_facets = Some(true);
    let facets = mutualfund_svc::search(&pool, user_id, &mf_params)
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
    asset_balance_svc::bulk_create(&pool, user_id, &asset_items)
        .await
        .expect("asset_balance bulk_create");

    let mut ab_params = default_asset_balance_search_params();
    ab_params.search.include_facets = Some(true);
    let facets = asset_balance_svc::search(&pool, user_id, &ab_params)
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
    let client = Client::new();
    let running = Arc::new(AtomicBool::new(false));

    assert!(
        dividend_cache::get_batch(&pool, &client, None, &[], &running)
            .await
            .unwrap()
            .is_empty()
    );

    dividend_cache::persistence::update_cache_error(&pool, "1234", "fetch failed")
        .await
        .expect("エラー記録");
    let items = dividend_cache::get_batch(&pool, &client, None, &["1234".to_string()], &running)
        .await
        .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].security_code, "1234");
    assert_eq!(items[0].status, "error");
    assert_eq!(items[0].dividend_per_share, None);
    assert!(items[0].is_stale, "error 記録は即時再取得対象");

    dividend_cache::persistence::update_cache_error_with_cooldown(&pool, "5678", "429", 3600)
        .await
        .expect("cooldown 付きエラー記録");
    let items = dividend_cache::get_batch(&pool, &client, None, &["5678".to_string()], &running)
        .await
        .unwrap();
    assert_eq!(items[0].status, "error");
    assert!(!items[0].is_stale, "cooldown 中は再取得対象外");

    // 既存行への upsert: stale_at が cooldown で未来に更新される
    dividend_cache::persistence::update_cache_error_with_cooldown(&pool, "1234", "429", 3600)
        .await
        .expect("既存行の上書き");
    let items = dividend_cache::get_batch(&pool, &client, None, &["1234".to_string()], &running)
        .await
        .unwrap();
    assert!(!items[0].is_stale);
    let row: (String,) = sqlx::query_as(
        "SELECT error_message FROM dividend_per_share_cache WHERE security_code = '1234'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "429");

    dividend_cache::acquire_rate_slot(&pool)
        .await
        .expect("スロット予約");
    let row: (chrono::DateTime<Utc>,) = sqlx::query_as(
        "SELECT next_available_at FROM market_data_provider_rate_control \
         WHERE provider = 'jquants'",
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

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn user_row_limit_rejects_over_limit_inserts() {
    use backend::services::bulk_helpers::{ensure_user_row_limit_with, UserDataDomain};
    let (pool, _node) = start_test_pool().await;
    let user_id = create_test_user(&pool).await;
    const LIMIT: i64 = 3;

    // 追記型(dividends): 既存行数 + 追加分が上限を超えると拒否
    let items = vec![make_dividend_item("1001"), make_dividend_item("1002")];
    dividend_svc::bulk_create(&pool, user_id, &items)
        .await
        .expect("初回 bulk_create は成功");

    // 上限ちょうど(既存2+追加1=3)は許可、超過(既存2+追加2=4)は拒否
    assert!(
        ensure_user_row_limit_with(&pool, user_id, UserDataDomain::Dividends, 1, LIMIT)
            .await
            .is_ok()
    );
    let err = ensure_user_row_limit_with(&pool, user_id, UserDataDomain::Dividends, 2, LIMIT)
        .await
        .expect_err("既存2+追加2=4 > 3 で拒否");
    assert!(
        matches!(err, backend::errors::ApiError::ValidationError(_)),
        "期待しないエラー: {err:?}"
    );

    // 追記型(domestic_stocks/mutualfunds)も同じ判定
    for domain in [UserDataDomain::DomesticStocks, UserDataDomain::MutualFunds] {
        assert!(ensure_user_row_limit_with(&pool, user_id, domain, 4, LIMIT)
            .await
            .is_err());
    }

    // 置換型(asset_balances): 追加分のみで上限判定(既存行数を見ない)
    let asset_items = vec![make_asset_item("1301"), make_asset_item("1605")];
    asset_balance_svc::bulk_create(&pool, user_id, &asset_items)
        .await
        .expect("asset_balances は置換のため上限内");
    // 既存2件あっても追加分4件 > 上限3 で拒否(既存行数を見ない)
    let err = ensure_user_row_limit_with(&pool, user_id, UserDataDomain::AssetBalances, 4, LIMIT)
        .await
        .expect_err("追加4件 > 上限3 で拒否");
    assert!(
        matches!(err, backend::errors::ApiError::ValidationError(_)),
        "期待しないエラー: {err:?}"
    );
    assert!(
        ensure_user_row_limit_with(&pool, user_id, UserDataDomain::AssetBalances, 3, LIMIT)
            .await
            .is_ok()
    );
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn wait_for_pool_with_retry_succeeds_and_fails() {
    let (pool, _node) = start_test_pool().await;
    wait_for_pool_with_retry(&pool)
        .await
        .expect("稼働中の pool では成功");

    let dead = connect_pool_lazy("postgres://postgres:postgres@127.0.0.1:1/postgres", 1)
        .expect("lazy pool 構築");
    let err = wait_for_pool_with_retry(&dead)
        .await
        .expect_err("接続不能な DB はリトライ上限で Err");
    assert!(err.contains("接続に失敗"), "期待しないエラー: {err}");
}
