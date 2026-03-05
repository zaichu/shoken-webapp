use std::sync::Arc;

use super::*;
use crate::state::Secrets;
use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use chrono::NaiveDate;
use reqwest::Client;
use serde_json::{json, Value};
use sqlx::{Pool, Postgres};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres as PgImage;
use tokio::time::{sleep, timeout, Duration};
use tower::ServiceExt;

/// testcontainers 経由で Postgres を起動し、マイグレーション + テストデータを投入する
/// 戻り値: (pool, _node) で _node を drop すると停止する
async fn setup_test_db() -> (Pool<Postgres>, impl Drop) {
    let node = PgImage::default().start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    // 接続リトライ
    let pool = {
        let mut last_error = None;
        let mut pool_ok = None;
        for _ in 0..20 {
            let attempt = timeout(
                Duration::from_secs(2),
                sqlx::postgres::PgPoolOptions::new().connect(&database_url),
            )
            .await;
            match attempt {
                Ok(Ok(p)) => {
                    pool_ok = Some(p);
                    break;
                }
                Ok(Err(e)) => last_error = Some(e),
                Err(_) => {}
            }
            sleep(Duration::from_millis(500)).await;
        }
        pool_ok.unwrap_or_else(|| panic!("DB 接続失敗: {:?}", last_error))
    };

    crate::db::run_migrations(&pool)
        .await
        .expect("マイグレーション失敗");

    sqlx::query(
        r#"
        INSERT INTO stock
        (date, code, name, market_category, industry_code_33, industry_category_33, industry_code_17, industry_category_17, size_code, size_category)
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
    .expect("テストデータ投入失敗");

    (pool, node)
}

fn setup_test_app(pool: Pool<Postgres>) -> Router {
    let secrets = Arc::new(Secrets {
        database_url: "postgresql://postgres:postgres@localhost/postgres".to_string(),
        jquants_api_key: None,
        google_client_id: None,
        google_client_secret: None,
        frontend_url: "http://localhost:8080".to_string(),
    });
    let client = Client::new();
    let app_state = crate::AppState {
        pool: pool.clone(),
        secrets,
        client,
        background_task_running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };

    Router::new()
        .route("/stock/{search_query}", get(select_stock_info))
        .route("/stock", post(add_stock_info))
        .with_state(app_state)
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn test_select_stock_info() {
    let (pool, _node) = setup_test_db().await;
    let app = setup_test_app(pool);

    // コードによる検索テスト
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/stock/1234")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 100)
        .await
        .unwrap();
    let stock: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(stock["code"], "1234");
    assert_eq!(stock["name"], "テスト株式会社");

    // 銘柄名による検索テスト
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/stock/テスト")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // 存在しない銘柄コードのテスト
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/stock/9999")
                .method("GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn test_add_stock_info() {
    let (pool, _node) = setup_test_db().await;
    let app = setup_test_app(pool.clone());

    let stock_data = json!({
        "date": "2025-03-25",
        "code": "5678",
        "name": "新規テスト株式会社",
        "market_category": "スタンダード",
        "industry_code_33": "456",
        "industry_category_33": "製造業",
        "industry_code_17": "45",
        "industry_category_17": "製造",
        "size_code": "20",
        "size_category": "中型株"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/stock")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(stock_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), 100)
        .await
        .unwrap();
    let stock: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(stock["code"], "5678");
    assert_eq!(stock["name"], "新規テスト株式会社");

    let invalid_data = json!({
        "date": "2025-03-25",
        "code": "",
        "name": "新規テスト株式会社",
        "market_category": "スタンダード",
        "industry_code_33": "456",
        "industry_category_33": "製造業",
        "industry_code_17": "45",
        "industry_category_17": "製造",
        "size_code": "20",
        "size_category": "中型株"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/stock")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(invalid_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

/// 未認証時に POST /stock が 401 を返すことを確認
/// セッション検証はハンドラー入口で実行され DB クエリは発生しないため DB 不要
#[tokio::test]
async fn test_add_stock_info_unauthorized() {
    let pool = crate::db::connect_pool_lazy("postgresql://postgres:postgres@localhost/postgres", 1)
        .unwrap();
    let app = setup_test_app(pool);

    let stock_data = json!({
        "date": "2025-03-25",
        "code": "9999",
        "name": "未認証テスト",
        "market_category": "プライム",
        "industry_code_33": null,
        "industry_category_33": null,
        "industry_code_17": null,
        "industry_category_17": null,
        "size_code": null,
        "size_category": null
    });

    // セッション Cookie なしでリクエスト
    let response = app
        .oneshot(
            Request::builder()
                .uri("/stock")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(stock_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
