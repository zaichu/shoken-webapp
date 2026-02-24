use crate::errors::ApiError;
use crate::models::dividend_cache::{DividendCache, DividendPerShareItem, CACHE_TTL_DAYS};
use crate::models::jquants::FinSummaryData;
use crate::services::jquants::JQuantsService;
use chrono::Utc;
use reqwest::Client;
use sqlx::PgPool;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::Duration;

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

    // DB からキャッシュを一括取得
    let cached: Vec<DividendCache> = sqlx::query_as::<_, DividendCache>(
        r#"
        SELECT security_code, dividend_per_share, status, fetched_at, source,
               created_at, updated_at
        FROM jquants_dividend_cache
        WHERE security_code = ANY($1)
        "#,
    )
    .bind(codes)
    .fetch_all(pool)
    .await?;

    let now = Utc::now();
    let ttl_threshold = now - chrono::Duration::days(CACHE_TTL_DAYS);

    // コードをキーにしてキャッシュをマップ化
    let cache_map: std::collections::HashMap<&str, &DividendCache> = cached
        .iter()
        .map(|c| (c.security_code.as_str(), c))
        .collect();

    // 未キャッシュ or TTL切れのコードを収集（バックグラウンド更新対象）
    let mut refresh_codes: Vec<String> = Vec::new();

    let items: Vec<DividendPerShareItem> = codes
        .iter()
        .map(|code| {
            if let Some(cached_item) = cache_map.get(code.as_str()) {
                let is_stale = cached_item
                    .fetched_at
                    .map(|t| t < ttl_threshold)
                    .unwrap_or(true);
                if is_stale && cached_item.status != "pending" {
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
                refresh_codes.push(code.clone());
                DividendPerShareItem {
                    security_code: code.clone(),
                    dividend_per_share: None,
                    status: "pending".to_string(),
                    fetched_at: None,
                    is_stale: false,
                }
            }
        })
        .collect();

    // バックグラウンド更新をキック（多重起動防止）
    if !refresh_codes.is_empty() {
        if let Some(key) = api_key {
            spawn_background_refresh(
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

/// バックグラウンドで未取得/TTL切れ銘柄を順次更新する（1分5回レート制御）
fn spawn_background_refresh(
    pool: PgPool,
    client: Client,
    api_key: String,
    codes: Vec<String>,
    running: Arc<AtomicBool>,
) {
    // 多重起動防止: false → true の比較交換に成功した場合のみ実行
    if running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        tracing::debug!("バックグラウンド更新タスクはすでに実行中");
        return;
    }

    tokio::spawn(async move {
        // タスク完了時（パニック含む）に必ずフラグをリセット
        struct Guard(Arc<AtomicBool>);
        impl Drop for Guard {
            fn drop(&mut self) {
                self.0.store(false, Ordering::SeqCst);
            }
        }
        let _guard = Guard(Arc::clone(&running));

        tracing::info!(
            "配当キャッシュ バックグラウンド更新開始: {}銘柄",
            codes.len()
        );

        for code in &codes {
            // DBレート制御: 1分5回(12秒間隔)を全インスタンスで保証
            match acquire_rate_slot(&pool).await {
                Ok(()) => {}
                Err(e) => {
                    tracing::error!("レート制御スロット取得エラー: {}", e);
                    break;
                }
            }

            match fetch_and_cache(&pool, &client, &api_key, code).await {
                Ok(status) => {
                    tracing::info!("配当キャッシュ更新完了: code={}, status={}", code, status);
                }
                Err(e) => {
                    tracing::error!("配当キャッシュ更新エラー: code={}, err={}", code, e);
                    // エラーをキャッシュに記録
                    let _ = update_cache_error(&pool, code, &e.to_string()).await;
                }
            }
        }

        tracing::info!("配当キャッシュ バックグラウンド更新完了");
    });
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

/// JQuants API から取得してキャッシュを更新する
async fn fetch_and_cache(
    pool: &PgPool,
    client: &Client,
    api_key: &str,
    code: &str,
) -> Result<String, ApiError> {
    use crate::models::jquants::FinSummaryQuery;

    let params = FinSummaryQuery {
        code: code.to_string(),
        from: None,
        to: None,
    };

    let response = JQuantsService::get_fin_summary(client, params, api_key).await?;

    let (dividend_per_share, status) = extract_dividend(&response.data);

    sqlx::query(
        r#"
        INSERT INTO jquants_dividend_cache
            (security_code, dividend_per_share, status, fetched_at, source, updated_at)
        VALUES ($1, $2, $3, NOW(), 'jquants', NOW())
        ON CONFLICT (security_code) DO UPDATE
            SET dividend_per_share = EXCLUDED.dividend_per_share,
                status             = EXCLUDED.status,
                fetched_at         = EXCLUDED.fetched_at,
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
async fn update_cache_error(pool: &PgPool, code: &str, error_msg: &str) -> Result<(), ApiError> {
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
            (security_code, dividend_per_share, status, error_message, source, updated_at)
        VALUES ($1, NULL, 'error', $2, 'jquants', NOW())
        ON CONFLICT (security_code) DO UPDATE
            SET dividend_per_share = NULL,
                status             = 'error',
                error_message      = EXCLUDED.error_message,
                updated_at         = NOW()
        "#,
    )
    .bind(code)
    .bind(truncated)
    .execute(pool)
    .await?;

    Ok(())
}

/// 決算サマリーから1株配当を抽出する
/// 優先順位: 来期予想(NxFDivAnn) > 今期予想(FDivAnn) > 実績(DivAnn)
fn extract_dividend(data: &[FinSummaryData]) -> (Option<f64>, String) {
    // 開示日で降順ソート（最新データを優先）
    let mut sorted: Vec<&FinSummaryData> = data.iter().collect();
    sorted.sort_by(|a, b| b.disclosed_date.cmp(&a.disclosed_date));

    for summary in &sorted {
        let raw = summary
            .next_year_forecast_dividend_per_share_annual
            .as_deref()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                summary
                    .forecast_dividend_per_share_annual
                    .as_deref()
                    .filter(|s| !s.is_empty())
            })
            .or_else(|| {
                summary
                    .result_dividend_per_share_annual
                    .as_deref()
                    .filter(|s| !s.is_empty())
            });

        if let Some(val_str) = raw {
            if let Ok(val) = val_str.parse::<f64>() {
                let status = if val > 0.0 { "ok" } else { "zero" };
                return (Some(val), status.to_string());
            }
        }
    }

    // 配当情報が見つからない → ゼロ配当として扱う
    (Some(0.0), "zero".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::jquants::FinSummaryData;

    fn make_summary(
        disc_date: &str,
        nx_div: Option<&str>,
        f_div: Option<&str>,
        div: Option<&str>,
    ) -> FinSummaryData {
        // FinSummaryData のデフォルト値を生成するためにデシリアライズを利用
        let json = serde_json::json!({
            "DiscDate": disc_date,
            "Code": "1234",
            "DocType": "test",
            "NxFDivAnn": nx_div,
            "FDivAnn": f_div,
            "DivAnn": div,
        });
        serde_json::from_value(json).expect("FinSummaryData のパースに失敗")
    }

    #[test]
    fn test_extract_dividend_ok() {
        let data = vec![make_summary("2024-01-01", Some("100.0"), None, None)];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(100.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_extract_dividend_zero() {
        let data = vec![make_summary("2024-01-01", Some("0.0"), None, None)];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(0.0));
        assert_eq!(status, "zero");
    }

    #[test]
    fn test_extract_dividend_priority_nx_over_f() {
        // NxFDivAnn が優先
        let data = vec![make_summary(
            "2024-01-01",
            Some("200.0"),
            Some("100.0"),
            Some("50.0"),
        )];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(200.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_extract_dividend_falls_back_to_result() {
        // NxFDivAnn, FDivAnn が None → DivAnn を使用
        let data = vec![make_summary("2024-01-01", None, None, Some("75.0"))];
        let (val, status) = extract_dividend(&data);
        assert_eq!(val, Some(75.0));
        assert_eq!(status, "ok");
    }

    #[test]
    fn test_extract_dividend_empty_data() {
        // データなし → ゼロ配当
        let (val, status) = extract_dividend(&[]);
        assert_eq!(val, Some(0.0));
        assert_eq!(status, "zero");
    }

    #[test]
    fn test_extract_dividend_newest_first() {
        // 開示日が新しいほうを優先
        let data = vec![
            make_summary("2023-01-01", None, None, Some("30.0")),
            make_summary("2024-01-01", None, None, Some("60.0")),
        ];
        let (val, _) = extract_dividend(&data);
        assert_eq!(val, Some(60.0));
    }

    #[test]
    fn test_cache_ttl_constant() {
        assert_eq!(CACHE_TTL_DAYS, 7);
    }
}
