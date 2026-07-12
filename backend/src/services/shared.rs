use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use chrono::NaiveDate;
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

pub fn parse_date_param(field: &str, value: &str) -> Result<NaiveDate, ApiError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ApiError::ValidationError(format!("{field} の形式が不正です（YYYY-MM-DD）")))
}

pub fn year_to_range(year: i32) -> Result<(NaiveDate, NaiveDate), ApiError> {
    let invalid = || ApiError::ValidationError("year の値が不正です".to_string());
    let start = NaiveDate::from_ymd_opt(year, 1, 1).ok_or_else(invalid)?;
    let end = NaiveDate::from_ymd_opt(year + 1, 1, 1).ok_or_else(invalid)?;
    Ok((start, end))
}

pub fn parse_year_month_range(value: &str) -> Result<(NaiveDate, NaiveDate), ApiError> {
    let invalid =
        || ApiError::ValidationError("year_month の形式が不正です（YYYY-MM）".to_string());
    let (year_str, month_str) = value.split_once('-').ok_or_else(invalid)?;
    let year: i32 = year_str.parse().map_err(|_| invalid())?;
    let month: u32 = month_str.parse().map_err(|_| invalid())?;
    let start = NaiveDate::from_ymd_opt(year, month, 1).ok_or_else(invalid)?;
    let end = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .ok_or_else(invalid)?;
    Ok((start, end))
}

/// ILIKE の wildcard 文字（%, _, \）をリテラル扱いにエスケープしてから前後を % で囲む
pub fn escape_like_pattern(token: &str) -> String {
    let escaped = token
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

#[cfg(test)]
mod tests {
    use super::{
        escape_like_pattern, parse_date_param, parse_year_month_range, user_ids_for_bulk_insert,
        year_to_range, BulkTimer,
    };
    use crate::errors::ApiError;
    use chrono::NaiveDate;
    use uuid::Uuid;

    #[test]
    fn test_year_to_range_produces_year_boundaries() {
        let (start, end) = year_to_range(2026).expect("2026年は有効な範囲");
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2027, 1, 1).unwrap());
    }

    #[test]
    fn test_parse_year_month_range_handles_december_wrap_and_invalid_values() {
        let (start, end) = parse_year_month_range("2026-06").expect("2026-06 は有効");
        assert_eq!(start, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(end, NaiveDate::from_ymd_opt(2026, 7, 1).unwrap());

        // 12月は年をまたいで翌年1月1日になる
        let (_, end) = parse_year_month_range("2026-12").expect("2026-12 は有効");
        assert_eq!(end, NaiveDate::from_ymd_opt(2027, 1, 1).unwrap());

        for invalid in ["2026/06", "2026-13", "abcd-06", "2026-06-01"] {
            assert!(
                matches!(
                    parse_year_month_range(invalid),
                    Err(ApiError::ValidationError(_))
                ),
                "value={invalid} は ValidationError になるべき"
            );
        }
    }

    #[test]
    fn test_parse_date_param_accepts_iso_format_and_rejects_others() {
        assert!(parse_date_param("date", "2026-01-15").is_ok());
        for invalid in ["2026/01/15", "15-01-2026", "not-a-date", ""] {
            assert!(
                matches!(
                    parse_date_param("date", invalid),
                    Err(ApiError::ValidationError(_))
                ),
                "value={invalid} は ValidationError になるべき"
            );
        }
    }

    #[test]
    fn test_escape_like_pattern_escapes_wildcard_characters() {
        assert_eq!(escape_like_pattern("abc"), "%abc%");
        assert_eq!(escape_like_pattern("50%"), "%50\\%%");
        assert_eq!(escape_like_pattern("A_B"), "%A\\_B%");
        assert_eq!(escape_like_pattern("a\\b"), "%a\\\\b%");
        assert_eq!(escape_like_pattern("100%_off\\"), "%100\\%\\_off\\\\%");
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
