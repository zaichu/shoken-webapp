use crate::errors::ApiError;
use crate::services::market_data::providers::jquants::JQuantsClient;
use futures::stream::{FuturesUnordered, StreamExt};
use sqlx::PgPool;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use super::acquire_rate_slot;
use super::persistence::{fetch_and_cache, update_cache_error, update_cache_error_with_cooldown};

/// 429 レートリミットエラーの場合に background refresh を打ち切るべきか判定する
pub(crate) fn should_abort_on_error(e: &ApiError) -> bool {
    matches!(e, ApiError::RateLimitError(_))
}

/// DB レート制御が 12秒間隔を保証するため、5 は同時に予約/待機させる上限であり、
/// 外部 API の実呼び出し間隔は acquire_rate_slot が制御する
const MAX_CONCURRENT_REFRESHES: usize = 5;

enum RefreshOutcome {
    Fetched(String),
    /// acquire_rate_slot の失敗 → 全体中断
    RateSlotFailed(ApiError),
    /// 429 → cooldown 設定 + 全体中断
    RateLimitExceeded(ApiError),
    /// その他の fetch エラー → エラー記録して継続
    FetchFailed(ApiError),
}

/// バックグラウンドで未取得/TTL切れ銘柄を並列更新する（最大 MAX_CONCURRENT_REFRESHES 件同時、1分5回レート制御）
pub fn spawn_background_refresh(
    pool: PgPool,
    jquants_client: JQuantsClient,
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

        let mut codes_iter = codes.iter();
        let mut pending: FuturesUnordered<_> = FuturesUnordered::new();

        loop {
            // バッチ単位でエンキュー: abort-worthy な結果を含む現バッチを
            // すべて処理し終えてから次バッチを enqueue する
            for code in codes_iter.by_ref().take(MAX_CONCURRENT_REFRESHES) {
                let pool = pool.clone();
                let client = jquants_client.clone();
                let code = code.clone();
                pending.push(async move {
                    // DBレート制御: 1分5回(12秒間隔)を全インスタンスで保証してから API を呼ぶ
                    if let Err(e) = acquire_rate_slot(&pool).await {
                        return (code, RefreshOutcome::RateSlotFailed(e));
                    }
                    let outcome = match fetch_and_cache(&pool, &client, &code).await {
                        Ok(status) => RefreshOutcome::Fetched(status),
                        Err(e) if should_abort_on_error(&e) => RefreshOutcome::RateLimitExceeded(e),
                        Err(e) => RefreshOutcome::FetchFailed(e),
                    };
                    (code, outcome)
                });
            }

            // コードが尽きて pending も空なら完了
            if pending.is_empty() {
                break;
            }

            // 現バッチを全件処理してから次バッチへ
            let mut abort = false;
            while let Some((code, outcome)) = pending.next().await {
                match outcome {
                    RefreshOutcome::Fetched(status) => {
                        tracing::info!("配当キャッシュ更新完了: code={}, status={}", code, status);
                    }
                    RefreshOutcome::RateSlotFailed(e) => {
                        tracing::error!("レート制御スロット取得エラー: {}", e);
                        // pending の future を drop して中断（残タスクはキャンセル）
                        abort = true;
                        break;
                    }
                    RefreshOutcome::RateLimitExceeded(e) => {
                        tracing::warn!(
                            "配当キャッシュ更新: レートリミット超過 code={}, バックグラウンド更新を中断",
                            code
                        );
                        if let Err(err) = update_cache_error_with_cooldown(
                            &pool,
                            &code,
                            &e.to_string(),
                            super::RATE_LIMIT_COOLDOWN_SECS,
                        )
                        .await
                        {
                            tracing::error!(
                                "配当キャッシュ エラー記録失敗: code={}, err={}",
                                code,
                                err
                            );
                        }
                        if let Err(err) = super::push_rate_control_cooldown(&pool).await {
                            tracing::error!("レートリミット cooldown 設定失敗: err={}", err);
                        }
                        // pending の future を drop して中断（残タスクはキャンセル）
                        abort = true;
                        break;
                    }
                    RefreshOutcome::FetchFailed(e) => {
                        tracing::error!("配当キャッシュ更新エラー: code={}, err={}", code, e);
                        let _ = update_cache_error(&pool, &code, &e.to_string()).await;
                    }
                }
            }

            if abort {
                // ループを抜けると pending が drop され、残タスクはキャンセルされる
                break;
            }
        }

        tracing::info!("配当キャッシュ バックグラウンド更新完了");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[test]
    fn test_should_abort_on_rate_limit_error() {
        let e = ApiError::RateLimitError("429 Too Many Requests".to_string());
        assert!(should_abort_on_error(&e));
    }

    #[test]
    fn test_should_not_abort_on_other_api_error() {
        let e = ApiError::ApiError("500 Internal Server Error".to_string());
        assert!(!should_abort_on_error(&e));
    }

    #[test]
    fn test_should_not_abort_on_network_error() {
        let e = ApiError::NetworkError("connection refused".to_string());
        assert!(!should_abort_on_error(&e));
    }

    #[tokio::test]
    async fn test_spawn_skips_if_already_running() {
        let pool =
            crate::db::connect_pool_lazy("postgresql://postgres:postgres@localhost/postgres", 1)
                .unwrap();
        let running = Arc::new(AtomicBool::new(true));

        spawn_background_refresh(
            pool,
            JQuantsClient::new(Client::new(), "dummy_key".to_string()),
            vec!["1234".to_string()],
            Arc::clone(&running),
        );

        // 既に実行中のためタスクを起動せず、フラグは true のまま
        assert!(running.load(Ordering::SeqCst));
    }
}
