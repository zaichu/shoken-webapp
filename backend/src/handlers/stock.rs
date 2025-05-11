use crate::{
    errors::ApiError,
    extractors::{char_width_converter::halfwidth_to_fullwidth, validated_json::ValidatedJson},
    models::stock::Stock,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

pub async fn select_stock_info(
    Path(search_query): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let search_query = search_query
        .chars()
        .map(halfwidth_to_fullwidth)
        .collect::<String>();
    let stock = sqlx::query_as::<_, Stock>(
        "SELECT * FROM stock WHERE code = $1 OR name ILIKE $2 ORDER BY date DESC LIMIT 1",
    )
    .bind(&search_query)
    .bind(format!("%{search_query}%"))
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(stock)))
}

pub async fn add_stock_info(
    State(state): State<AppState>,
    ValidatedJson(data): ValidatedJson<Stock>,
) -> Result<impl IntoResponse, ApiError> {
    let stock = sqlx::query_as::<_, Stock>(
        "INSERT INTO stock (date, code, name, market_category, industry_code_33, industry_category_33, industry_code_17, industry_category_17, size_code, size_category) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) 
         RETURNING *"
    )
    .bind(data.date)
    .bind(&data.code)
    .bind(&data.name)
    .bind(&data.market_category)
    .bind(&data.industry_code_33)
    .bind(&data.industry_category_33)
    .bind(&data.industry_code_17)
    .bind(&data.industry_category_17)
    .bind(&data.size_code)
    .bind(&data.size_category)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(stock)))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        routing::{get, post},
        Router,
    };
    use chrono::NaiveDate;
    use reqwest::Client;
    use serde_json::{json, Value};
    use shuttle_runtime::SecretStore;
    use sqlx::{
        postgres::{PgConnectOptions, PgPoolOptions},
        Pool, Postgres,
    };
    use tower::ServiceExt;

    async fn setup_test_db() -> Pool<Postgres> {
        let options = PgConnectOptions::new()
            .host("localhost")
            .port(5432)
            .database("test_db")
            .username("postgres")
            .password("password");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("Failed to connect to database");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS stock (
                date DATE NOT NULL,
                code VARCHAR(10) NOT NULL,
                name VARCHAR(100) NOT NULL,
                market_category VARCHAR(50) NOT NULL,
                industry_code_33 VARCHAR(10),
                industry_category_33 VARCHAR(100),
                industry_code_17 VARCHAR(10),
                industry_category_17 VARCHAR(100),
                size_code VARCHAR(10),
                size_category VARCHAR(50),
                PRIMARY KEY (date, code)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("Failed to create test table");

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
        .expect("Failed to insert test data");

        pool
    }

    fn setup_test_app(pool: Pool<Postgres>) -> Router {
        let bt = BTreeMap::from([
            ("1".to_owned(), "2".to_owned().into()),
            ("3".to_owned(), "4".to_owned().into()),
        ]);
        let client = Client::new();
        let app_state = AppState {
            pool: pool.clone(),
            secrets: SecretStore::new(bt),
            client,
        };

        Router::new()
            .route("/stock/:search_query", get(select_stock_info))
            .route("/stock", post(add_stock_info))
            .with_state(app_state)
    }

    #[tokio::test]
    #[ignore]
    async fn test_select_stock_info() {
        let pool = setup_test_db().await;
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
    #[ignore]
    async fn test_add_stock_info() {
        let pool = setup_test_db().await;
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
}
