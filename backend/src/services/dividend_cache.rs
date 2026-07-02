mod background;
mod logic;
mod persistence;

use crate::errors::ApiError;
use crate::models::dividend_cache::{DividendCache, DividendPerShareItem};
use crate::services::jquants::JQuantsClient;
use chrono::Utc;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::time::Duration;

use logic::compute_is_stale;

/// 429 発生時の全インスタンス共有 cooldown 期間（秒）
const RATE_LIMIT_COOLDOWN_SECS: i32 = 60;

/// キャッシュをバッチ取得し、未取得/TTL切れ銘柄のバックグラウンド更新をキック
pub async fn get_batch(
    pool: &PgPool,
    client: &Client,
    api_key: Option<&str>,
    codes: &[String],
    background_task_running: &Arc<AtomicBool>,
) -> Result<Vec<DividendPerShareItem>, ApiError> {
    if codes.is_empty() {
        return Ok(vec![]);
    }

    // DB からキャッシュを一括取得（ANY がDB側で重複を除く）
    let cached: Vec<DividendCache> = sqlx::query_as::<_, DividendCache>(
        r#"
        SELECT security_code, dividend_per_share, status, fetched_at, stale_at, source,
               created_at, updated_at
        FROM dividend_per_share_cache
        WHERE security_code = ANY($1)
        "#,
    )
    .bind(codes)
    .fetch_all(pool)
    .await?;

    let now = Utc::now();

    // コードをキーにしてキャッシュをマップ化
    let cache_map: std::collections::HashMap<&str, &DividendCache> = cached
        .iter()
        .map(|c| (c.security_code.as_str(), c))
        .collect();

    let (items, refresh_codes) = build_batch_items(codes, &cache_map, now);

    // バックグラウンド更新をキック（多重起動防止）
    if !refresh_codes.is_empty() {
        if let Some(key) = api_key {
            let jquants_client = JQuantsClient::new(client.clone(), key.to_string());
            background::spawn_background_refresh(
                pool.clone(),
                jquants_client,
                refresh_codes,
                Arc::clone(background_task_running),
            );
        } else {
            tracing::warn!("JQUANTS_API_KEY が未設定のためバックグラウンド更新をスキップ");
        }
    }

    Ok(items)
}

#[allow(clippy::needless_lifetimes)]
fn build_batch_items<'a>(
    codes: &'a [String],
    cache_map: &std::collections::HashMap<&str, &DividendCache>,
    now: chrono::DateTime<chrono::Utc>,
) -> (Vec<DividendPerShareItem>, Vec<String>) {
    // items は元の codes 順で構築し API の返却件数を維持する
    // refresh_codes は初出現順を保持しつつ重複を除去する（更新優先度順を維持するため）
    let mut refresh_seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut refresh_codes: Vec<String> = Vec::new();
    let mut items: Vec<DividendPerShareItem> = Vec::with_capacity(codes.len());

    for code in codes {
        let cached_entry = cache_map.get(code.as_str());
        let is_stale = cached_entry.is_none_or(|c| compute_is_stale(&c.status, c.stale_at, now));
        if is_stale && refresh_seen.insert(code.as_str()) {
            refresh_codes.push(code.clone());
        }
        let item = DividendPerShareItem {
            security_code: code.clone(),
            dividend_per_share: cached_entry.and_then(|c| c.dividend_per_share),
            status: cached_entry.map_or_else(|| "pending".to_string(), |c| c.status.clone()),
            fetched_at: cached_entry.and_then(|c| c.fetched_at),
            is_stale: cached_entry.is_some() && is_stale,
        };
        items.push(item);
    }

    (items, refresh_codes)
}

/// DBレート制御テーブルを使って次の実行スロットを予約し、必要なら待機する
/// 1リクエスト = 12秒間隔（60秒 / 5回）を全インスタンスで原子的に保証
pub async fn acquire_rate_slot(pool: &PgPool) -> Result<(), ApiError> {
    // UPSERT でスロットを予約し、前のスロット開始時刻を返す
    let row: (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as(
        r#"
        INSERT INTO market_data_provider_rate_control (id, next_available_at)
            VALUES (1, NOW() + INTERVAL '12 seconds')
        ON CONFLICT (id) DO UPDATE
            SET next_available_at =
                GREATEST(market_data_provider_rate_control.next_available_at, NOW()) + INTERVAL '12 seconds'
        RETURNING
            GREATEST(next_available_at - INTERVAL '12 seconds', NOW() - INTERVAL '1 second')
        "#,
    )
    .fetch_one(pool)
    .await?;

    if let Some(slot_time) = row.0 {
        let now = Utc::now();
        if slot_time > now {
            let wait_ms = (slot_time - now).num_milliseconds().max(0) as u64;
            if wait_ms > 0 {
                tracing::debug!("レート制御: {}ms 待機", wait_ms);
                tokio::time::sleep(Duration::from_millis(wait_ms)).await;
            }
        }
    }

    Ok(())
}

/// 429 発生時に market_data_provider_rate_control.next_available_at を少なくとも cooldown 分先へ延ばす
/// 既存の future 値がある場合は後退させず、GREATEST で大きい方を維持する
async fn push_rate_control_cooldown(pool: &PgPool) -> Result<(), ApiError> {
    sqlx::query(
        r#"
        INSERT INTO market_data_provider_rate_control (id, next_available_at)
            VALUES (1, NOW() + $1 * INTERVAL '1 second')
        ON CONFLICT (id) DO UPDATE
            SET next_available_at =
                GREATEST(market_data_provider_rate_control.next_available_at, NOW() + $1 * INTERVAL '1 second')
        "#,
    )
    .bind(RATE_LIMIT_COOLDOWN_SECS)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    fn make_cache(
        code: &str,
        status: &str,
        stale_at: Option<chrono::DateTime<Utc>>,
    ) -> DividendCache {
        DividendCache {
            security_code: code.to_string(),
            dividend_per_share: Some(30.0),
            status: status.to_string(),
            fetched_at: Some(Utc::now()),
            stale_at,
            source: "jquants".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn make_cache_map(caches: &[DividendCache]) -> std::collections::HashMap<&str, &DividendCache> {
        caches
            .iter()
            .map(|cache| (cache.security_code.as_str(), cache))
            .collect()
    }

    fn fixed_now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2024, 1, 10, 0, 0, 0)
            .single()
            .expect("固定時刻の生成に失敗")
    }

    #[test]
    fn test_empty_codes() {
        let codes = Vec::new();
        let cache_map = std::collections::HashMap::new();

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, fixed_now());

        assert!(items.is_empty());
        assert!(refresh_codes.is_empty());
    }

    #[test]
    fn test_uncached_code() {
        let codes = vec!["1234".to_string()];
        let cache_map = std::collections::HashMap::new();

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, fixed_now());

        assert_eq!(refresh_codes, vec!["1234".to_string()]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].security_code, "1234");
        assert_eq!(items[0].status, "pending");
        assert_eq!(items[0].dividend_per_share, None);
        assert!(!items[0].is_stale);
    }

    #[test]
    fn test_cached_fresh() {
        let now = fixed_now();
        let caches = vec![make_cache("1234", "ok", Some(now + Duration::days(1)))];
        let cache_map = make_cache_map(&caches);
        let codes = vec!["1234".to_string()];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert!(refresh_codes.is_empty());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "ok");
        assert!(!items[0].is_stale);
    }

    #[test]
    fn test_cached_stale() {
        let now = fixed_now();
        let caches = vec![make_cache("1234", "ok", Some(now - Duration::days(1)))];
        let cache_map = make_cache_map(&caches);
        let codes = vec!["1234".to_string()];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert_eq!(refresh_codes, vec!["1234".to_string()]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "ok");
        assert!(items[0].is_stale);
    }

    #[test]
    fn test_cached_error() {
        let now = fixed_now();
        let caches = vec![make_cache("1234", "error", None)];
        let cache_map = make_cache_map(&caches);
        let codes = vec!["1234".to_string()];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert_eq!(refresh_codes, vec!["1234".to_string()]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "error");
        assert!(items[0].is_stale);
    }

    #[test]
    fn test_cached_error_with_future_stale_at_is_not_stale() {
        // 429 cooldown 中: status=error かつ stale_at が future の場合は
        // is_stale=false となり refresh_codes に入らない
        let now = fixed_now();
        let caches = vec![make_cache("1234", "error", Some(now + Duration::hours(1)))];
        let cache_map = make_cache_map(&caches);
        let codes = vec!["1234".to_string()];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert!(refresh_codes.is_empty());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "error");
        assert!(!items[0].is_stale);
    }

    #[test]
    fn test_duplicate_codes() {
        let now = fixed_now();
        let caches = vec![make_cache("1234", "ok", Some(now - Duration::days(1)))];
        let cache_map = make_cache_map(&caches);
        let codes = vec!["1234".to_string(), "1234".to_string()];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert_eq!(items.len(), 2);
        assert_eq!(refresh_codes, vec!["1234".to_string()]);
    }

    #[test]
    fn test_order_preserved() {
        let now = fixed_now();
        let caches = vec![
            make_cache("1111", "ok", Some(now + Duration::days(1))),
            make_cache("2222", "ok", Some(now + Duration::days(1))),
            make_cache("3333", "ok", Some(now + Duration::days(1))),
        ];
        let cache_map = make_cache_map(&caches);
        let codes = vec!["3333".to_string(), "1111".to_string(), "2222".to_string()];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert!(refresh_codes.is_empty());
        assert_eq!(
            items
                .iter()
                .map(|item| item.security_code.as_str())
                .collect::<Vec<_>>(),
            vec!["3333", "1111", "2222"]
        );
    }
}
