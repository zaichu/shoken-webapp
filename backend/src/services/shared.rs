use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use sqlx::PgPool;
use std::time::Instant;
use tracing::info;
use uuid::Uuid;

/// bulk_create の開始ログ・完了ログ・処理時間計測をまとめた補助構造体
pub struct BulkTimer {
    domain: &'static str,
    start: Instant,
    total: usize,
}

impl BulkTimer {
    pub fn new(domain: &'static str, total: usize) -> Self {
        info!("[{}.bulk_create] リクエスト受信: {}件", domain, total);
        Self {
            domain,
            start: Instant::now(),
            total,
        }
    }

    pub fn finish(self, inserted: usize) -> BulkCreateResponse {
        let skipped = self.total - inserted;
        let elapsed_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        info!(
            "[{}.bulk_create] 完了: inserted={}, skipped={}, 処理時間={:.2}ms",
            self.domain, inserted, skipped, elapsed_ms
        );
        BulkCreateResponse { inserted, skipped }
    }
}

/// ユーザーに紐づく全レコードを削除する共通実装。
/// テーブル名は呼び出し元がリテラルで指定するため SQL インジェクションの危険はない。
pub async fn delete_all_for_user(
    pool: &PgPool,
    user_id: Uuid,
    table_name: &'static str,
    domain: &'static str,
) -> Result<u64, ApiError> {
    info!("[{}.delete_all] リクエスト受信", domain);
    let result = sqlx::query(&format!("DELETE FROM {} WHERE user_id = $1", table_name))
        .bind(user_id)
        .execute(pool)
        .await?;
    let deleted = result.rows_affected();
    info!("[{}.delete_all] 完了: {}件削除", domain, deleted);
    Ok(deleted)
}

#[cfg(test)] #[rustfmt::skip] mod tests {
    use super::BulkTimer;
    #[test]
    fn test_bulk_timer_finish() { for (inserted, expected) in [(0, (0, 5)), (5, (5, 0)), (3, (3, 2))] { let response = BulkTimer::new("test", 5).finish(inserted); assert_eq!((response.inserted, response.skipped), expected); } }
}
