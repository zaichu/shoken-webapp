use crate::errors::ApiError;
use crate::models::common::{PaginatedSearchResponse, SearchQueryParams};
use chrono::NaiveDate;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use std::future::Future;
use utoipa::ToSchema;
use uuid::Uuid;

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

/// `SearchQueryParams` の date 系フィールドを解析した中間表現
/// （date_eq / date_from / date_to / year_range / year_month_range）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DateAxisFilter {
    pub date_eq: Option<NaiveDate>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub year_range: Option<(NaiveDate, NaiveDate)>,
    pub year_month_range: Option<(NaiveDate, NaiveDate)>,
}

impl DateAxisFilter {
    pub fn from_search_params(search: &SearchQueryParams) -> Result<Self, ApiError> {
        let date_eq = search
            .date
            .as_deref()
            .map(|v| parse_date_param("date", v))
            .transpose()?;
        let date_from = search
            .date_from
            .as_deref()
            .map(|v| parse_date_param("date_from", v))
            .transpose()?;
        let date_to = search
            .date_to
            .as_deref()
            .map(|v| parse_date_param("date_to", v))
            .transpose()?;
        let year_range = search.year.map(year_to_range).transpose()?;
        let year_month_range = search
            .year_month
            .as_deref()
            .map(parse_year_month_range)
            .transpose()?;

        Ok(Self {
            date_eq,
            date_from,
            date_to,
            year_range,
            year_month_range,
        })
    }
}

/// date_column（呼び出し側が渡す固定文字列のみ）を使って date 条件を WHERE 句へ push する
pub fn push_date_axis_filters(
    qb: &mut QueryBuilder<Postgres>,
    date_column: &'static str,
    filter: &DateAxisFilter,
) {
    if let Some(v) = filter.date_eq {
        qb.push(format!(" AND {date_column} = ")).push_bind(v);
    }
    if let Some(v) = filter.date_from {
        qb.push(format!(" AND {date_column} >= ")).push_bind(v);
    }
    if let Some(v) = filter.date_to {
        qb.push(format!(" AND {date_column} <= ")).push_bind(v);
    }
    if let Some((start, end)) = filter.year_range {
        qb.push(format!(" AND {date_column} >= ")).push_bind(start);
        qb.push(format!(" AND {date_column} < ")).push_bind(end);
    }
    if let Some((start, end)) = filter.year_month_range {
        qb.push(format!(" AND {date_column} >= ")).push_bind(start);
        qb.push(format!(" AND {date_column} < ")).push_bind(end);
    }
}

/// フリーワード検索欄をスペース区切りの token 配列へ変換する
pub fn tokens_from_query(q: Option<&str>) -> Vec<String> {
    q.map(|q| q.split_whitespace().map(str::to_string).collect())
        .unwrap_or_default()
}

/// token ごとに columns（呼び出し側が渡す固定文字列配列のみ）への ILIKE OR 条件を
/// `AND (col1 ILIKE $n ESCAPE '\' OR col2 ILIKE $n+1 ESCAPE '\' ...)` として push する
pub fn push_token_ilike_filters(
    qb: &mut QueryBuilder<Postgres>,
    tokens: &[String],
    columns: &[&'static str],
) {
    if columns.is_empty() {
        return;
    }

    for token in tokens {
        let pattern = escape_like_pattern(token);
        qb.push(" AND (");
        for (i, column) in columns.iter().enumerate() {
            if i > 0 {
                qb.push(" OR ");
            }
            qb.push(format!("{column} ILIKE "))
                .push_bind(pattern.clone())
                .push(" ESCAPE '\\'");
        }
        qb.push(")");
    }
}

/// user_id 条件 + optional date axis 条件 + Option 完全一致条件 + token ILIKE 条件を
/// この順で WHERE 句として QueryBuilder へ積む共通ヘルパー。
///
/// - `date_axis`: `(date_column, filter)`。date 系フィルタを持たないドメイン（asset_balance 等）は `None` を渡す
/// - `exact_match_fields`: `(column, value)` の並び。呼び出し側が対象カラムを固定 `&'static str` で指定する
/// - `token_columns`: フリーワード検索（ILIKE OR）の対象カラム
pub fn push_search_filters(
    qb: &mut QueryBuilder<Postgres>,
    user_id: Uuid,
    date_axis: Option<(&'static str, &DateAxisFilter)>,
    exact_match_fields: &[(&'static str, &Option<String>)],
    tokens: &[String],
    token_columns: &[&'static str],
) {
    qb.push(" WHERE user_id = ").push_bind(user_id);
    if let Some((date_column, filter)) = date_axis {
        push_date_axis_filters(qb, date_column, filter);
    }
    for (column, value) in exact_match_fields {
        if let Some(v) = value {
            qb.push(format!(" AND {column} = ")).push_bind(v.clone());
        }
    }
    push_token_ilike_filters(qb, tokens, token_columns);
}

/// count query / data query / summary future / facets future を `tokio::try_join!` で並行実行し
/// `PaginatedSearchResponse` を組み立てる4ドメイン共通の検索制御フロー。
///
/// - `count_sql` / `data_sql`: フィルタ前の `SELECT ... FROM table`（呼び出し側が渡す固定文字列）
/// - `order_by`: data query に付与する `ORDER BY` 句（先頭スペース込み。LIMIT/OFFSET は本関数側で付与する）
/// - `push_filters`: count/data 両方の WHERE 句を積むクロージャ（`push_search_filters` 等を呼ぶ想定）
/// - `summary_fut` / `facets_fut`: `should_include_summary`/`should_include_facets` に応じて
///   `Some`/`None` を返す形に呼び出し側が組み立てた future をそのまま渡す
#[allow(clippy::too_many_arguments)]
pub async fn run_paginated_search<T, Summary, Facets, SummaryFut, FacetsFut>(
    pool: &PgPool,
    count_sql: &str,
    data_sql: &str,
    order_by: &str,
    push_filters: impl Fn(&mut QueryBuilder<Postgres>),
    page: i64,
    per_page: i64,
    offset: i64,
    summary_fut: SummaryFut,
    facets_fut: FacetsFut,
) -> Result<PaginatedSearchResponse<T, Summary, Facets>, ApiError>
where
    T: for<'r> FromRow<'r, PgRow> + ToSchema + Send + Unpin + 'static,
    Summary: ToSchema + 'static,
    Facets: ToSchema + 'static,
    SummaryFut: Future<Output = Result<Option<Summary>, ApiError>>,
    FacetsFut: Future<Output = Result<Option<Facets>, ApiError>>,
{
    let mut count_qb: QueryBuilder<Postgres> = QueryBuilder::new(count_sql);
    push_filters(&mut count_qb);
    let count_fut = async move {
        let total: i64 = count_qb.build_query_scalar().fetch_one(pool).await?;
        Ok::<_, ApiError>(total)
    };

    let mut data_qb: QueryBuilder<Postgres> = QueryBuilder::new(data_sql);
    push_filters(&mut data_qb);
    data_qb.push(order_by);
    data_qb.push(" LIMIT ").push_bind(per_page);
    data_qb.push(" OFFSET ").push_bind(offset);
    let data_fut = async move {
        let data = data_qb.build_query_as::<T>().fetch_all(pool).await?;
        Ok::<_, ApiError>(data)
    };

    // count/data/summary/facets は相互に依存しないため並行実行する
    let (total, data, summary, facets) =
        tokio::try_join!(count_fut, data_fut, summary_fut, facets_fut)?;

    Ok(PaginatedSearchResponse {
        data,
        total,
        page,
        per_page,
        summary,
        facets,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        escape_like_pattern, parse_date_param, parse_year_month_range, push_date_axis_filters,
        push_token_ilike_filters, tokens_from_query, year_to_range, DateAxisFilter,
    };
    use crate::errors::ApiError;
    use chrono::NaiveDate;
    use sqlx::{Postgres, QueryBuilder};

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
    fn test_tokens_from_query_splits_on_whitespace() {
        assert_eq!(tokens_from_query(None), Vec::<String>::new());
        assert_eq!(tokens_from_query(Some("")), Vec::<String>::new());
        assert_eq!(
            tokens_from_query(Some("AA  BB")),
            vec!["AA".to_string(), "BB".to_string()]
        );
    }

    #[test]
    fn test_push_date_axis_filters_pushes_all_present_conditions_with_given_column() {
        let filter = DateAxisFilter {
            date_eq: NaiveDate::from_ymd_opt(2026, 1, 15),
            date_from: NaiveDate::from_ymd_opt(2026, 1, 1),
            date_to: NaiveDate::from_ymd_opt(2026, 12, 31),
            year_range: Some((
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            )),
            year_month_range: Some((
                NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            )),
        };

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM dummy");
        push_date_axis_filters(&mut qb, "settlement_date", &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        assert_eq!(sql.matches("settlement_date = ").count(), 1);
        assert_eq!(sql.matches("settlement_date >= ").count(), 3);
        assert_eq!(sql.matches("settlement_date <= ").count(), 1);
        assert_eq!(sql.matches("settlement_date < ").count(), 2);
    }

    #[test]
    fn test_push_date_axis_filters_pushes_nothing_when_all_none() {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM dummy");
        push_date_axis_filters(&mut qb, "settlement_date", &DateAxisFilter::default());
        assert_eq!(qb.sql().as_str(), "SELECT 1 FROM dummy");
    }

    #[test]
    fn test_push_token_ilike_filters_combines_columns_as_or_per_token() {
        let tokens = vec!["AA".to_string(), "BB".to_string()];
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM dummy");
        push_token_ilike_filters(&mut qb, &tokens, &["product", "account"]);
        let sql = qb.sql();
        let sql = sql.as_str();

        assert_eq!(sql.matches(" AND (").count(), 2);
        assert_eq!(sql.matches("product ILIKE").count(), 2);
        assert_eq!(sql.matches("account ILIKE").count(), 2);
        assert_eq!(sql.matches(" OR ").count(), 2);
    }

    #[test]
    fn test_push_token_ilike_filters_pushes_nothing_when_columns_empty() {
        let tokens = vec!["AA".to_string()];
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM dummy");
        push_token_ilike_filters(&mut qb, &tokens, &[]);
        assert_eq!(qb.sql().as_str(), "SELECT 1 FROM dummy");
    }
}
