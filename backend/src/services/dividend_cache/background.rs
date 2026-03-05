use reqwest::Client;
use sqlx::PgPool;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use super::acquire_rate_slot;
use super::persistence::{fetch_and_cache, update_cache_error};

/// バックグラウンドで未取得/TTL切れ銘柄を順次更新する（1分5回レート制御）
pub fn spawn_background_refresh(
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
