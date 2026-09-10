use crate::{
    errors::ApiError,
    models::market_data::financial_statement::{
        FinancialStatementsQuery, FinancialStatementsResponse,
    },
    services::market_data::providers::jquants::JQuantsClient,
};
use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// 決算サマリーキャッシュの TTL（時間）。決算データは日次以上の頻度では更新されない。
pub const FINANCIAL_STATEMENTS_CACHE_TTL_HOURS: i64 = 24;

/// キャッシュキーの区切り文字（銘柄コード・日付に現れない単位区切りを使用）。
const CACHE_KEY_SEPARATOR: char = '\u{1f}';

/// キャッシュキーを組み立てる。from/to の None は空文字として扱い、
/// キー文字列の完全一致で code + from + to の組み合わせを一意化する。
fn cache_key(code: &str, from: Option<&str>, to: Option<&str>) -> String {
    format!(
        "{code}{CACHE_KEY_SEPARATOR}{}{CACHE_KEY_SEPARATOR}{}",
        from.unwrap_or_default(),
        to.unwrap_or_default()
    )
}

/// キャッシュヒット時は外部APIを呼ばずに返し、ミス・TTL切れ時のみ取得して保存する。
/// キャッシュ保存の失敗はリクエストを失敗させない（取得したてのレスポンスを返す）。
pub async fn get_or_fetch(
    pool: &PgPool,
    jquants_client: &JQuantsClient,
    params: FinancialStatementsQuery,
) -> Result<FinancialStatementsResponse, ApiError> {
    if let Some(cached) = get_cached(
        pool,
        &params.code,
        params.from.as_deref(),
        params.to.as_deref(),
    )
    .await?
    {
        return Ok(cached);
    }

    let code = params.code.clone();
    let from = params.from.clone();
    let to = params.to.clone();
    let response = jquants_client.get_fin_summary(params).await?;
    if let Err(e) = put_cached(pool, &code, from.as_deref(), to.as_deref(), &response).await {
        tracing::warn!("決算サマリーキャッシュ保存失敗: {}", e);
    }
    Ok(response)
}

/// TTL 内のキャッシュがあれば返す。なし・TTL切れ・破損時は None。
pub async fn get_cached(
    pool: &PgPool,
    code: &str,
    from: Option<&str>,
    to: Option<&str>,
) -> Result<Option<FinancialStatementsResponse>, ApiError> {
    // sqlx の `json` feature を使っていないため JSONB はテキストで読み出して serde で戻す。
    let row: Option<(String, DateTime<Utc>)> = sqlx::query_as(
        r#"
        SELECT response::text, fetched_at
        FROM financial_statements_cache
        WHERE cache_key = $1
        "#,
    )
    .bind(cache_key(code, from, to))
    .fetch_optional(pool)
    .await?;

    match row {
        Some((body, fetched_at)) if is_fresh(fetched_at, Utc::now()) => {
            match serde_json::from_str(&body) {
                Ok(response) => Ok(Some(response)),
                Err(e) => {
                    tracing::warn!("決算サマリーキャッシュ破損のため再取得する: {}", e);
                    Ok(None)
                }
            }
        }
        _ => Ok(None),
    }
}

/// 取得結果を UPSERT 保存する。キーは code + from + to の完全一致（NULL 含む）。
pub async fn put_cached(
    pool: &PgPool,
    code: &str,
    from: Option<&str>,
    to: Option<&str>,
    response: &FinancialStatementsResponse,
) -> Result<(), ApiError> {
    let body = serde_json::to_string(response)?;
    sqlx::query(
        r#"
        INSERT INTO financial_statements_cache
            (cache_key, code, from_param, to_param, response, fetched_at, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5::jsonb, NOW(), NOW(), NOW())
        ON CONFLICT (cache_key) DO UPDATE
            SET response   = EXCLUDED.response,
                fetched_at = NOW(),
                updated_at = NOW()
        "#,
    )
    .bind(cache_key(code, from, to))
    .bind(code)
    .bind(from)
    .bind(to)
    .bind(&body)
    .execute(pool)
    .await?;
    Ok(())
}

fn is_fresh(fetched_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
    now.signed_duration_since(fetched_at)
        < chrono::Duration::hours(FINANCIAL_STATEMENTS_CACHE_TTL_HOURS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use std::time::Duration as StdDuration;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;
    use tokio::time::{sleep, timeout};
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 6, 10, 0, 0, 0)
            .single()
            .expect("固定時刻の生成に失敗")
    }

    #[test]
    fn test_is_fresh_within_ttl() {
        let now = fixed_now();
        assert!(is_fresh(
            now - chrono::Duration::hours(FINANCIAL_STATEMENTS_CACHE_TTL_HOURS - 1),
            now
        ));
    }

    #[test]
    fn test_is_fresh_at_ttl_boundary_is_stale() {
        let now = fixed_now();
        assert!(!is_fresh(
            now - chrono::Duration::hours(FINANCIAL_STATEMENTS_CACHE_TTL_HOURS),
            now
        ));
    }

    #[test]
    fn test_is_fresh_after_ttl() {
        let now = fixed_now();
        assert!(!is_fresh(
            now - chrono::Duration::hours(FINANCIAL_STATEMENTS_CACHE_TTL_HOURS + 1),
            now
        ));
    }

    #[test]
    fn test_cache_key_distinguishes_params() {
        let base = cache_key("7203", None, None);
        assert_ne!(base, cache_key("7204", None, None));
        assert_ne!(base, cache_key("7203", Some("2024-01-01"), None));
        assert_ne!(base, cache_key("7203", None, Some("2024-12-31")));
        assert_ne!(
            cache_key("7203", Some("2024-01-01"), Some("2024-12-31")),
            cache_key("7203", Some("2024-01-01"), None)
        );
        assert_eq!(base, cache_key("7203", None, None));
    }

    async fn start_test_pool() -> (PgPool, impl Drop) {
        let node = Postgres::default().start().await.unwrap();
        let port = node.get_host_port_ipv4(5432).await.unwrap();
        let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        let mut last_error = None;
        for _ in 0..20 {
            match timeout(
                StdDuration::from_secs(2),
                PgPoolOptions::new().connect(&database_url),
            )
            .await
            {
                Ok(Ok(pool)) => {
                    crate::db::run_migrations(&pool)
                        .await
                        .expect("migrations failed");
                    return (pool, node);
                }
                Ok(Err(err)) => last_error = Some(err),
                Err(_) => {}
            }
            sleep(StdDuration::from_millis(500)).await;
        }
        panic!("Failed to connect to Postgres: {:?}", last_error);
    }

    fn test_params(code: &str, from: Option<&str>, to: Option<&str>) -> FinancialStatementsQuery {
        FinancialStatementsQuery {
            code: code.to_string(),
            from: from.map(str::to_string),
            to: to.map(str::to_string),
        }
    }

    fn mock_body(code: &str) -> serde_json::Value {
        json!({
            "data": [
                {
                    "DiscDate": "2024-05-10",
                    "Code": code,
                    "DocType": "FY"
                }
            ]
        })
    }

    async fn start_mock_server(code: &str) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v2/fins/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(mock_body(code)))
            .mount(&server)
            .await;
        server
    }

    fn test_client(server: &MockServer) -> JQuantsClient {
        JQuantsClient::with_base_url(
            reqwest::Client::new(),
            "test-api-key".to_string(),
            format!("{}/v2/fins/summary", server.uri()),
        )
    }

    #[tokio::test]
    async fn test_miss_returns_none() {
        let (pool, _guard) = start_test_pool().await;
        assert!(get_cached(&pool, "7203", None, None)
            .await
            .expect("cache lookup failed")
            .is_none());
    }

    #[tokio::test]
    async fn test_second_request_served_from_cache_without_external_call() {
        let (pool, _guard) = start_test_pool().await;
        let server = start_mock_server("7203").await;
        let client = test_client(&server);

        let first = get_or_fetch(&pool, &client, test_params("7203", None, None))
            .await
            .expect("1回目の取得に失敗");
        assert_eq!(first.data.len(), 1);
        assert_eq!(first.data[0].local_code, "7203");

        let second = get_or_fetch(&pool, &client, test_params("7203", None, None))
            .await
            .expect("2回目の取得に失敗");
        assert_eq!(second.data.len(), 1);
        assert_eq!(second.data[0].local_code, "7203");

        let received = server.received_requests().await.unwrap();
        assert_eq!(
            received.len(),
            1,
            "2回目の同一パラメータは外部APIを呼ばないこと"
        );
    }

    #[tokio::test]
    async fn test_different_params_use_separate_cache_keys() {
        let (pool, _guard) = start_test_pool().await;
        let server = start_mock_server("7203").await;
        let client = test_client(&server);

        get_or_fetch(&pool, &client, test_params("7203", None, None))
            .await
            .expect("取得に失敗");
        get_or_fetch(
            &pool,
            &client,
            test_params("7203", Some("2024-01-01"), Some("2024-12-31")),
        )
        .await
        .expect("取得に失敗");

        let received = server.received_requests().await.unwrap();
        assert_eq!(
            received.len(),
            2,
            "from/to が異なる場合は別キーとして外部APIを呼ぶこと"
        );

        // 両方キャッシュ済みなので以降は外部APIを呼ばない
        get_or_fetch(&pool, &client, test_params("7203", None, None))
            .await
            .expect("取得に失敗");
        get_or_fetch(
            &pool,
            &client,
            test_params("7203", Some("2024-01-01"), Some("2024-12-31")),
        )
        .await
        .expect("取得に失敗");
        let received = server.received_requests().await.unwrap();
        assert_eq!(received.len(), 2);
    }

    #[tokio::test]
    async fn test_expired_cache_refetches_from_external_api() {
        let (pool, _guard) = start_test_pool().await;
        let server = start_mock_server("7203").await;
        let client = test_client(&server);

        get_or_fetch(&pool, &client, test_params("7203", None, None))
            .await
            .expect("取得に失敗");

        // fetched_at を TTL（24時間）より古く書き換える
        sqlx::query(
            "UPDATE financial_statements_cache SET fetched_at = NOW() - INTERVAL '25 hours' WHERE code = $1",
        )
        .bind("7203")
        .execute(&pool)
        .await
        .expect("fetched_at の書き換えに失敗");

        assert!(
            get_cached(&pool, "7203", None, None)
                .await
                .expect("cache lookup failed")
                .is_none(),
            "TTL経過後のキャッシュはミス扱いであること"
        );

        get_or_fetch(&pool, &client, test_params("7203", None, None))
            .await
            .expect("取得に失敗");
        let received = server.received_requests().await.unwrap();
        assert_eq!(received.len(), 2, "TTL経過後は再度外部APIから取得すること");
    }

    #[tokio::test]
    async fn test_api_error_is_not_cached_and_propagates() {
        let (pool, _guard) = start_test_pool().await;
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v2/fins/summary"))
            .respond_with(ResponseTemplate::new(500).set_body_string("upstream failed"))
            .mount(&server)
            .await;
        let client = test_client(&server);

        let error = get_or_fetch(&pool, &client, test_params("7203", None, None))
            .await
            .expect_err("500 ではエラーを返すこと");
        assert!(
            matches!(&error, ApiError::ApiError(message) if message.contains("upstream failed")),
            "既存と同じ ApiError であること: {:?}",
            error
        );

        // エラー時はキャッシュされないため再リクエストも外部APIへ到達する
        let _ = get_or_fetch(&pool, &client, test_params("7203", None, None)).await;
        let received = server.received_requests().await.unwrap();
        assert_eq!(received.len(), 2, "エラー結果はキャッシュしないこと");
    }
}
