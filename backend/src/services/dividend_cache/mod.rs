mod background;
mod logic;
mod persistence;

use crate::errors::ApiError;
use crate::models::dividend_cache::{DividendCache, DividendPerShareItem};
use chrono::Utc;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::time::Duration;

use logic::compute_is_stale;

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
        FROM jquants_dividend_cache
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

    // items は元の codes 順で構築し API の返却件数を維持する
    // refresh_codes は初出現順を保持しつつ重複を除去する（更新優先度順を維持するため）
    let mut refresh_seen: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut refresh_codes: Vec<String> = Vec::new();
    let mut items: Vec<DividendPerShareItem> = Vec::with_capacity(codes.len());

    for code in codes {
        let item = if let Some(cached_item) = cache_map.get(code.as_str()) {
            let is_stale = compute_is_stale(&cached_item.status, cached_item.stale_at, now);
            if is_stale && refresh_seen.insert(code.as_str()) {
                refresh_codes.push(code.clone());
            }
            DividendPerShareItem {
                security_code: code.clone(),
                dividend_per_share: cached_item.dividend_per_share,
                status: cached_item.status.clone(),
                fetched_at: cached_item.fetched_at,
                is_stale,
            }
        } else {
            // 未キャッシュ → pending としてキューに積む
            if refresh_seen.insert(code.as_str()) {
                refresh_codes.push(code.clone());
            }
            DividendPerShareItem {
                security_code: code.clone(),
                dividend_per_share: None,
                status: "pending".to_string(),
                fetched_at: None,
                is_stale: false,
            }
        };
        items.push(item);
    }

    // バックグラウンド更新をキック（多重起動防止）
    if !refresh_codes.is_empty() {
        if let Some(key) = api_key {
            background::spawn_background_refresh(
                pool.clone(),
                client.clone(),
                key.to_string(),
                refresh_codes,
                Arc::clone(background_task_running),
            );
        } else {
            tracing::warn!("JQUANTS_API_KEY が未設定のためバックグラウンド更新をスキップ");
        }
    }

    Ok(items)
}

/// DBレート制御テーブルを使って次の実行スロットを予約し、必要なら待機する
/// 1リクエスト = 12秒間隔（60秒 / 5回）を全インスタンスで原子的に保証
pub async fn acquire_rate_slot(pool: &PgPool) -> Result<(), ApiError> {
    // UPSERT でスロットを予約し、前のスロット開始時刻を返す
    let row: (Option<chrono::DateTime<chrono::Utc>>,) = sqlx::query_as(
        r#"
        INSERT INTO jquants_rate_control (id, next_available_at)
            VALUES (1, NOW() + INTERVAL '12 seconds')
        ON CONFLICT (id) DO UPDATE
            SET next_available_at =
                GREATEST(jquants_rate_control.next_available_at, NOW()) + INTERVAL '12 seconds'
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
