use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use shared::value::UserId;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;
use std::fmt;
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
    /// u64 → usize の変換が失敗した場合は ApiError を返す。
    pub fn finish_from_result(
        self,
        result: &PgQueryResult,
    ) -> Result<BulkCreateResponse, ApiError> {
        let inserted = usize::try_from(result.rows_affected()).map_err(|_| {
            ApiError::Internal("bulk insert の rows_affected が usize に収まりません")
        })?;
        Ok(self.finish(inserted))
    }
}

/// bulk insert の UNNEST に渡す user_id 配列を生成する。
pub fn user_ids_for_bulk_insert(user_id: UserId, total: usize) -> Vec<UserId> {
    vec![user_id; total]
}

/// 行数上限を適用するユーザー紐付き書き込みドメイン
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserDataDomain {
    AssetBalances,
    Dividends,
    DomesticStocks,
    MutualFunds,
}

impl UserDataDomain {
    /// asset_balances は bulk_create が DELETE→INSERT の全置換のため累積しない。
    /// 置換型は追加分のみ、追記型は既存行数+追加分で上限を判定する
    fn replaces_existing(self) -> bool {
        matches!(self, Self::AssetBalances)
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

/// 上限判定の純粋ロジック(DB 非依存)。existing は追記型のみ使用する
fn exceeds_user_row_limit(
    domain: UserDataDomain,
    existing: Option<i64>,
    additional: usize,
    limit: RowLimit,
) -> bool {
    // usize→i64 は飽和扱い（usize::MAX 級は上限超過とみなしてよい）
    let additional = i64::try_from(additional).unwrap_or(i64::MAX);
    if domain.replaces_existing() {
        additional > limit.get()
    } else {
        existing.unwrap_or(0) + additional > limit.get()
    }
}

/// 書き込み前に、利用者ごとの保存行数が上限を超えないことを確認する。
/// 追記型(dividends/domestic_stocks/mutualfunds)は既存行数との合算、
/// 置換型(asset_balances)は追加分のみで上限を判定する。
/// `executor` には `&PgPool` または `&mut Transaction`(同一 tx 内で直列化する場合)を渡す。
/// 上限値は引数で渡す(プロセス全体の環境変数に依存させない)
pub async fn ensure_user_row_limit_with<'e, E>(
    executor: E,
    user_id: UserId,
    domain: UserDataDomain,
    additional: usize,
    limit: RowLimit,
) -> Result<(), ApiError>
where
    E: sqlx::Executor<'e, Database = sqlx::Postgres>,
{
    let existing = if domain.replaces_existing() {
        None
    } else {
        match domain {
            UserDataDomain::Dividends => {
                sqlx::query_scalar::<_, Option<i64>>(
                    "SELECT COUNT(*) FROM dividends WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_one(executor)
                .await?
            }
            UserDataDomain::DomesticStocks => {
                sqlx::query_scalar::<_, Option<i64>>(
                    "SELECT COUNT(*) FROM domestic_stocks WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_one(executor)
                .await?
            }
            UserDataDomain::MutualFunds => {
                sqlx::query_scalar::<_, Option<i64>>(
                    "SELECT COUNT(*) FROM mutualfunds WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_one(executor)
                .await?
            }
            UserDataDomain::AssetBalances => unreachable!(),
        }
    };
    if exceeds_user_row_limit(domain, existing, additional, limit) {
        return Err(ApiError::Validation(format!(
            "1アカウントあたりの保存件数の上限({limit}件)を超えています。既存データを整理してから取り込んでください"
        )));
    }
    Ok(())
}

pub type DeleteTarget = UserDataDomain;

/// ユーザーに紐づく全レコードを削除する共通実装。
///
/// `sqlx::query!` はマクロ呼び出し箇所にSQLリテラルが必要なため、
/// `DeleteTarget` ごとに固定SQLを個別に呼び出す。
pub async fn delete_all_for_user(
    pool: &PgPool,
    user_id: UserId,
    target: DeleteTarget,
) -> Result<u64, ApiError> {
    let domain = target.domain();
    info!("[{}.delete_all] リクエスト受信", domain);
    let result = match target {
        DeleteTarget::AssetBalances => {
            sqlx::query!(
                "DELETE FROM asset_balances WHERE user_id = $1",
                user_id.get()
            )
            .execute(pool)
            .await?
        }
        DeleteTarget::Dividends => {
            sqlx::query!("DELETE FROM dividends WHERE user_id = $1", user_id.get())
                .execute(pool)
                .await?
        }
        DeleteTarget::DomesticStocks => {
            sqlx::query!(
                "DELETE FROM domestic_stocks WHERE user_id = $1",
                user_id.get()
            )
            .execute(pool)
            .await?
        }
        DeleteTarget::MutualFunds => {
            sqlx::query!("DELETE FROM mutualfunds WHERE user_id = $1", user_id.get())
                .execute(pool)
                .await?
        }
    };
    let deleted = result.rows_affected();
    info!("[{}.delete_all] 完了: {}件削除", domain, deleted);
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::{
        exceeds_user_row_limit, user_ids_for_bulk_insert, BulkTimer, DeleteTarget, RowLimit,
        UserDataDomain,
    };
    use shared::value::UserId;
    use uuid::Uuid;

    #[test]
    fn test_exceeds_user_row_limit() {
        // 追記型: existing + additional が limit を超えると true
        for domain in [
            UserDataDomain::Dividends,
            UserDataDomain::DomesticStocks,
            UserDataDomain::MutualFunds,
        ] {
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
            UserDataDomain::AssetBalances,
            None,
            100,
            RowLimit::new(100)
        ));
        assert!(exceeds_user_row_limit(
            UserDataDomain::AssetBalances,
            None,
            101,
            RowLimit::new(100)
        ));
        assert!(!exceeds_user_row_limit(
            UserDataDomain::AssetBalances,
            Some(1_000_000),
            1,
            RowLimit::new(100)
        ));
    }

    #[test]
    fn test_delete_target_domain_names() {
        assert_eq!(
            [
                DeleteTarget::AssetBalances.domain(),
                DeleteTarget::Dividends.domain(),
                DeleteTarget::DomesticStocks.domain(),
                DeleteTarget::MutualFunds.domain(),
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
