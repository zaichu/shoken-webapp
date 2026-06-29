use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use sqlx::postgres::PgQueryResult;
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

    /// 空配列の場合は Err(即時レスポンス) を返し、非空なら Ok(タイマー) を返す。
    pub fn new_with_guard<T>(
        domain: &'static str,
        items: &[T],
    ) -> Result<Self, BulkCreateResponse> {
        let timer = Self::new(domain, items.len());
        if items.is_empty() {
            Err(timer.finish(0))
        } else {
            Ok(timer)
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

    /// PgQueryResult から rows_affected を取り出して finish する。
    pub fn finish_from_result(self, result: PgQueryResult) -> BulkCreateResponse {
        self.finish(result.rows_affected() as usize)
    }
}

/// bulk insert の UNNEST に渡す user_id 配列を生成する。
pub fn user_ids_for_bulk_insert(user_id: Uuid, total: usize) -> Vec<Uuid> {
    vec![user_id; total]
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeleteTarget {
    AssetBalances,
    Dividends,
    DomesticStocks,
    MutualFunds,
}

impl DeleteTarget {
    fn sql(self) -> &'static str {
        match self {
            Self::AssetBalances => "DELETE FROM asset_balances WHERE user_id = $1",
            Self::Dividends => "DELETE FROM dividends WHERE user_id = $1",
            Self::DomesticStocks => "DELETE FROM domestic_stocks WHERE user_id = $1",
            Self::MutualFunds => "DELETE FROM mutualfunds WHERE user_id = $1",
        }
    }

    fn domain(self) -> &'static str {
        match self {
            Self::AssetBalances => "asset_balance",
            Self::Dividends => "dividend",
            Self::DomesticStocks => "domestic_stock",
            Self::MutualFunds => "mutualfund",
        }
    }
}

/// ユーザーに紐づく全レコードを削除する共通実装。
pub async fn delete_all_for_user(
    pool: &PgPool,
    user_id: Uuid,
    target: DeleteTarget,
) -> Result<u64, ApiError> {
    let domain = target.domain();
    info!("[{}.delete_all] リクエスト受信", domain);
    let result = sqlx::query(target.sql())
        .bind(user_id)
        .execute(pool)
        .await?;
    let deleted = result.rows_affected();
    info!("[{}.delete_all] 完了: {}件削除", domain, deleted);
    Ok(deleted)
}
#[cfg(test)]
mod tests {
    use super::{user_ids_for_bulk_insert, BulkTimer};
    use uuid::Uuid;

    #[test]
    fn test_bulk_timer_finish() {
        for (inserted, expected) in [(0, (0, 5)), (5, (5, 0)), (3, (3, 2))] {
            let response = BulkTimer::new("test", 5).finish(inserted);
            assert_eq!((response.inserted, response.skipped), expected);
        }
    }

    #[test]
    fn test_user_ids_for_bulk_insert() {
        let id = Uuid::new_v4();
        let result = user_ids_for_bulk_insert(id, 3);
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|&v| v == id));

        assert!(user_ids_for_bulk_insert(id, 0).is_empty());
    }
}
