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
    /// u64 → usize の変換が失敗した場合は ApiError を返す。
    pub fn finish_from_result(self, result: PgQueryResult) -> Result<BulkCreateResponse, ApiError> {
        let inserted = usize::try_from(result.rows_affected()).map_err(|_| {
            ApiError::ApiError("bulk insert の rows_affected が usize に収まりません".to_string())
        })?;
        Ok(self.finish(inserted))
    }
}

/// bulk insert の UNNEST に渡す user_id 配列を生成する。
pub fn user_ids_for_bulk_insert(user_id: Uuid, total: usize) -> Vec<Uuid> {
    vec![user_id; total]
}

/// 利用者あたりのドメイン別保存行数の既定上限。
/// 証券口座の取引履歴は年間数百〜数千行の想定に十分な余裕を持たせつつ、
/// DB(Neon)容量の悪用を抑止する値。USER_ROW_LIMIT で上書き可能
const DEFAULT_MAX_USER_ROWS: i64 = 100_000;

fn user_row_limit() -> i64 {
    std::env::var("USER_ROW_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_USER_ROWS)
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
}

/// 上限判定の純粋ロジック(DB 非依存)。existing は追記型のみ使用する
fn exceeds_user_row_limit(
    domain: UserDataDomain,
    existing: Option<i64>,
    additional: usize,
    limit: i64,
) -> bool {
    let additional = additional as i64;
    if domain.replaces_existing() {
        additional > limit
    } else {
        existing.unwrap_or(0) + additional > limit
    }
}

/// 書き込み前に、利用者ごとの保存行数が上限を超えないことを確認する。
/// 追記型(dividends/domestic_stocks/mutualfunds)は既存行数との合算、
/// 置換型(asset_balances)は追加分のみで上限を判定する
pub async fn ensure_user_row_limit(
    pool: &PgPool,
    user_id: Uuid,
    domain: UserDataDomain,
    additional: usize,
) -> Result<(), ApiError> {
    let existing = if domain.replaces_existing() {
        None
    } else {
        match domain {
            UserDataDomain::Dividends => {
                sqlx::query_scalar::<_, Option<i64>>(
                    "SELECT COUNT(*) FROM dividends WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_one(pool)
                .await?
            }
            UserDataDomain::DomesticStocks => {
                sqlx::query_scalar::<_, Option<i64>>(
                    "SELECT COUNT(*) FROM domestic_stocks WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_one(pool)
                .await?
            }
            UserDataDomain::MutualFunds => {
                sqlx::query_scalar::<_, Option<i64>>(
                    "SELECT COUNT(*) FROM mutualfunds WHERE user_id = $1",
                )
                .bind(user_id)
                .fetch_one(pool)
                .await?
            }
            UserDataDomain::AssetBalances => unreachable!(),
        }
    };
    let limit = user_row_limit();
    if exceeds_user_row_limit(domain, existing, additional, limit) {
        return Err(ApiError::ValidationError(format!(
            "1アカウントあたりの保存件数の上限({limit}件)を超えています。既存データを整理してから取り込んでください"
        )));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeleteTarget {
    AssetBalances,
    Dividends,
    DomesticStocks,
    MutualFunds,
}

impl DeleteTarget {
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
///
/// `sqlx::query!` はマクロ呼び出し箇所にSQLリテラルが必要なため、
/// `DeleteTarget` ごとに固定SQLを個別に呼び出す。
pub async fn delete_all_for_user(
    pool: &PgPool,
    user_id: Uuid,
    target: DeleteTarget,
) -> Result<u64, ApiError> {
    let domain = target.domain();
    info!("[{}.delete_all] リクエスト受信", domain);
    let result = match target {
        DeleteTarget::AssetBalances => {
            sqlx::query!("DELETE FROM asset_balances WHERE user_id = $1", user_id)
                .execute(pool)
                .await?
        }
        DeleteTarget::Dividends => {
            sqlx::query!("DELETE FROM dividends WHERE user_id = $1", user_id)
                .execute(pool)
                .await?
        }
        DeleteTarget::DomesticStocks => {
            sqlx::query!("DELETE FROM domestic_stocks WHERE user_id = $1", user_id)
                .execute(pool)
                .await?
        }
        DeleteTarget::MutualFunds => {
            sqlx::query!("DELETE FROM mutualfunds WHERE user_id = $1", user_id)
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
        exceeds_user_row_limit, user_ids_for_bulk_insert, BulkTimer, DeleteTarget, UserDataDomain,
    };
    use uuid::Uuid;

    #[test]
    fn test_exceeds_user_row_limit() {
        // 追記型: existing + additional が limit を超えると true
        for domain in [
            UserDataDomain::Dividends,
            UserDataDomain::DomesticStocks,
            UserDataDomain::MutualFunds,
        ] {
            assert!(!exceeds_user_row_limit(domain, Some(0), 0, 100));
            assert!(!exceeds_user_row_limit(domain, Some(0), 100, 100));
            assert!(!exceeds_user_row_limit(domain, Some(99), 1, 100));
            assert!(exceeds_user_row_limit(domain, Some(100), 1, 100));
            assert!(exceeds_user_row_limit(domain, Some(0), 101, 100));
            assert!(exceeds_user_row_limit(domain, Some(99_999), 2, 100_000));
        }
        // 置換型(asset_balances): 既存行数を見ず追加分のみで判定
        assert!(!exceeds_user_row_limit(
            UserDataDomain::AssetBalances,
            None,
            100,
            100
        ));
        assert!(exceeds_user_row_limit(
            UserDataDomain::AssetBalances,
            None,
            101,
            100
        ));
        assert!(!exceeds_user_row_limit(
            UserDataDomain::AssetBalances,
            Some(1_000_000),
            1,
            100
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
        let id = Uuid::new_v4();
        let result = user_ids_for_bulk_insert(id, 3);
        assert_eq!(result.len(), 3);
        assert!(result.iter().all(|&v| v == id));

        assert!(user_ids_for_bulk_insert(id, 0).is_empty());
    }
}
