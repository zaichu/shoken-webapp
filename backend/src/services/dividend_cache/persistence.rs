use crate::errors::ApiError;
use crate::models::jquants::FinSummaryQuery;
use crate::services::jquants::{JQuantsService, FIN_SUMMARY_URL};
use reqwest::Client;
use sqlx::PgPool;

use super::logic::extract_dividend;

/// JQuants API から取得してキャッシュを更新する
pub async fn fetch_and_cache(
    pool: &PgPool,
    client: &Client,
    api_key: &str,
    code: &str,
) -> Result<String, ApiError> {
    let params = FinSummaryQuery {
        code: code.to_string(),
        from: None,
        to: None,
    };

    let response =
        JQuantsService::get_fin_summary(client, params, api_key, FIN_SUMMARY_URL).await?;

    let (dividend_per_share, status) = extract_dividend(&response.data);

    sqlx::query(
        r#"
        INSERT INTO jquants_dividend_cache
            (security_code, dividend_per_share, status, fetched_at, stale_at, source, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW() + INTERVAL '7 days', 'jquants', NOW())
        ON CONFLICT (security_code) DO UPDATE
            SET dividend_per_share = EXCLUDED.dividend_per_share,
                status             = EXCLUDED.status,
                fetched_at         = EXCLUDED.fetched_at,
                stale_at           = NOW() + INTERVAL '7 days',
                source             = EXCLUDED.source,
                error_message      = NULL,
                updated_at         = NOW()
        "#,
    )
    .bind(code)
    .bind(dividend_per_share)
    .bind(&status)
    .execute(pool)
    .await?;

    Ok(status)
}

/// エラー情報をキャッシュに記録する
pub async fn update_cache_error(
    pool: &PgPool,
    code: &str,
    error_msg: &str,
) -> Result<(), ApiError> {
    // エラーメッセージは最大 200 文字に切り捨て（機密情報混入を防ぐため短く保つ）
    // char_indices で文字境界を求めてスライスし、マルチバイト文字での panic を防ぐ
    let truncated = if error_msg.len() > 200 {
        let end = error_msg
            .char_indices()
            .nth(200)
            .map(|(i, _)| i)
            .unwrap_or(error_msg.len());
        &error_msg[..end]
    } else {
        error_msg
    };

    sqlx::query(
        r#"
        INSERT INTO jquants_dividend_cache
            (security_code, dividend_per_share, status, error_message, stale_at, source, updated_at)
        VALUES ($1, NULL, 'error', $2, NULL, 'jquants', NOW())
        ON CONFLICT (security_code) DO UPDATE
            SET status        = 'error',
                error_message = EXCLUDED.error_message,
                stale_at      = NULL,
                updated_at    = NOW()
        "#,
    )
    .bind(code)
    .bind(truncated)
    .execute(pool)
    .await?;

    Ok(())
}
