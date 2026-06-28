use crate::errors::ApiError;
use crate::services::jquants::JQuantsClient;
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

/// バックグラウンドで未取得/TTL切れ銘柄を順次更新する（1分5回レート制御）
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

        for code in &codes {
            // DBレート制御: 1分5回(12秒間隔)を全インスタンスで保証
            if let Err(e) = acquire_rate_slot(&pool).await {
                tracing::error!("レート制御スロット取得エラー: {}", e);
                break;
            }

            match fetch_and_cache(&pool, &jquants_client, code).await {
                Ok(status) => {
                    tracing::info!("配当キャッシュ更新完了: code={}, status={}", code, status);
                }
                Err(e) => {
                    if should_abort_on_error(&e) {
                        tracing::warn!(
                            "配当キャッシュ更新: レートリミット超過 code={}, バックグラウンド更新を中断",
                            code
                        );
                        let _ = update_cache_error_with_cooldown(
                            &pool,
                            code,
                            &e.to_string(),
                            super::RATE_LIMIT_COOLDOWN_SECS,
                        )
                        .await;
                        let _ = super::push_rate_control_cooldown(&pool).await;
                        break;
                    }
                    tracing::error!("配当キャッシュ更新エラー: code={}, err={}", code, e);
                    let _ = update_cache_error(&pool, code, &e.to_string()).await;
                }
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
