use crate::errors::ApiError;
use crate::models::market_data::providers::jquants::FinSummaryQuery;
use crate::services::market_data::providers::jquants::JQuantsClient;
use sqlx::PgPool;

use super::logic::extract_dividend;

/// JQuants API から取得してキャッシュを更新する
pub async fn fetch_and_cache(
    pool: &PgPool,
    jquants_client: &JQuantsClient,
    code: &str,
) -> Result<String, ApiError> {
    let params = FinSummaryQuery {
        code: code.to_string(),
        from: None,
        to: None,
    };

    let response = jquants_client.get_fin_summary(params).await?;

    let (dividend_per_share, status) = extract_dividend(&response.data);

    sqlx::query(
        r#"
        INSERT INTO dividend_per_share_cache
            (security_code, dividend_per_share, status, fetched_at, stale_at, provider, updated_at)
        VALUES ($1, $2, $3, NOW(), NOW() + INTERVAL '7 days', 'jquants', NOW())
        ON CONFLICT (security_code) DO UPDATE
            SET dividend_per_share = EXCLUDED.dividend_per_share,
                status             = EXCLUDED.status,
                fetched_at         = EXCLUDED.fetched_at,
                stale_at           = NOW() + INTERVAL '7 days',
                provider           = EXCLUDED.provider,
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

/// 429 レートリミット発生時のエラーを cooldown 付きでキャッシュに記録する
/// stale_at を future に設定することで cooldown 中の即時再取得を防ぐ
pub async fn update_cache_error_with_cooldown(
    pool: &PgPool,
    code: &str,
    error_msg: &str,
    cooldown_secs: i32,
) -> Result<(), ApiError> {
    let truncated = truncate_error_message(error_msg);

    sqlx::query(
        r#"
        INSERT INTO dividend_per_share_cache
            (security_code, dividend_per_share, status, error_message, stale_at, provider, updated_at)
        VALUES ($1, NULL, 'error', $2, NOW() + $3 * INTERVAL '1 second', 'jquants', NOW())
        ON CONFLICT (security_code) DO UPDATE
            SET status        = 'error',
                error_message = EXCLUDED.error_message,
                stale_at      = NOW() + $3 * INTERVAL '1 second',
                updated_at    = NOW()
        "#,
    )
    .bind(code)
    .bind(truncated)
    .bind(cooldown_secs)
    .execute(pool)
    .await?;

    Ok(())
}

/// エラーメッセージを最大 200 文字に切り捨てる（マルチバイト文字境界を考慮）
fn truncate_error_message(msg: &str) -> &str {
    if msg.len() <= 200 {
        return msg;
    }
    let end = msg
        .char_indices()
        .nth(200)
        .map(|(i, _)| i)
        .unwrap_or(msg.len());
    &msg[..end]
}

/// エラー情報をキャッシュに記録する
pub async fn update_cache_error(
    pool: &PgPool,
    code: &str,
    error_msg: &str,
) -> Result<(), ApiError> {
    let truncated = truncate_error_message(error_msg);

    sqlx::query(
        r#"
        INSERT INTO dividend_per_share_cache
            (security_code, dividend_per_share, status, error_message, stale_at, provider, updated_at)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_error_message() {
        // 200文字以下はそのまま
        let short = "エラー";
        assert_eq!(truncate_error_message(short), short);

        // 200文字ちょうどもそのまま
        let exact = "a".repeat(200);
        assert_eq!(truncate_error_message(&exact), exact);

        // 201文字は200文字に切り捨て
        let long = "a".repeat(201);
        assert_eq!(truncate_error_message(&long), "a".repeat(200));

        // マルチバイト文字（3バイト）は文字境界で切り捨て
        // 'あ' は3バイトなので len() > 200 になるが、200文字目で正しく切る
        let japanese = "あ".repeat(201);
        let result = truncate_error_message(&japanese);
        assert_eq!(result.chars().count(), 200);
        assert!(result.is_char_boundary(result.len()));

        // 空文字はそのまま
        assert_eq!(truncate_error_message(""), "");
    }
}
