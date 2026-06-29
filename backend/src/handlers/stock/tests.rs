use {
    super::*,
    crate::state::Secrets,
    axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::get,
        Router,
    },
    chrono::NaiveDate,
    reqwest::Client,
    serde_json::{json, Value},
    sqlx::{Pool, Postgres},
    std::sync::Arc,
    testcontainers::runners::AsyncRunner,
    testcontainers_modules::postgres::Postgres as PgImage,
    tokio::time::{sleep, timeout, Duration},
    tower::ServiceExt,
};

const BODY_LIMIT: usize = 100;

async fn setup_test_db() -> (Pool<Postgres>, impl Drop) {
    let node = PgImage::default().start().await.unwrap();
    let port = node.get_host_port_ipv4(5432).await.unwrap();
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", port);

    let pool = {
        let mut last_error = None;
        let mut pool_ok = None;

        for _ in 0..20 {
            match timeout(
                Duration::from_secs(2),
                sqlx::postgres::PgPoolOptions::new().connect(&database_url),
            )
            .await
            {
                Ok(Ok(pool)) => {
                    pool_ok = Some(pool);
                    break;
                }
                Ok(Err(error)) => last_error = Some(error),
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
        r#"INSERT INTO stock (date, code, name, market_category, industry_code_33, industry_category_33, industry_code_17, industry_category_17, size_code, size_category) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"#,
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
    Router::new()
        .route(
            "/api/v1/stocks",
            get(crate::handlers::v1::stocks::search).post(create_stock),
        )
        .with_state(crate::AppState {
            pool: pool.clone(),
            secrets: Arc::new(Secrets {
                database_url: "postgresql://postgres:postgres@localhost/postgres".to_string(),
                jquants_api_key: None,
                google_client_id: None,
                google_client_secret: None,
                frontend_url: "http://localhost:8080".to_string(),
            }),
            client: Client::new(),
            dividend_cache: crate::state::DividendCacheState::default(),
        })
}

async fn call(
    app: Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> axum::response::Response {
    let mut request = Request::builder().method(method).uri(uri);
    let body = if let Some(payload) = body {
        request = request.header("content-type", "application/json");
        Body::from(payload.to_string())
    } else {
        Body::empty()
    };

    app.oneshot(request.body(body).unwrap()).await.unwrap()
}

async fn make_session(pool: &Pool<Postgres>) -> String {
    let (user_id,): (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO users (google_id, email) VALUES ('test-google-id', 'test@example.com') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    crate::services::auth::create_session(pool, user_id)
        .await
        .unwrap()
}

async fn call_with_session(
    app: Router,
    method: &str,
    uri: &str,
    session_cookie: &str,
) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method(method)
            .uri(uri)
            .header("cookie", session_cookie)
            .body(Body::empty())
            .unwrap(),
    )
    .await
    .unwrap()
}

async fn read_json(response: axum::response::Response) -> Value {
    serde_json::from_slice(
        &axum::body::to_bytes(response.into_body(), BODY_LIMIT)
            .await
            .unwrap(),
    )
    .unwrap()
}

fn stock_payload(code: &str, name: &str) -> Value {
    json!({
        "date": "2025-03-25",
        "code": code,
        "name": name,
        "market_category": "スタンダード",
        "industry_code_33": "456",
        "industry_category_33": "製造業",
        "industry_code_17": "45",
        "industry_category_17": "製造",
        "size_code": "20",
        "size_category": "中型株"
    })
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn test_search_stock() {
    let (pool, _node) = setup_test_db().await;
    let session_token = make_session(&pool).await;
    let cookie = format!("session_token={}", session_token);
    let app = setup_test_app(pool);

    let response =
        call_with_session(app.clone(), "GET", "/api/v1/stocks?query=1234", &cookie).await;
    assert_eq!(response.status(), StatusCode::OK);

    let stock = read_json(response).await;
    assert_eq!(
        (stock["code"].as_str(), stock["name"].as_str()),
        (Some("1234"), Some("テスト株式会社"))
    );

    for (uri, expected_status) in [
        ("/api/v1/stocks?query=テスト", StatusCode::OK),
        ("/api/v1/stocks?query=9999", StatusCode::NOT_FOUND),
    ] {
        assert_eq!(
            call_with_session(app.clone(), "GET", uri, &cookie)
                .await
                .status(),
            expected_status
        );
    }
}

#[tokio::test]
#[ignore = "requires Docker to run Postgres container"]
async fn test_create_stock() {
    let (pool, _node) = setup_test_db().await;
    let app = setup_test_app(pool);

    let response = call(
        app.clone(),
        "POST",
        "/api/v1/stocks",
        Some(stock_payload("5678", "新規テスト株式会社")),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);

    let stock = read_json(response).await;
    assert_eq!(
        (stock["code"].as_str(), stock["name"].as_str()),
        (Some("5678"), Some("新規テスト株式会社"))
    );

    let mut invalid_data = stock_payload("5678", "新規テスト株式会社");
    invalid_data["code"] = json!("");
    assert_eq!(
        call(app, "POST", "/api/v1/stocks", Some(invalid_data))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn test_create_stock_unauthorized() {
    let pool = crate::db::connect_pool_lazy("postgresql://postgres:postgres@localhost/postgres", 1)
        .unwrap();
    let app = setup_test_app(pool);

    assert_eq!(
        call(
            app,
            "POST",
            "/api/v1/stocks",
            Some(stock_payload("9999", "未認証テスト"))
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn test_search_stock_unauthorized() {
    let pool = crate::db::connect_pool_lazy("postgresql://postgres:postgres@localhost/postgres", 1)
        .unwrap();
    let app = setup_test_app(pool);

    assert_eq!(
        call(app, "GET", "/api/v1/stocks?query=7203", None)
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}
