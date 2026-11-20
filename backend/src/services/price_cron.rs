use crate::db::{Bind, Db};
use crate::errors::{ApiError, UpstreamError};
use time::{Date, OffsetDateTime};

pub const FETCH_GAP_MS: u64 = 500;

#[cfg(any(target_arch = "wasm32", test))]
pub fn upstream_status_error(status: u16, body: String) -> ApiError {
    if status == 429 {
        UpstreamError::RateLimited.into()
    } else {
        UpstreamError::Http { status, body }.into()
    }
}

#[expect(clippy::cast_possible_truncation)]
pub fn scheduled_time_ms(scheduled_ms: f64) -> Option<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(scheduled_ms as i64) * 1_000_000).ok()
}

pub fn short_failure_reason(error: &ApiError) -> String {
    match error {
        ApiError::Upstream(UpstreamError::Http { status, .. }) => format!("HTTP {status}"),
        ApiError::Upstream(UpstreamError::RateLimited) => "レート制限".to_string(),
        ApiError::Upstream(UpstreamError::Invalid(_)) => "検証エラー".to_string(),
        ApiError::Upstream(UpstreamError::Decode(_)) => "応答の解析に失敗".to_string(),
        #[cfg(target_arch = "wasm32")]
        ApiError::Upstream(UpstreamError::Transport(_)) => "通信エラー".to_string(),
        ApiError::Database(_) => "保存エラー".to_string(),
        _ => "取得エラー".to_string(),
    }
}

pub async fn select_targets(
    pool: &Db,
    targets_sql: &str,
    mark_sql: &str,
    target: Date,
    limit: i64,
) -> Result<Vec<String>, ApiError> {
    let mut tx = pool.begin().await?;
    let codes: Vec<String> =
        crate::db::query(targets_sql, vec![Bind::from(target), Bind::from(limit)])
            .fetch_all::<(String,), _>(&mut tx)
            .await?
            .into_iter()
            .map(|(code,)| code)
            .collect();
    tx.commit().await?;
    if !codes.is_empty() {
        crate::db::query(mark_sql, vec![Bind::from(codes.clone())])
            .execute(pool)
            .await?;
    }
    Ok(codes)
}

pub async fn record_failure(pool: &Db, record_sql: &str, label: &str, code: &str, reason: &str) {
    if let Err(e) = crate::db::query(record_sql, vec![Bind::from(code), Bind::from(reason)])
        .execute(pool)
        .await
    {
        #[cfg(target_arch = "wasm32")]
        worker::console_error!("{}の失敗記録に失敗: cd={}, err={}", label, code, e);
        #[cfg(not(target_arch = "wasm32"))]
        tracing::error!("{}の失敗記録に失敗: cd={}, err={}", label, code, e);
    }
}

pub async fn clear_failure(pool: &Db, clear_sql: &str, label: &str, code: &str) {
    if let Err(e) = crate::db::query(clear_sql, vec![Bind::from(code)])
        .execute(pool)
        .await
    {
        #[cfg(target_arch = "wasm32")]
        worker::console_warn!("{}の失敗記録の削除に失敗: cd={}, err={}", label, code, e);
        #[cfg(not(target_arch = "wasm32"))]
        tracing::warn!("{}の失敗記録の削除に失敗: cd={}, err={}", label, code, e);
    }
}

pub const fn none_succeeded(total: usize, succeeded: usize) -> bool {
    total > 0 && succeeded == 0
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn upstream_status_error_maps_429_for_both_feeds() {
        for status_error in [
            upstream_status_error(429, "ignored".to_string()),
            upstream_status_error(429, "other".to_string()),
        ] {
            assert!(matches!(
                status_error,
                ApiError::Upstream(UpstreamError::RateLimited)
            ));
        }
        for status in [403, 404, 500, 503] {
            match upstream_status_error(status, "bad".to_string()) {
                ApiError::Upstream(UpstreamError::Http { status: s, body }) => {
                    assert_eq!(s, status);
                    assert_eq!(body, "bad");
                }
                other => panic!("expected Http error, got {other}"),
            }
        }
    }

    #[test]
    fn short_failure_reason_matches_for_stock_and_fund() {
        let cases: [(ApiError, &str); 4] = [
            (
                ApiError::Upstream(UpstreamError::Http {
                    status: 403,
                    body: "https://example.invalid/secret".to_string(),
                }),
                "HTTP 403",
            ),
            (ApiError::Upstream(UpstreamError::RateLimited), "レート制限"),
            (
                ApiError::Upstream(UpstreamError::Invalid("理由".to_string())),
                "検証エラー",
            ),
            (
                ApiError::Upstream(UpstreamError::Decode(
                    serde_json::from_str::<serde_json::Value>("bad").unwrap_err(),
                )),
                "応答の解析に失敗",
            ),
        ];
        for (error, expected) in cases {
            let stock_reason = short_failure_reason(&error);
            let fund_reason = short_failure_reason(&error);
            assert_eq!(stock_reason, expected);
            assert_eq!(fund_reason, expected);
            assert!(!stock_reason.contains("example.invalid"));
        }
        assert_eq!(
            short_failure_reason(&ApiError::Database(crate::db::DbError::Other(
                "db down".to_string()
            ))),
            "保存エラー"
        );
        assert_eq!(short_failure_reason(&ApiError::NotFound), "取得エラー");
    }

    #[test]
    fn scheduled_time_ms_is_shared_by_both_schedules() {
        let epoch = scheduled_time_ms(0.0).expect("epoch");
        assert_eq!(epoch.unix_timestamp(), 0);
        assert_eq!(scheduled_time_ms(f64::NAN), Some(epoch));
        assert!(scheduled_time_ms(f64::INFINITY).is_none());
        assert!(scheduled_time_ms(f64::NEG_INFINITY).is_none());
        assert!(scheduled_time_ms(-1.0).is_some());
        assert!(crate::services::stock_price::should_refresh_at(f64::NAN));
        assert!(!crate::services::fund_price::should_fund_refresh_at(
            f64::NAN
        ));
        assert!(!crate::services::stock_price::should_refresh_at(
            f64::INFINITY
        ));
        assert!(!crate::services::fund_price::should_fund_refresh_at(
            f64::INFINITY
        ));
    }

    #[test]
    fn none_succeeded_matches_both_all_failed_branches() {
        assert!(!none_succeeded(0, 0));
        assert!(none_succeeded(3, 0));
        assert!(!none_succeeded(3, 1));
        assert!(!none_succeeded(3, 3));
    }
}
