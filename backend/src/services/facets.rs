use crate::errors::ApiError;
use crate::models::common::FacetOption;
use sqlx::{PgPool, Postgres, QueryBuilder};

/// fetch_group_facets の GROUP BY 結果に対する ORDER BY 方向（呼び出し側が渡す固定値のみ）
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FacetOrder {
    Asc,
    Desc,
}

impl FacetOrder {
    fn as_sql(self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

/// fetch_group_facets で GROUP BY に使える式を限定する集計対象カラム（3ドメイン共通）。
/// 年/年月のSQL式は日付カラムごとにバリアントを分け、SQL文字列自体は集約前と同一にする。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupField {
    Account,
    Product,
    FundName,
    TradeDateYear,
    TradeDateYearMonth,
    SettlementDateYear,
    SettlementDateYearMonth,
}

impl GroupField {
    /// SQL に埋め込む式（固定の &'static str のみを返す）
    pub fn as_sql_expr(self) -> &'static str {
        match self {
            Self::Account => "account",
            Self::Product => "product",
            Self::FundName => "fund_name",
            Self::TradeDateYear => "EXTRACT(YEAR FROM trade_date)::integer::text",
            Self::TradeDateYearMonth => "TO_CHAR(trade_date, 'YYYY-MM')",
            Self::SettlementDateYear => "EXTRACT(YEAR FROM settlement_date)::integer::text",
            Self::SettlementDateYearMonth => "TO_CHAR(settlement_date, 'YYYY-MM')",
        }
    }
}

/// table / group_expr（呼び出し側が渡す固定の &'static str のみ）を使って
/// `SELECT ... GROUP BY ... ORDER BY ...` の QueryBuilder を組み立てる。
/// push_filters は WHERE 句（user_id を含む検索条件）を積むクロージャ。
fn build_group_facets_query(
    table: &'static str,
    group_expr: &'static str,
    order: FacetOrder,
    push_filters: impl FnOnce(&mut QueryBuilder<Postgres>),
) -> QueryBuilder<Postgres> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(format!(
        "SELECT {group_expr} AS value, {group_expr} AS label, COUNT(*) AS count FROM {table}"
    ));
    push_filters(&mut qb);
    qb.push(format!(
        " GROUP BY {group_expr} ORDER BY {group_expr} {}",
        order.as_sql()
    ));
    qb
}

/// group_expr の値ごとに件数を集計して FacetOption を返す共通ヘルパー。
/// table / group_expr / order は呼び出し側が定義する固定値のみを渡すこと。
pub async fn fetch_group_facets(
    pool: &PgPool,
    table: &'static str,
    group_expr: &'static str,
    order: FacetOrder,
    push_filters: impl FnOnce(&mut QueryBuilder<Postgres>),
) -> Result<Vec<FacetOption>, ApiError> {
    let mut qb = build_group_facets_query(table, group_expr, order, push_filters);
    Ok(qb.build_query_as::<FacetOption>().fetch_all(pool).await?)
}

/// table / label_order（呼び出し側が渡す固定の &'static str のみ）を使って
/// `security_code` ごとの facet 集計クエリを組み立てる。
/// push_filters は WHERE 句（user_id を含む検索条件）を積むクロージャ。
fn build_security_facets_query(
    table: &'static str,
    label_order: &'static str,
    push_filters: impl FnOnce(&mut QueryBuilder<Postgres>),
) -> QueryBuilder<Postgres> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(format!(
        "SELECT security_code AS value, \
         (ARRAY_AGG(security_name ORDER BY {label_order}))[1] AS label, \
         COUNT(*) AS count \
         FROM {table}"
    ));
    push_filters(&mut qb);
    qb.push(" GROUP BY security_code ORDER BY security_code");
    qb
}

/// security_code の値ごとに security_name（label_order で選択）・件数を集計して
/// FacetOption を返す共通ヘルパー。
/// table / label_order は呼び出し側が定義する固定値のみを渡すこと。
pub async fn fetch_security_facets(
    pool: &PgPool,
    table: &'static str,
    label_order: &'static str,
    push_filters: impl FnOnce(&mut QueryBuilder<Postgres>),
) -> Result<Vec<FacetOption>, ApiError> {
    let mut qb = build_security_facets_query(table, label_order, push_filters);
    Ok(qb.build_query_as::<FacetOption>().fetch_all(pool).await?)
}

#[cfg(test)]
mod tests {
    use super::{build_group_facets_query, build_security_facets_query, FacetOrder, GroupField};
    use uuid::Uuid;

    #[test]
    fn test_group_field_as_sql_expr_preserves_legacy_sql_strings() {
        assert_eq!(GroupField::Account.as_sql_expr(), "account");
        assert_eq!(GroupField::Product.as_sql_expr(), "product");
        assert_eq!(GroupField::FundName.as_sql_expr(), "fund_name");
        assert_eq!(
            GroupField::TradeDateYear.as_sql_expr(),
            "EXTRACT(YEAR FROM trade_date)::integer::text"
        );
        assert_eq!(
            GroupField::TradeDateYearMonth.as_sql_expr(),
            "TO_CHAR(trade_date, 'YYYY-MM')"
        );
        assert_eq!(
            GroupField::SettlementDateYear.as_sql_expr(),
            "EXTRACT(YEAR FROM settlement_date)::integer::text"
        );
        assert_eq!(
            GroupField::SettlementDateYearMonth.as_sql_expr(),
            "TO_CHAR(settlement_date, 'YYYY-MM')"
        );
    }

    #[test]
    fn test_build_group_facets_query_uses_given_table_group_expr_order_and_filters() {
        let qb = build_group_facets_query("dividends", "product", FacetOrder::Asc, |qb| {
            qb.push(" WHERE user_id = ").push_bind(Uuid::nil());
        });
        let sql = qb.sql();
        let sql = sql.as_str();

        assert!(sql.starts_with(
            "SELECT product AS value, product AS label, COUNT(*) AS count FROM dividends"
        ));
        assert!(sql.contains("WHERE user_id = "));
        assert!(sql.ends_with("GROUP BY product ORDER BY product ASC"));
    }

    #[test]
    fn test_build_group_facets_query_desc_order_and_no_filters() {
        let qb = build_group_facets_query("mutualfunds", "account", FacetOrder::Desc, |_| {});
        let sql = qb.sql();
        let sql = sql.as_str();

        assert!(sql.starts_with(
            "SELECT account AS value, account AS label, COUNT(*) AS count FROM mutualfunds"
        ));
        assert!(sql.ends_with("GROUP BY account ORDER BY account DESC"));
    }

    #[test]
    fn test_build_security_facets_query_uses_given_table_label_order_and_filters() {
        let qb = build_security_facets_query("dividends", "settlement_date DESC, id DESC", |qb| {
            qb.push(" WHERE user_id = ").push_bind(Uuid::nil());
        });
        let sql = qb.sql();
        let sql = sql.as_str();

        assert!(sql.starts_with(
            "SELECT security_code AS value, \
             (ARRAY_AGG(security_name ORDER BY settlement_date DESC, id DESC))[1] AS label, \
             COUNT(*) AS count FROM dividends"
        ));
        assert!(sql.contains("WHERE user_id = "));
        assert!(sql.ends_with("GROUP BY security_code ORDER BY security_code"));
    }

    #[test]
    fn test_build_security_facets_query_with_no_filters() {
        let qb = build_security_facets_query("asset_balances", "id", |_| {});
        let sql = qb.sql();
        let sql = sql.as_str();

        assert!(sql.starts_with(
            "SELECT security_code AS value, \
             (ARRAY_AGG(security_name ORDER BY id))[1] AS label, \
             COUNT(*) AS count FROM asset_balances"
        ));
        assert!(sql.ends_with("GROUP BY security_code ORDER BY security_code"));
    }
}
