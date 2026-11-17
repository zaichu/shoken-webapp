use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::services::jquants::{FinancialStatementsQuery, JQuantsClient};
use shared::dividend_per_share::DividendCacheStatus;
use shared::value::SecurityCode;

use super::logic::extract_dividend;

/// JQuants API から取得してキャッシュを更新する
pub async fn fetch_and_cache(
    pool: &Db,
    jquants_client: &JQuantsClient,
    code: &SecurityCode,
) -> Result<DividendCacheStatus, ApiError> {
    let params = FinancialStatementsQuery {
        code: code.to_string(),
        from: None,
        to: None,
    };

    let response = jquants_client.get_fin_summary(params).await?;

    let (dividend_per_share, status) = extract_dividend(&response.data);

    crate::db::query(
        "SELECT public.dividend_cache_apply_success_v2($1, $2, $3)",
        vec![
            Bind::from(code.as_str()),
            Bind::from(dividend_per_share),
            Bind::from(status),
        ],
    )
    .execute(pool)
    .await?;

    Ok(status)
}

/// エラー情報をキャッシュに記録する。
/// stale_at を cooldown_secs だけ未来に設定し、再取得は間隔経過後に限る
/// （NULL だと drain が毎分拾い続けるため、失敗行は必ず間隔を持たせる）
pub async fn update_cache_error(
    pool: &Db,
    code: &SecurityCode,
    error_msg: &str,
    cooldown_secs: i32,
) -> Result<(), ApiError> {
    let truncated = truncate_error_message(error_msg);

    crate::db::query(
        "SELECT public.dividend_cache_apply_error_v2($1, $2, $3)",
        vec![
            Bind::from(code.as_str()),
            Bind::from(truncated),
            Bind::from(cooldown_secs),
        ],
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// エラーメッセージを最大 200 文字に切り捨てる（マルチバイト文字境界を考慮）
fn truncate_error_message(msg: &str) -> &str {
    if msg.len() <= 200 {
        return msg;
    }
    let end = msg.char_indices().nth(200).map_or(msg.len(), |(i, _)| i);
    &msg[..end]
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
