use super::{Domain, WriteMode};
use crate::db::{Bind, Db, Executor, QueryBuilder, Tx};
use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use shared::value::UserId;
use std::fmt;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
use tracing::info;

/// 1ユーザーあたりの保存行数上限（USER_ROW_LIMIT 由来）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowLimit(i64);

impl RowLimit {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub fn get(self) -> i64 {
        self.0
    }
}

impl fmt::Display for RowLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// bulk_create の開始ログ・完了ログ・処理時間計測をまとめた補助構造体
/// wasm では Instant が未対応のため epoch ミリ秒で保持する
pub struct BulkTimer {
    domain: &'static str,
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,
    #[cfg(target_arch = "wasm32")]
    start_ms: u64,
    total: usize,
}

impl BulkTimer {
    pub fn new(domain: &'static str, total: usize) -> Self {
        info!("[{}.bulk_create] リクエスト受信: {}件", domain, total);
        Self {
            domain,
            #[cfg(not(target_arch = "wasm32"))]
            start: Instant::now(),
            #[cfg(target_arch = "wasm32")]
            start_ms: worker::Date::now().as_millis(),
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
        #[cfg(not(target_arch = "wasm32"))]
        let elapsed_ms = self.start.elapsed().as_secs_f64() * 1000.0;
        #[cfg(target_arch = "wasm32")]
        let elapsed_ms = worker::Date::now()
            .as_millis()
            .saturating_sub(self.start_ms) as f64;
        info!(
            "[{}.bulk_create] 完了: inserted={}, skipped={}, 処理時間={:.2}ms",
            self.domain, inserted, skipped, elapsed_ms
        );
        BulkCreateResponse { inserted, skipped }
    }

    /// rows_affected を usize に変換して finish する。
    /// u64 → usize の変換が失敗した場合は ApiError を返す。
    pub fn finish_from_affected(self, rows_affected: u64) -> Result<BulkCreateResponse, ApiError> {
        let inserted = usize::try_from(rows_affected).map_err(|_| {
            ApiError::Internal("bulk insert の rows_affected が usize に収まりません")
        })?;
        Ok(self.finish(inserted))
    }
}

/// bulk insert の UNNEST に渡す user_id 配列を生成する。
pub fn user_ids_for_bulk_insert(user_id: UserId, total: usize) -> Vec<UserId> {
    vec![user_id; total]
}

/// 上限判定の純粋ロジック(DB 非依存)。existing は追記型のみ使用する
fn exceeds_user_row_limit(
    mode: WriteMode,
    existing: Option<i64>,
    additional: usize,
    limit: RowLimit,
) -> bool {
    // usize→i64 は飽和扱い（usize::MAX 級は上限超過とみなしてよい）
    let additional = i64::try_from(additional).unwrap_or(i64::MAX);
    match mode {
        WriteMode::Replace => additional > limit.get(),
        WriteMode::Append => existing.unwrap_or(0) + additional > limit.get(),
    }
}

/// 書き込み前に、利用者ごとの保存行数が上限を超えないことを確認する。
/// 追記型は既存行数との合算、置換型は追加分のみで上限を判定する。
/// `executor` には `&Db` または `&mut Tx`(同一 tx 内で直列化する場合)を渡す。
/// 上限値は引数で渡す(プロセス全体の環境変数に依存させない)
pub async fn ensure_user_row_limit_with<D, E>(
    executor: E,
    user_id: UserId,
    additional: usize,
    limit: RowLimit,
) -> Result<(), ApiError>
where
    D: Domain,
    E: Executor,
{
    let existing = match D::WRITE_MODE {
        WriteMode::Replace => None,
        WriteMode::Append => {
            let mut qb = QueryBuilder::new("SELECT COUNT(*) FROM ");
            qb.push(D::TABLE)
                .push(" WHERE user_id = ")
                .push_bind(user_id);
            qb.build_query_scalar::<Option<i64>>()
                .fetch_one(executor)
                .await?
        }
    };
    if exceeds_user_row_limit(D::WRITE_MODE, existing, additional, limit) {
        return Err(ApiError::Validation(format!(
            "1アカウントあたりの保存件数の上限({limit}件)を超えています。既存データを整理してから取り込んでください"
        )));
    }
    Ok(())
}

/// 同じ利用者の並行した一括登録を直列化する
/// (行数上限の同時突破と、置換型での A∪B の混入を防ぐ)。
/// Hyperdrive は advisory lock をサポートしないため、users 行の
/// `FOR UPDATE` ロックで代替する（同一 tx 内で直列化される点は同じ）
pub async fn lock_user_domain<D: Domain>(tx: &mut Tx, user_id: UserId) -> Result<(), ApiError> {
    crate::db::query(
        "SELECT id FROM users WHERE id = $1 FOR UPDATE",
        vec![Bind::from(user_id)],
    )
    .fetch_scalar_optional::<uuid::Uuid, _>(&mut *tx)
    .await?;
    Ok(())
}

/// ユーザーに紐づく全レコードを削除する共通実装
pub async fn delete_all<D: Domain>(pool: &Db, user_id: UserId) -> Result<u64, ApiError> {
    info!("[{}.delete_all] リクエスト受信", D::NAME);
    let deleted = D::delete_rows(pool, user_id).await?;
    info!("[{}.delete_all] 完了: {}件削除", D::NAME, deleted);
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::{BulkTimer, RowLimit, exceeds_user_row_limit, user_ids_for_bulk_insert};
    use crate::services::domain::WriteMode;
    use shared::value::UserId;
    use uuid::Uuid;

    #[test]
    fn test_exceeds_user_row_limit() {
        // 追記型: existing + additional が limit を超えると true
        {
            let domain = WriteMode::Append;
            assert!(!exceeds_user_row_limit(
                domain,
                Some(0),
                0,
                RowLimit::new(100)
            ));
            assert!(!exceeds_user_row_limit(
                domain,
                Some(0),
                100,
                RowLimit::new(100)
            ));
            assert!(!exceeds_user_row_limit(
                domain,
                Some(99),
                1,
                RowLimit::new(100)
            ));
            assert!(exceeds_user_row_limit(
                domain,
                Some(100),
                1,
                RowLimit::new(100)
            ));
            assert!(exceeds_user_row_limit(
                domain,
                Some(0),
                101,
                RowLimit::new(100)
            ));
            assert!(exceeds_user_row_limit(
                domain,
                Some(99_999),
                2,
                RowLimit::new(100_000)
            ));
        }
        // 置換型(asset_balances): 既存行数を見ず追加分のみで判定
        assert!(!exceeds_user_row_limit(
            WriteMode::Replace,
            None,
            100,
            RowLimit::new(100)
        ));
        assert!(exceeds_user_row_limit(
            WriteMode::Replace,
            None,
            101,
            RowLimit::new(100)
        ));
        assert!(!exceeds_user_row_limit(
            WriteMode::Replace,
            Some(1_000_000),
            1,
            RowLimit::new(100)
        ));
    }

    #[test]
    fn test_domain_names() {
        use crate::services::asset_balance::AssetBalanceDomain;
        use crate::services::dividend::DividendDomain;
        use crate::services::domain::Domain;
        use crate::services::domestic_stock::DomesticStockDomain;
        use crate::services::mutualfund::MutualfundDomain;
        assert_eq!(
            [
                AssetBalanceDomain::NAME,
                DividendDomain::NAME,
                DomesticStockDomain::NAME,
                MutualfundDomain::NAME,
            ],
            ["asset_balance", "dividend", "domestic_stock", "mutualfund"]
        );
    }

    #[test]
    fn test_bulk_timer_finish() {
        for (inserted, expected) in [(0, (0, 5)), (5, (5, 0)), (3, (3, 2))] {
            let response = BulkTimer::new("test", 5).finish(inserted);
            assert_eq!((response.inserted, response.skipped), expected);
        }
    }

    #[test]
    fn test_user_ids_for_bulk_insert() {
        let id = UserId::from(Uuid::new_v4());
        let result = user_ids_for_bulk_insert(id, 3);
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|&v| v == id));

        assert!(user_ids_for_bulk_insert(id, 0).is_empty());
    }
}
