// tokio::spawn ベースのバックグラウンド更新は isolate にスレッドを持てない wasm では
// コンパイルしない。wasm 側は get_batch が pending 行を積み、cron が消化する
#[cfg(not(target_arch = "wasm32"))]
mod background;
mod logic;
// tests/db_integration.rs からの検証用に公開しているため docs には出さない
#[doc(hidden)]
pub mod persistence;

use crate::db::{Bind, Db};
use crate::errors::{ApiError, UpstreamError};
use crate::models::dividend_cache::{DividendCache, DividendPerShareItem};
use crate::services::jquants::JQuantsClient;
use chrono::Utc;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::Client;
use shared::dividend_per_share::DividendCacheStatus;
use shared::value::SecurityCode;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, atomic::AtomicBool};
use std::time::Duration;

use logic::compute_is_stale;
use persistence::{fetch_and_cache, update_cache_error, update_cache_error_with_cooldown};

/// 429 発生時の全インスタンス共有 cooldown 期間（秒）
const RATE_LIMIT_COOLDOWN_SECS: i32 = 60;

/// market_data_provider_rate_control.provider の J-Quants 用キー
const JQUANTS_PROVIDER: &str = "jquants";

/// scheduled イベント1回の実行で消化する銘柄数の上限。
/// 12秒間隔スロットで最大約4銘柄/分（設計 §8-4）。残りは次回の cron に委ねる
const MAX_DRAIN_CODES_PER_RUN: i64 = 4;

/// 429 レートリミットエラーの場合に消化を打ち切るべきか判定する
pub(crate) fn should_abort_on_error(e: &ApiError) -> bool {
    matches!(e, ApiError::Upstream(UpstreamError::RateLimited))
}

/// キャッシュをバッチ取得し、未取得/TTL切れ銘柄のバックグラウンド更新をキック
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_batch(
    pool: &Db,
    client: &Client,
    api_key: Option<&str>,
    codes: &[SecurityCode],
    background_task_running: &Arc<AtomicBool>,
) -> Result<Vec<DividendPerShareItem>, ApiError> {
    let (items, refresh_codes) = fetch_cached_items(pool, codes).await?;

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

/// Workers 版: isolate 内で spawn できないため、未取得/TTL切れ銘柄を pending 行として
/// DB に積み、scheduled イベント（毎分 cron）が drain_refresh_queue で消化する
#[cfg(target_arch = "wasm32")]
pub async fn get_batch(
    pool: &Db,
    jquants_client: Option<&JQuantsClient>,
    codes: &[SecurityCode],
) -> Result<Vec<DividendPerShareItem>, ApiError> {
    let (items, refresh_codes) = fetch_cached_items(pool, codes).await?;

    if !refresh_codes.is_empty() {
        if jquants_client.is_some() {
            enqueue_refresh(pool, &refresh_codes).await?;
        } else {
            tracing::warn!("JQUANTS_API_KEY が未設定のため更新キュー投入をスキップ");
        }
    }

    Ok(items)
}

/// 未取得銘柄を pending 行として積む。
/// 既存行は stale_at / cooldown 経過で cron の選定条件を満たすため、そのままにする
/// （status が pending の間は compute_is_stale=false で再投入も起きない）
#[cfg(target_arch = "wasm32")]
async fn enqueue_refresh(pool: &Db, codes: &[SecurityCode]) -> Result<(), ApiError> {
    crate::db::query(
        r#"
        INSERT INTO dividend_per_share_cache (security_code, status, provider, updated_at)
        SELECT code, 'pending', 'jquants', NOW()
        FROM unnest($1::varchar[]) AS code
        ON CONFLICT (security_code) DO NOTHING
        "#,
        vec![Bind::from(codes.to_vec())],
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn fetch_cached_items(
    pool: &Db,
    codes: &[SecurityCode],
) -> Result<(Vec<DividendPerShareItem>, Vec<SecurityCode>), ApiError> {
    if codes.is_empty() {
        return Ok((vec![], vec![]));
    }

    // DB からキャッシュを一括取得（ANY がDB側で重複を除く）
    let cached = crate::db::query_as::<DividendCache>(
        r#"
        SELECT security_code, dividend_per_share, status,
               fetched_at, stale_at, provider, created_at, updated_at
        FROM dividend_per_share_cache
        WHERE security_code = ANY($1)
        "#,
        vec![Bind::from(codes.to_vec())],
    )
    .fetch_all(pool)
    .await?;

    let now = Utc::now();

    let cache_map: std::collections::HashMap<&str, &DividendCache> = cached
        .iter()
        .map(|c| (c.security_code.as_str(), c))
        .collect();

    Ok(build_batch_items(codes, &cache_map, now))
}

/// scheduled イベントから stale/pending 銘柄を消化する。
/// レート制御は market_data_provider_rate_control で全インスタンス共有されるため、
/// 他経路のバックグラウンド更新と並走しても 12 秒間隔は破られない。
/// 中断規則は background refresh と同じ（429→cooldown+中断、スロット失敗→中断）。
/// 戻り値は今回の実行で実際に取得を試みた件数（ログ・テスト用）
pub async fn drain_refresh_queue(
    pool: &Db,
    jquants_client: &JQuantsClient,
) -> Result<usize, ApiError> {
    // status='pending' は stale_at が NULL、error(無cooldown) も NULL で拾われる。
    // 429 cooldown 中（error + 未来の stale_at）は対象外
    let codes: Vec<SecurityCode> = crate::db::query(
        r#"
        SELECT security_code FROM dividend_per_share_cache
        WHERE status = 'pending' OR stale_at IS NULL OR stale_at < NOW()
        ORDER BY stale_at ASC NULLS FIRST, updated_at ASC
        LIMIT $1
        "#,
        vec![Bind::from(MAX_DRAIN_CODES_PER_RUN)],
    )
    .fetch_all::<(SecurityCode,), _>(pool)
    .await?
    .into_iter()
    .map(|(code,)| code)
    .collect();

    let mut processed = 0;
    for code in codes {
        if let Err(e) = acquire_rate_slot(pool).await {
            tracing::error!("レート制御スロット取得エラー: {}", e);
            break;
        }
        match fetch_and_cache(pool, jquants_client, &code).await {
            Ok(status) => {
                processed += 1;
                tracing::info!("配当キャッシュ更新完了: code={}, status={}", code, status);
            }
            Err(e) if should_abort_on_error(&e) => {
                tracing::warn!(
                    "配当キャッシュ更新: レートリミット超過 code={}, 消化を中断",
                    code
                );
                if let Err(err) = update_cache_error_with_cooldown(
                    pool,
                    &code,
                    &e.to_string(),
                    RATE_LIMIT_COOLDOWN_SECS,
                )
                .await
                {
                    tracing::error!("配当キャッシュ エラー記録失敗: code={}, err={}", code, err);
                }
                if let Err(err) = push_rate_control_cooldown(pool).await {
                    tracing::error!("レートリミット cooldown 設定失敗: err={}", err);
                }
                break;
            }
            Err(e) => {
                processed += 1;
                tracing::error!("配当キャッシュ更新エラー: code={}, err={}", code, e);
                let _ = update_cache_error(pool, &code, &e.to_string()).await;
            }
        }
    }
    Ok(processed)
}

#[allow(clippy::needless_lifetimes)]
fn build_batch_items<'a>(
    codes: &'a [SecurityCode],
    cache_map: &std::collections::HashMap<&str, &DividendCache>,
    now: chrono::DateTime<chrono::Utc>,
) -> (Vec<DividendPerShareItem>, Vec<SecurityCode>) {
    // items は元の codes 順で構築し API の返却件数を維持する
    // refresh_codes は初出現順を保持しつつ重複を除去する（更新優先度順を維持するため）
    let mut refresh_seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut refresh_codes: Vec<SecurityCode> = Vec::new();
    let mut items: Vec<DividendPerShareItem> = Vec::with_capacity(codes.len());

    for code in codes {
        let cached_entry = cache_map.get(code.as_str());
        let is_stale = cached_entry.is_none_or(|c| compute_is_stale(c.status, c.stale_at, now));
        if is_stale && refresh_seen.insert(code.as_str()) {
            refresh_codes.push(code.clone());
        }
        let item = DividendPerShareItem {
            security_code: code.clone(),
            dividend_per_share: cached_entry.and_then(|c| c.dividend_per_share),
            status: cached_entry.map_or(DividendCacheStatus::Pending, |c| c.status),
            fetched_at: cached_entry.and_then(|c| c.fetched_at),
            is_stale: cached_entry.is_some_and(|_| is_stale),
        };
        items.push(item);
    }

    (items, refresh_codes)
}

/// DBレート制御テーブルを使って次の実行スロットを予約し、必要なら待機する
/// 1リクエスト = 12秒間隔（60秒 / 5回）を全インスタンスで原子的に保証
pub async fn acquire_rate_slot(pool: &Db) -> Result<(), ApiError> {
    // UPSERT でスロットを予約し、前のスロット開始時刻を返す
    let slot_time = crate::db::query_scalar::<Option<chrono::DateTime<Utc>>>(
        r#"
        INSERT INTO market_data_provider_rate_control (provider, next_available_at)
            VALUES ($1, NOW() + INTERVAL '12 seconds')
        ON CONFLICT (provider) DO UPDATE
            SET next_available_at =
                GREATEST(market_data_provider_rate_control.next_available_at, NOW()) + INTERVAL '12 seconds'
        RETURNING
            GREATEST(next_available_at - INTERVAL '12 seconds', NOW() - INTERVAL '1 second')
        "#,
        vec![Bind::from(JQUANTS_PROVIDER)],
    )
    .fetch_one(pool)
    .await?;

    if let Some(slot_time) = slot_time {
        let now = Utc::now();
        if slot_time > now {
            let wait_ms = (slot_time - now).num_milliseconds().max(0) as u64;
            if wait_ms > 0 {
                tracing::debug!("レート制御: {}ms 待機", wait_ms);
                sleep_millis(wait_ms).await;
            }
        }
    }

    Ok(())
}

/// native は tokio タイマー、wasm は worker::Delay（ランタイム提供のタイマー）で待機する
async fn sleep_millis(ms: u64) {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(Duration::from_millis(ms)).await;
    #[cfg(target_arch = "wasm32")]
    worker::Delay::from(Duration::from_millis(ms)).await;
}

/// 429 発生時に market_data_provider_rate_control.next_available_at を少なくとも cooldown 分先へ延ばす
/// 既存の future 値がある場合は後退させず、GREATEST で大きい方を維持する
async fn push_rate_control_cooldown(pool: &Db) -> Result<(), ApiError> {
    crate::db::query(
        r#"
        INSERT INTO market_data_provider_rate_control (provider, next_available_at)
            VALUES ($1, NOW() + $2::int4 * INTERVAL '1 second')
        ON CONFLICT (provider) DO UPDATE
            SET next_available_at =
                GREATEST(market_data_provider_rate_control.next_available_at, NOW() + $2::int4 * INTERVAL '1 second')
        "#,
        vec![
            Bind::from(JQUANTS_PROVIDER),
            Bind::from(RATE_LIMIT_COOLDOWN_SECS),
        ],
    )
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
        status: DividendCacheStatus,
        stale_at: Option<chrono::DateTime<Utc>>,
    ) -> DividendCache {
        DividendCache {
            security_code: code.parse().unwrap(),
            dividend_per_share: Some(30.0),
            status,
            fetched_at: Some(Utc::now()),
            stale_at,
            provider: "jquants".to_string(),
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

    fn code(s: &str) -> SecurityCode {
        s.parse().unwrap()
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
        let codes = vec![code("1234")];
        let cache_map = std::collections::HashMap::new();

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, fixed_now());

        assert_eq!(refresh_codes, vec![code("1234")]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].security_code.as_str(), "1234");
        assert_eq!(items[0].status, DividendCacheStatus::Pending);
        assert_eq!(items[0].dividend_per_share, None);
        assert!(!items[0].is_stale);
    }

    #[test]
    fn test_cached_fresh() {
        let now = fixed_now();
        let caches = vec![make_cache(
            "1234",
            DividendCacheStatus::Ok,
            Some(now + Duration::days(1)),
        )];
        let cache_map = make_cache_map(&caches);
        let codes = vec![code("1234")];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert!(refresh_codes.is_empty());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, DividendCacheStatus::Ok);
        assert!(!items[0].is_stale);
    }

    #[test]
    fn test_cached_stale() {
        let now = fixed_now();
        let caches = vec![make_cache(
            "1234",
            DividendCacheStatus::Ok,
            Some(now - Duration::days(1)),
        )];
        let cache_map = make_cache_map(&caches);
        let codes = vec![code("1234")];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert_eq!(refresh_codes, vec![code("1234")]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, DividendCacheStatus::Ok);
        assert!(items[0].is_stale);
    }

    #[test]
    fn test_cached_error() {
        let now = fixed_now();
        let caches = vec![make_cache("1234", DividendCacheStatus::Error, None)];
        let cache_map = make_cache_map(&caches);
        let codes = vec![code("1234")];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert_eq!(refresh_codes, vec![code("1234")]);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, DividendCacheStatus::Error);
        assert!(items[0].is_stale);
    }

    #[test]
    fn test_cached_error_with_future_stale_at_is_not_stale() {
        // 429 cooldown 中: status=error かつ stale_at が future の場合は
        // is_stale=false となり refresh_codes に入らない
        let now = fixed_now();
        let caches = vec![make_cache(
            "1234",
            DividendCacheStatus::Error,
            Some(now + Duration::hours(1)),
        )];
        let cache_map = make_cache_map(&caches);
        let codes = vec![code("1234")];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert!(refresh_codes.is_empty());
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, DividendCacheStatus::Error);
        assert!(!items[0].is_stale);
    }

    #[test]
    fn test_duplicate_codes() {
        let now = fixed_now();
        let caches = vec![make_cache(
            "1234",
            DividendCacheStatus::Ok,
            Some(now - Duration::days(1)),
        )];
        let cache_map = make_cache_map(&caches);
        let codes = vec![code("1234"), code("1234")];

        let (items, refresh_codes) = build_batch_items(&codes, &cache_map, now);

        assert_eq!(items.len(), 2);
        assert_eq!(refresh_codes, vec![code("1234")]);
    }

    #[test]
    fn test_order_preserved() {
        let now = fixed_now();
        let caches = vec![
            make_cache(
                "1111",
                DividendCacheStatus::Ok,
                Some(now + Duration::days(1)),
            ),
            make_cache(
                "2222",
                DividendCacheStatus::Ok,
                Some(now + Duration::days(1)),
            ),
            make_cache(
                "3333",
                DividendCacheStatus::Ok,
                Some(now + Duration::days(1)),
            ),
        ];
        let cache_map = make_cache_map(&caches);
        let codes = vec![code("3333"), code("1111"), code("2222")];

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
