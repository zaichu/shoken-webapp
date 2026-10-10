use crate::db::QueryBuilder;
use crate::errors::ApiError;
use crate::models::common::SearchQueryParams;
use chrono::NaiveDate;
use shared::value::UserId;
use std::future::Future;

pub fn parse_date_param(field: &str, value: &str) -> Result<NaiveDate, ApiError> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ApiError::Validation(format!("{field} の形式が不正です（YYYY-MM-DD）")))
}

pub fn year_to_range(year: i32) -> Result<(NaiveDate, NaiveDate), ApiError> {
    let invalid = || ApiError::Validation("year の値が不正です".to_string());
    let start = NaiveDate::from_ymd_opt(year, 1, 1).ok_or_else(invalid)?;
    let end = NaiveDate::from_ymd_opt(year + 1, 1, 1).ok_or_else(invalid)?;
    Ok((start, end))
}

pub fn parse_year_month_range(value: &str) -> Result<(NaiveDate, NaiveDate), ApiError> {
    let invalid = || ApiError::Validation("year_month の形式が不正です（YYYY-MM）".to_string());
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

/// 検索条件に使えるドメイン固有フィールド。SQL カラム名と1対1で対応する
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FilterField {
    Product,
    Account,
    SecurityCode,
    SecurityName,
    FundName,
    Dividends,
}

impl FilterField {
    pub const COUNT: usize = 6;

    /// 対応する SQL カラム名（固定文字列）
    pub fn column(self) -> &'static str {
        match self {
            Self::Product => "product",
            Self::Account => "account",
            Self::SecurityCode => "security_code",
            Self::SecurityName => "security_name",
            Self::FundName => "fund_name",
            Self::Dividends => "dividends",
        }
    }
}

/// ドメインに存在しないフィールドを指す共有の None（`Search::filter_value` 用）
pub(crate) static NO_FIELD: Option<String> = None;

/// 4ドメイン共通の検索条件中間表現。
/// 使うフィールドはドメインが `Search::FILTER_FIELDS` で宣言する
#[derive(Debug)]
pub struct SearchFilter {
    /// date 系条件（日付軸を持たないドメインでは常に既定値＝条件なし）
    pub date_axis: DateAxisFilter,
    /// フリーワードをスペース分割した token 列
    pub tokens: Vec<String>,
    /// FilterField ごとの完全一致条件
    pub(crate) values: [Option<String>; FilterField::COUNT],
}

impl SearchFilter {
    /// フィールドの完全一致条件（対象外フィールドは None）
    pub fn value(&self, field: FilterField) -> &Option<String> {
        &self.values[field as usize]
    }
}

/// date_column（呼び出し側が渡す固定文字列のみ）を使って date 条件を WHERE 句へ push する
pub fn push_date_axis_filters(
    qb: &mut QueryBuilder,
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
    qb: &mut QueryBuilder,
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
    qb: &mut QueryBuilder,
    user_id: UserId,
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

/// `include_*` フラグに応じて fetch future を実行し `Some`/`None` を返す。
///
/// future は lazy のため `include=false` の場合は inner future を poll せず、
/// クエリを発行しない。4ドメインの `search()` 内でバイト同一だった
/// summary/facets 条件分岐ブロックの共通化（SQL 生成には触らない）。
pub async fn fetch_if_included<T>(
    include: bool,
    fetch: impl Future<Output = Result<T, ApiError>>,
) -> Result<Option<T>, ApiError> {
    if include {
        fetch.await.map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DateAxisFilter, escape_like_pattern, fetch_if_included, parse_date_param,
        parse_year_month_range, push_date_axis_filters, push_token_ilike_filters,
        tokens_from_query, year_to_range,
    };
    use crate::db::QueryBuilder;
    use crate::errors::ApiError;
    use chrono::NaiveDate;

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
                    Err(ApiError::Validation(_))
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
                    Err(ApiError::Validation(_))
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

        let mut qb = QueryBuilder::new("SELECT 1 FROM dummy");
        push_date_axis_filters(&mut qb, "settlement_date", &filter);
        let sql = qb.sql();

        assert_eq!(sql.matches("settlement_date = ").count(), 1);
        assert_eq!(sql.matches("settlement_date >= ").count(), 3);
        assert_eq!(sql.matches("settlement_date <= ").count(), 1);
        assert_eq!(sql.matches("settlement_date < ").count(), 2);
    }

    #[test]
    fn test_push_date_axis_filters_pushes_nothing_when_all_none() {
        let mut qb = QueryBuilder::new("SELECT 1 FROM dummy");
        push_date_axis_filters(&mut qb, "settlement_date", &DateAxisFilter::default());
        assert_eq!(qb.sql(), "SELECT 1 FROM dummy");
    }

    #[test]
    fn test_push_token_ilike_filters_combines_columns_as_or_per_token() {
        let tokens = vec!["AA".to_string(), "BB".to_string()];
        let mut qb = QueryBuilder::new("SELECT 1 FROM dummy");
        push_token_ilike_filters(&mut qb, &tokens, &["product", "account"]);
        let sql = qb.sql();

        assert_eq!(sql.matches(" AND (").count(), 2);
        assert_eq!(sql.matches("product ILIKE").count(), 2);
        assert_eq!(sql.matches("account ILIKE").count(), 2);
        assert_eq!(sql.matches(" OR ").count(), 2);
        assert_eq!(
            sql,
            "SELECT 1 FROM dummy AND (product ILIKE $1 ESCAPE '\\' OR account ILIKE $2 ESCAPE '\\') AND (product ILIKE $3 ESCAPE '\\' OR account ILIKE $4 ESCAPE '\\')"
        );
    }

    #[test]
    fn test_push_token_ilike_filters_pushes_nothing_when_columns_empty() {
        let tokens = vec!["AA".to_string()];
        let mut qb = QueryBuilder::new("SELECT 1 FROM dummy");
        push_token_ilike_filters(&mut qb, &tokens, &[]);
        assert_eq!(qb.sql(), "SELECT 1 FROM dummy");
    }

    #[tokio::test]
    async fn test_fetch_if_included_returns_some_and_propagates_error_when_included() {
        let some = fetch_if_included(true, async { Ok::<_, ApiError>(42) })
            .await
            .expect("include=true かつ Ok なら Some を返す");
        assert_eq!(some, Some(42));

        let err = fetch_if_included(true, async { Err::<i32, _>(ApiError::NotFound) }).await;
        assert!(
            matches!(err, Err(ApiError::NotFound)),
            "include=true の場合は inner future のエラーをそのまま返す"
        );
    }

    #[tokio::test]
    async fn test_fetch_if_included_returns_none_without_polling_when_excluded() {
        // include=false の場合は inner future を poll しない（クエリ発行なし）。
        // poll されたら panic する future を渡して固定する。
        let result: Result<Option<i32>, ApiError> = fetch_if_included(false, async {
            panic!("include=false の場合は poll してはならない")
        })
        .await;
        assert_eq!(result.expect("include=false は Ok(None) を返す"), None);
    }

    fn unescape_like_pattern(escaped: &str) -> Option<String> {
        let inner = escaped.strip_prefix('%')?.strip_suffix('%')?;
        let mut chars = inner.chars();
        let mut recovered = String::new();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next()? {
                    esc @ ('\\' | '%' | '_') => recovered.push(esc),
                    _ => return None,
                }
            } else {
                // 素朴モデルでは %, _ はワイルドカード扱いになるため、
                // エスケープなしで現れてはならない
                if c == '%' || c == '_' {
                    return None;
                }
                recovered.push(c);
            }
        }
        Some(recovered)
    }

    mod domain_specs {
        use crate::db::QueryBuilder;
        use crate::errors::ApiError;
        use crate::models::asset_balance::AssetBalanceSearchQueryParams;
        use crate::models::common::SearchQueryParams;
        use crate::models::dividend::DividendSearchQueryParams;
        use crate::models::domestic_stock::DomesticStockSearchQueryParams;
        use crate::models::mutualfund::MutualfundSearchQueryParams;
        use crate::services::asset_balance::AssetBalanceDomain;
        use crate::services::dividend::DividendDomain;
        use crate::services::domain::search::Search;
        use crate::services::domestic_stock::DomesticStockDomain;
        use crate::services::mutualfund::MutualfundDomain;
        use shared::value::UserId;

        fn filter_sql<D: Search>(params: &D::Params) -> String {
            let filter = D::make_filter(params).expect("正常な params はフィルタ化できる");
            let mut qb = QueryBuilder::new(format!("SELECT 1 FROM {}", D::TABLE));
            D::push_filters(&mut qb, UserId::default(), &filter);
            qb.sql().to_string()
        }

        fn search_with_q(q: &str) -> SearchQueryParams {
            SearchQueryParams {
                q: Some(q.to_string()),
                ..Default::default()
            }
        }

        /// q の token 数分だけ `AND (col ILIKE ... OR ...)` ブロックが出ることを全ドメインで確認
        #[test]
        fn test_push_filters_expands_q_tokens_over_domain_fields() {
            let cases: [(String, &[&str]); 4] = [
                (
                    filter_sql::<DividendDomain>(&DividendSearchQueryParams {
                        search: search_with_q("AA BB"),
                        ..Default::default()
                    }),
                    &["product", "account", "security_code", "security_name"],
                ),
                (
                    filter_sql::<DomesticStockDomain>(&DomesticStockSearchQueryParams {
                        search: search_with_q("AA BB"),
                        ..Default::default()
                    }),
                    &["account", "security_code", "security_name"],
                ),
                (
                    filter_sql::<MutualfundDomain>(&MutualfundSearchQueryParams {
                        search: search_with_q("AA BB"),
                        ..Default::default()
                    }),
                    &["account", "fund_name", "dividends"],
                ),
                (
                    filter_sql::<AssetBalanceDomain>(&AssetBalanceSearchQueryParams {
                        search: search_with_q("AA BB"),
                        ..Default::default()
                    }),
                    &["security_code", "security_name"],
                ),
            ];

            for (sql, columns) in cases {
                assert_eq!(sql.matches(" AND (").count(), 2, "sql={sql}");
                for column in columns {
                    assert_eq!(
                        sql.matches(&format!("{column} ILIKE")).count(),
                        2,
                        "column={column} sql={sql}"
                    );
                }
            }
        }

        /// ドメイン固有フィールドが `AND <col> =` の完全一致条件になることを全ドメインで確認
        #[test]
        fn test_push_filters_emits_exact_match_for_domain_fields() {
            let cases: [(String, &[&str]); 4] = [
                (
                    filter_sql::<DividendDomain>(&DividendSearchQueryParams {
                        product: Some("国内株式".to_string()),
                        account: Some("特定".to_string()),
                        security_code: Some("1234".to_string()),
                        security_name: Some("テスト株式会社".to_string()),
                        ..Default::default()
                    }),
                    &["product", "account", "security_code", "security_name"],
                ),
                (
                    filter_sql::<DomesticStockDomain>(&DomesticStockSearchQueryParams {
                        account: Some("特定".to_string()),
                        security_code: Some("1234".to_string()),
                        security_name: Some("テスト株式会社".to_string()),
                        ..Default::default()
                    }),
                    &["account", "security_code", "security_name"],
                ),
                (
                    filter_sql::<MutualfundDomain>(&MutualfundSearchQueryParams {
                        account: Some("特定".to_string()),
                        fund_name: Some("eMAXIS Slim".to_string()),
                        dividends: Some("再投資型".to_string()),
                        ..Default::default()
                    }),
                    &["account", "fund_name", "dividends"],
                ),
                (
                    filter_sql::<AssetBalanceDomain>(&AssetBalanceSearchQueryParams {
                        security_code: Some("1234".to_string()),
                        security_name: Some("テスト株式会社".to_string()),
                        ..Default::default()
                    }),
                    &["security_code", "security_name"],
                ),
            ];

            for (sql, columns) in cases {
                for column in columns {
                    assert!(
                        sql.contains(&format!("AND {column} = ")),
                        "column={column} sql={sql}"
                    );
                }
            }
        }

        /// date 系パラメータが DATE_COLUMN への条件になること（日付軸を持つ3ドメイン）
        #[test]
        fn test_push_filters_applies_date_params_to_date_column() {
            let search = SearchQueryParams {
                date: Some("2026-01-15".to_string()),
                year: Some(2026),
                year_month: Some("2026-06".to_string()),
                ..Default::default()
            };

            let cases: [(String, &str); 3] = [
                (
                    filter_sql::<DividendDomain>(&DividendSearchQueryParams {
                        search: search.clone(),
                        ..Default::default()
                    }),
                    "settlement_date",
                ),
                (
                    filter_sql::<DomesticStockDomain>(&DomesticStockSearchQueryParams {
                        search: search.clone(),
                        ..Default::default()
                    }),
                    "trade_date",
                ),
                (
                    filter_sql::<MutualfundDomain>(&MutualfundSearchQueryParams {
                        search,
                        ..Default::default()
                    }),
                    "trade_date",
                ),
            ];

            for (sql, column) in cases {
                assert_eq!(sql.matches(&format!("{column} = ")).count(), 1, "sql={sql}");
                assert_eq!(
                    sql.matches(&format!("{column} >= ")).count(),
                    2,
                    "sql={sql}"
                );
                assert_eq!(sql.matches(&format!("{column} < ")).count(), 2, "sql={sql}");
            }
        }

        /// 不正な date 系パラメータが ValidationError になること（日付軸を持つドメインのみ）
        #[test]
        fn test_make_filter_rejects_invalid_date_params() {
            let cases: [(&'static str, SearchQueryParams); 5] = [
                (
                    "date",
                    SearchQueryParams {
                        date: Some("2026/01/15".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "date_from",
                    SearchQueryParams {
                        date_from: Some("20260101".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "date_to",
                    SearchQueryParams {
                        date_to: Some("not-a-date".to_string()),
                        ..Default::default()
                    },
                ),
                (
                    "year",
                    SearchQueryParams {
                        year: Some(i32::MIN),
                        ..Default::default()
                    },
                ),
                (
                    "year_month",
                    SearchQueryParams {
                        year_month: Some("2026-13".to_string()),
                        ..Default::default()
                    },
                ),
            ];

            for (field, search) in cases {
                let err = DividendDomain::make_filter(&DividendSearchQueryParams {
                    search: search.clone(),
                    ..Default::default()
                })
                .expect_err("不正値は ValidationError になるべき");
                assert!(matches!(err, ApiError::Validation(_)), "field={field}");
                let err = DomesticStockDomain::make_filter(&DomesticStockSearchQueryParams {
                    search: search.clone(),
                    ..Default::default()
                })
                .expect_err("不正値は ValidationError になるべき");
                assert!(matches!(err, ApiError::Validation(_)), "field={field}");
                let err = MutualfundDomain::make_filter(&MutualfundSearchQueryParams {
                    search,
                    ..Default::default()
                })
                .expect_err("不正値は ValidationError になるべき");
                assert!(matches!(err, ApiError::Validation(_)), "field={field}");
            }
        }

        /// asset_balances は snapshot 日付を持たず date 系パラメータを解釈・検証しない
        /// （不正な値でもエラーにならず WHERE にも現れない）
        #[test]
        fn test_asset_balance_ignores_date_params() {
            let sql = filter_sql::<AssetBalanceDomain>(&AssetBalanceSearchQueryParams {
                search: SearchQueryParams {
                    date: Some("not-a-date".to_string()),
                    year: Some(i32::MIN),
                    year_month: Some("invalid".to_string()),
                    ..Default::default()
                },
                security_code: Some("1234".to_string()),
                ..Default::default()
            });

            assert!(!sql.contains("date"), "date 条件が出てはいけない: {sql}");
            assert!(sql.contains("AND security_code = "));
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_escape_like_pattern_roundtrip(token in ".*") {
            let escaped = escape_like_pattern(&token);
            let recovered = unescape_like_pattern(&escaped);
            proptest::prop_assert_eq!(
                recovered.as_deref(),
                Some(token.as_str()),
                "escaped={}",
                escaped
            );
        }

        // i32::MAX では実装・参照モデル双方の year+1 が debug でオーバーフローするため除く
        #[test]
        fn prop_year_to_range_matches_reference(year in i32::MIN..i32::MAX) {
            match year_to_range(year) {
                Ok((start, end)) => {
                    proptest::prop_assert_eq!(
                        start,
                        NaiveDate::from_ymd_opt(year, 1, 1).unwrap()
                    );
                    proptest::prop_assert_eq!(
                        end,
                        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
                    );
                    proptest::prop_assert!(start < end);
                }
                Err(e) => {
                    proptest::prop_assert!(
                        matches!(e, ApiError::Validation(_)),
                        "year={year} の Err が ValidationError ではない"
                    );
                }
            }
        }

        #[test]
        fn prop_parse_year_month_range_matches_reference(
            year in 0i32..10_000i32,
            month in 0u32..=13u32,
        ) {
            let input = format!("{year}-{month:02}");
            match parse_year_month_range(&input) {
                Ok((start, end)) => {
                    proptest::prop_assert_eq!(
                        start,
                        NaiveDate::from_ymd_opt(year, month, 1).unwrap()
                    );
                    let expected_end = if month == 12 {
                        NaiveDate::from_ymd_opt(year + 1, 1, 1)
                    } else {
                        NaiveDate::from_ymd_opt(year, month + 1, 1)
                    };
                    proptest::prop_assert_eq!(end, expected_end.unwrap());
                }
                Err(e) => {
                    proptest::prop_assert!(
                        matches!(e, ApiError::Validation(_)),
                        "input={input} の Err が ValidationError ではない"
                    );
                    proptest::prop_assert!(
                        !(1..=12).contains(&month),
                        "有効な月 {input} が Err になった"
                    );
                }
            }
        }

        #[test]
        fn prop_tokens_from_query_splits_and_never_empty(
            tokens in proptest::collection::vec("[^\\s]+", 0..8),
        ) {
            let query = tokens.join(" ");
            let parsed = tokens_from_query(Some(&query));
            proptest::prop_assert!(parsed.iter().all(|t| !t.is_empty()));
            proptest::prop_assert_eq!(parsed, tokens);
        }
    }
}
