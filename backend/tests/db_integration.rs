use axum::{
    body::{to_bytes, Body},
    http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, Method, Request, StatusCode},
};
use backend::{
    config::Config,
    db::run_migrations,
    models::asset_balance::CreateAssetBalanceRequest,
    models::dividend::CreateDividendRequest,
    models::domestic_stock::CreateDomesticStockRequest,
    models::mutualfund::CreateMutualfundRequest,
    routes::app_router,
    services::asset_balance as asset_balance_svc,
    services::dividend as dividend_svc,
    services::domestic_stock as domestic_stock_svc,
    services::mutualfund as mutualfund_svc,
    state::{AppState, Secrets},
};
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
    let app = app_router(state, &config);

    let req = Request::builder()
        .method(Method::GET)
        .uri("/stock/1234")
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

    let rows = asset_balance_svc::list(&pool, user_id)
        .await
        .expect("list 失敗");
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

    let rows = asset_balance_svc::list(&pool, user_id)
        .await
        .expect("list 失敗");

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

    let rows = dividend_svc::list(&pool, user_id)
        .await
        .expect("dividend list failed");
    assert_eq!(rows.len(), 3);

    let deleted = dividend_svc::delete_all(&pool, user_id)
        .await
        .expect("dividend delete_all failed");
    assert_eq!(deleted, 3);

    let rows = dividend_svc::list(&pool, user_id)
        .await
        .expect("dividend list after delete failed");
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

    let rows = domestic_stock_svc::list(&pool, user_id)
        .await
        .expect("domestic_stock list failed");
    assert_eq!(rows.len(), 3);

    let deleted = domestic_stock_svc::delete_all(&pool, user_id)
        .await
        .expect("domestic_stock delete_all failed");
    assert_eq!(deleted, 3);

    let rows = domestic_stock_svc::list(&pool, user_id)
        .await
        .expect("domestic_stock list after delete failed");
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

    let rows = mutualfund_svc::list(&pool, user_id)
        .await
        .expect("mutualfund list failed");
    assert_eq!(rows.len(), 2);

    let deleted = mutualfund_svc::delete_all(&pool, user_id)
        .await
        .expect("mutualfund delete_all failed");
    assert_eq!(deleted, 2);

    let rows = mutualfund_svc::list(&pool, user_id)
        .await
        .expect("mutualfund list after delete failed");
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
    let app = app_router(state, &config);

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
