use axum::{
    body::{to_bytes, Body},
    http::{header::ACCESS_CONTROL_ALLOW_ORIGIN, Method, Request, StatusCode},
};
use backend::{
    config::Config,
    db::run_migrations,
    routes::app_router,
    state::{AppState, Secrets},
};
use chrono::NaiveDate;
use reqwest::Client;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::{env, sync::Arc, time::Duration};
use testcontainers::{clients::Cli, images::postgres::Postgres};
use tokio::time::{sleep, timeout};
use tower::ServiceExt;

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
        let attempt =
            timeout(Duration::from_secs(2), PgPoolOptions::new().connect(database_url)).await;
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
    let _cors_origins =
        EnvGuard::set("CORS_ORIGINS", Some("https://shoken-webapp.vercel.app"));

    let docker = Cli::default();
    let node = docker.run(Postgres::default());
    let port = node.get_host_port_ipv4(5432);
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
