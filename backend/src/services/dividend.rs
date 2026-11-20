use crate::db::{Bind, Db, FromRow, Tx};
use crate::errors::ApiError;
use crate::models::common::{BulkCreateResponse, FacetOption, SearchFacets};
use crate::models::csv_import::CsvRowError;
use crate::models::dividend::{
    CreateDividendRequest, Dividend, DividendSearchQueryParams, DividendSummary,
};
use crate::services::csv::import;
use crate::services::csv::import::CsvImport;
use crate::services::csv::pipeline::{CsvParserConfig, CsvTable};
use crate::services::csv::util;
use crate::services::csv::util::{CellError, CsvRowView, RowNumber};
use crate::services::domain::bulk;
use crate::services::domain::bulk::RowLimit;
use crate::services::domain::facets::{FacetOrder, FacetSlot, FacetSpec, GroupField};
use crate::services::domain::search::{AggregateResult, Search};
use crate::services::domain::search_filters;
use crate::services::domain::search_filters::{FilterField, SearchFilter};
use crate::services::domain::{Domain, WriteMode};
use rust_decimal::Decimal;
use shared::normalize::normalize_security_name;
use shared::value::UserId;

/// 配当金ドメイン
pub struct DividendDomain;

impl Domain for DividendDomain {
    const NAME: &'static str = "dividend";
    const TABLE: &'static str = "dividends";
    const WRITE_MODE: WriteMode = WriteMode::Append;
}

impl CsvImport for DividendDomain {
    type Row = CreateDividendRequest;
    const CSV_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 0,
        exclude_row_fn: None,
        // 証券会社の新形式は表題行が先頭に付くため、見出しは列名で検出する
        header_marker: Some("入金日"),
    };

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
        import::validate_csv_rows(table, transform_dividend_row)
    }

    async fn bulk_create(
        pool: &Db,
        user_id: UserId,
        items: &[Self::Row],
        limit: RowLimit,
    ) -> Result<BulkCreateResponse, ApiError> {
        bulk_create(pool, user_id, items, limit).await
    }
}

const SUMMARY_SQL: &str = "SELECT COALESCE(SUM(dividends_before_tax), 0) AS total_dividends_before_tax, \
     COALESCE(SUM(taxes), 0) AS total_taxes, \
     COALESCE(SUM(net_amount_received), 0) AS total_net_amount_received \
     FROM dividends";

const AGGREGATE_SELECT_SQL: &str = "SELECT slot, value, value AS label, count, total_dividends_before_tax, total_taxes, total_net_amount_received FROM (SELECT CASE WHEN GROUPING(product) = 0 THEN 0 WHEN GROUPING(account) = 0 THEN 1 WHEN GROUPING(security_code) = 0 THEN 2 WHEN GROUPING(EXTRACT(YEAR FROM settlement_date)::integer::text) = 0 THEN 3 WHEN GROUPING(TO_CHAR(settlement_date, 'YYYY-MM')) = 0 THEN 4 ELSE 5 END AS slot, COALESCE(product, account, security_code, EXTRACT(YEAR FROM settlement_date)::integer::text, TO_CHAR(settlement_date, 'YYYY-MM')) AS value, COUNT(*) AS count, COALESCE(SUM(dividends_before_tax), 0) AS total_dividends_before_tax, COALESCE(SUM(taxes), 0) AS total_taxes, COALESCE(SUM(net_amount_received), 0) AS total_net_amount_received FROM dividends";

const AGGREGATE_GROUP_SQL: &str = " GROUP BY GROUPING SETS ((product), (account), (security_code), (EXTRACT(YEAR FROM settlement_date)::integer::text), (TO_CHAR(settlement_date, 'YYYY-MM')), ())";

const AGGREGATE_ORDER_SQL: &str =
    " ) AS aggregates ORDER BY slot, CASE WHEN slot IN (3, 4) THEN value END DESC, value ASC";

/// 銘柄 facet の最新名だけを取る問い合わせ。集計とは別に DISTINCT ON で最新行を選ぶ
const LATEST_SECURITY_NAME_SQL: &str =
    "SELECT DISTINCT ON (security_code) security_code, security_name FROM dividends";

struct DividendAggregateRow {
    slot: i32,
    value: Option<String>,
    label: Option<String>,
    count: i64,
    total_dividends_before_tax: crate::db::Numeric,
    total_taxes: crate::db::Numeric,
    total_net_amount_received: crate::db::Numeric,
}

impl FromRow for DividendAggregateRow {
    fn from_row(row: &crate::db::Row) -> Result<Self, crate::db::DbError> {
        Ok(Self {
            slot: row.try_get("slot")?,
            value: row.try_get("value")?,
            label: row.try_get("label")?,
            count: row.try_get("count")?,
            total_dividends_before_tax: row.try_get("total_dividends_before_tax")?,
            total_taxes: row.try_get("total_taxes")?,
            total_net_amount_received: row.try_get("total_net_amount_received")?,
        })
    }
}

impl Search for DividendDomain {
    type Data = Dividend;
    type Params = DividendSearchQueryParams;
    type Summary = DividendSummary;

    const COLUMNS: &'static str = "id, user_id, settlement_date, product, account, security_code, \
        security_name, unit_price, shares, dividends_before_tax, taxes, net_amount_received, \
        created_at, updated_at";
    const ORDER_BY: &'static str = " ORDER BY settlement_date DESC, id DESC";
    const DATE_COLUMN: Option<&'static str> = Some("settlement_date");
    const FILTER_FIELDS: &'static [FilterField] = &[
        FilterField::Product,
        FilterField::Account,
        FilterField::SecurityCode,
        FilterField::SecurityName,
    ];
    const FACETS: &'static [FacetSpec] = &[
        FacetSpec::group(FacetSlot::Products, GroupField::Product, FacetOrder::Asc),
        FacetSpec::group(FacetSlot::Accounts, GroupField::Account, FacetOrder::Asc),
        FacetSpec::security(FacetSlot::Securities, "settlement_date DESC, id DESC"),
        FacetSpec::group(
            FacetSlot::Years,
            GroupField::SettlementDateYear,
            FacetOrder::Desc,
        ),
        FacetSpec::group(
            FacetSlot::YearMonths,
            GroupField::SettlementDateYearMonth,
            FacetOrder::Desc,
        ),
    ];

    fn filter_value(params: &DividendSearchQueryParams, field: FilterField) -> &Option<String> {
        match field {
            FilterField::Product => &params.product,
            FilterField::Account => &params.account,
            FilterField::SecurityCode => &params.security_code,
            FilterField::SecurityName => &params.security_name,
            _ => &search_filters::NO_FIELD,
        }
    }

    async fn fetch_summary(
        tx: &mut Tx,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> Result<DividendSummary, ApiError> {
        Ok(Self::filtered_query(SUMMARY_SQL, user_id, filter)
            .build_query_as::<DividendSummary>()
            .fetch_one(&mut *tx)
            .await?)
    }

    async fn fetch_aggregates(
        tx: &mut Tx,
        user_id: UserId,
        filter: &SearchFilter,
        include_summary: bool,
        include_facets: bool,
    ) -> Result<AggregateResult<DividendSummary>, ApiError> {
        if !include_summary && !include_facets {
            let mut count_qb = Self::count_query(user_id, filter);
            let total: i64 = count_qb.build_query_scalar().fetch_one(&mut *tx).await?;
            return Ok((total, None, None));
        }
        let mut aggregate_qb = Self::filtered_query(AGGREGATE_SELECT_SQL, user_id, filter);
        aggregate_qb.push(AGGREGATE_GROUP_SQL);
        aggregate_qb.push(AGGREGATE_ORDER_SQL);
        let mut rows = aggregate_qb
            .build_query_as::<DividendAggregateRow>()
            .fetch_all(&mut *tx)
            .await?;
        // 銘柄の最新名は集計とは別の DISTINCT ON で取り、Rust 側で上書きする
        // (ARRAY_AGG を GROUPING SETS 内で回すと全行ソートが重いため)
        if include_facets {
            let mut name_qb = Self::filtered_query(LATEST_SECURITY_NAME_SQL, user_id, filter);
            name_qb.push(" ORDER BY security_code, settlement_date DESC, id DESC");
            let names = name_qb
                .build_query_as::<(String, String)>()
                .fetch_all(&mut *tx)
                .await?;
            let latest: std::collections::HashMap<&str, &str> = names
                .iter()
                .map(|(code, name)| (code.as_str(), name.as_str()))
                .collect();
            for row in &mut rows {
                if row.slot == 2
                    && let Some(value) = row.value.as_deref()
                    && let Some(name) = latest.get(value)
                {
                    row.label = Some((*name).to_string());
                }
            }
        }
        let mut total = 0;
        let mut summary = None;
        let mut products = Vec::new();
        let mut accounts = Vec::new();
        let mut securities = Vec::new();
        let mut years = Vec::new();
        let mut year_months = Vec::new();
        for row in rows {
            if row.slot == 5 {
                total = row.count;
                if include_summary {
                    summary = Some(DividendSummary {
                        total_dividends_before_tax: row.total_dividends_before_tax.0,
                        total_taxes: row.total_taxes.0,
                        total_net_amount_received: row.total_net_amount_received.0,
                    });
                }
            } else if include_facets {
                let option = FacetOption {
                    value: row.value.ok_or(crate::db::DbError::RowNotFound)?,
                    label: row.label.ok_or(crate::db::DbError::RowNotFound)?,
                    count: Some(row.count),
                };
                match row.slot {
                    0 => products.push(option),
                    1 => accounts.push(option),
                    2 => securities.push(option),
                    3 => years.push(option),
                    4 => year_months.push(option),
                    _ => {}
                }
            }
        }
        let facets = if include_facets {
            Some(SearchFacets {
                products: Some(products),
                accounts: Some(accounts),
                securities: Some(securities),
                funds: None,
                years: Some(years),
                year_months: Some(year_months),
            })
        } else {
            None
        };
        Ok((total, summary, facets))
    }
}

/// 配当金を一括追加（全件挿入）
/// 同一内容の行が複数ある場合も全件保存する。
///
/// 再アップロード防止:
///   content_hash + グローバル occurrence_index（user_id + content_hash 単位の通し番号）により
///   DB に既存の行と同一内容・同一順序の行はスキップされる。
pub async fn bulk_create(
    pool: &Db,
    user_id: UserId,
    items: &[CreateDividendRequest],
    limit: RowLimit,
) -> Result<BulkCreateResponse, ApiError> {
    let user_ids = bulk::user_ids_for_bulk_insert(user_id, items.len());
    let settlement_dates: Vec<time::Date> = items.iter().map(|i| i.settlement_date).collect();
    let products: Vec<String> = items.iter().map(|i| i.product.clone()).collect();
    let accounts: Vec<String> = items
        .iter()
        .map(|i| i.account.as_str().to_owned())
        .collect();
    let security_codes: Vec<String> = items.iter().map(|i| i.security_code.clone()).collect();
    let security_names: Vec<String> = items.iter().map(|i| i.security_name.clone()).collect();
    let unit_prices: Vec<Decimal> = items.iter().map(|i| i.unit_price).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let dividends_before_taxes: Vec<Decimal> =
        items.iter().map(|i| i.dividends_before_tax).collect();
    let taxes: Vec<Decimal> = items.iter().map(|i| i.taxes).collect();
    let net_amounts: Vec<Decimal> = items.iter().map(|i| i.net_amount_received).collect();

    bulk::bulk_insert::<DividendDomain, _>(
        pool,
        user_id,
        items,
        limit,
        crate::db::query(
            r#"
            WITH batch_data AS (
                SELECT
                    user_id, settlement_date, product, account, security_code,
                    security_name, unit_price, shares, dividends_before_tax,
                    taxes, net_amount_received,
                    ordinality,
                    md5(
                        settlement_date::text || '|' || product || '|' || account || '|' ||
                        security_code || '|' || security_name || '|' ||
                        unit_price::text || '|' || shares::text || '|' || dividends_before_tax::text || '|' ||
                        taxes::text || '|' || net_amount_received::text
                    ) AS content_hash
                FROM UNNEST(
                    $1::uuid[], $2::date[], $3::text[], $4::text[], $5::text[],
                    $6::text[], $7::numeric[], $8::numeric[], $9::numeric[],
                    $10::numeric[], $11::numeric[]
                ) WITH ORDINALITY AS t(user_id, settlement_date, product, account, security_code,
                       security_name, unit_price, shares, dividends_before_tax,
                       taxes, net_amount_received, ordinality)
            ),
            batch_indexed AS (
                SELECT *,
                    ROW_NUMBER() OVER (PARTITION BY content_hash ORDER BY ordinality)::int4
                        AS batch_occurrence_index
                FROM batch_data
            ),
            db_counts AS (
                SELECT content_hash, COUNT(*)::int4 AS existing_count
                FROM dividends
                WHERE user_id = $12::uuid
                GROUP BY content_hash
            )
            INSERT INTO dividends (user_id, settlement_date, product, account, security_code,
                                   security_name, unit_price, shares, dividends_before_tax,
                                   taxes, net_amount_received, content_hash, occurrence_index)
            SELECT
                b.user_id, b.settlement_date, b.product, b.account, b.security_code,
                b.security_name, b.unit_price, b.shares, b.dividends_before_tax,
                b.taxes, b.net_amount_received,
                b.content_hash,
                b.batch_occurrence_index
            FROM batch_indexed b
            LEFT JOIN db_counts d ON b.content_hash = d.content_hash
            WHERE b.batch_occurrence_index > COALESCE(d.existing_count, 0)
            ON CONFLICT (user_id, content_hash, occurrence_index) DO NOTHING
            "#,
            vec![
                Bind::from(user_ids),
                Bind::from(settlement_dates),
                Bind::from(products),
                Bind::from(accounts),
                Bind::from(security_codes),
                Bind::from(security_names),
                Bind::decimal_vec(unit_prices),
                Bind::decimal_vec(shares),
                Bind::decimal_vec(dividends_before_taxes),
                Bind::decimal_vec(taxes),
                Bind::decimal_vec(net_amounts),
                // $12: スカラーのユーザーID（db_counts WHERE 句用）
                Bind::from(user_id),
            ],
        ),
    )
    .await
}

// 証券会社CSVの新旧形式で表記が違う列の別名候補。先頭は欠落時エラーの正名
const COL_DATE: &[&str] = &["入金日", "入金日(受渡日)"];
const COL_SHARES: &[&str] = &["数量[株/口]", "数量[株/口/額面]"];
const COL_BEFORE_TAX: &[&str] = &[
    "配当・分配金合計（税引前）[円/現地通貨]",
    "配当金・分配金・利金合計(税引前)[円/現地通貨]",
];
const COL_NET: &[&str] = &["受取金額[円/現地通貨]", "受取金額(税引後)[円/現地通貨]"];
const COL_TAX_TOTAL: &str = "税額合計[円/現地通貨]";
const COL_TAX_FOREIGN: &str = "外国税額[円]";
const COL_TAX_DOMESTIC: &str = "国内税額合計[円]";

fn transform_dividend_row(
    row: &CsvRowView<'_>,
    row_num: RowNumber,
) -> Result<CreateDividendRequest, CsvRowError> {
    // 円限定の現仕様: 受取通貨が円以外の行は明示的な行エラーとして拒否する
    // 「円」「日本円」など表記ゆれを許容するため contains で判定する
    let received_currency = row.cell("受取通貨").trim();
    if received_currency.is_empty() {
        return Err(CsvRowError {
            row: row_num.get(),
            message: "受取通貨が空です".to_string(),
        });
    }
    if !received_currency.contains('円') {
        return Err(CsvRowError {
            row: row_num.get(),
            message: format!("受取通貨: 円のみ取り込めます（{received_currency}）"),
        });
    }

    Ok(CreateDividendRequest {
        settlement_date: util::parse_required_date(row, row.resolve_col(COL_DATE), row_num)?,
        product: util::check_max_chars(
            util::parse_required_string(row, "商品", row_num)?,
            "商品",
            100,
            row_num,
        )?,
        account: util::parse_required_account(row, "口座", row_num)?,
        security_code: util::check_max_chars(
            util::parse_optional_string(row, "銘柄コード"),
            "銘柄コード",
            10,
            row_num,
        )?,
        security_name: util::check_max_chars(
            normalize_security_name(&util::parse_required_string(row, "銘柄", row_num)?),
            "銘柄",
            200,
            row_num,
        )?,
        unit_price: util::parse_required_number(row, "単価[円/現地通貨]", row_num)?,
        shares: util::parse_required_number(row, row.resolve_col(COL_SHARES), row_num)?,
        dividends_before_tax: util::parse_required_number(
            row,
            row.resolve_col(COL_BEFORE_TAX),
            row_num,
        )?,
        taxes: parse_taxes(row, row_num)?,
        net_amount_received: util::parse_required_number(row, row.resolve_col(COL_NET), row_num)?,
    })
}

/// 税額。新形式では外国・国内の2列に分かれるため、税額合計が無いときは2列の和を取る
/// (いずれの列も無いファイルは税額合計の必須列エラーとする)
fn parse_taxes(row: &CsvRowView<'_>, row_num: RowNumber) -> Result<Decimal, CsvRowError> {
    if row.has_column(COL_TAX_TOTAL) {
        return util::parse_required_number(row, COL_TAX_TOTAL, row_num);
    }
    if !row.has_column(COL_TAX_FOREIGN) && !row.has_column(COL_TAX_DOMESTIC) {
        return Err(CsvRowError {
            row: row_num.get(),
            message: CellError::MissingRequired(COL_TAX_TOTAL.to_string()).to_string(),
        });
    }
    Ok(util::parse_optional_number(row, COL_TAX_FOREIGN, row_num)?
        + util::parse_optional_number(row, COL_TAX_DOMESTIC, row_num)?)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::models::csv_import::CsvPreviewResponse;
    use crate::services::csv::import::CsvImport;

    const HEADER: &str = "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]";

    fn preview_from_lines(lines: &[&str]) -> CsvPreviewResponse<CreateDividendRequest> {
        DividendDomain::preview_csv(lines.join("\n").as_bytes()).unwrap()
    }

    #[test]
    fn test_preview_csv_basic() {
        let preview = preview_from_lines(&[
            HEADER,
            "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"9908\",\"架空リース\",\"円\",\"90.00\",\"200\",\"18,000\",\"3,656\",\"14,344\"",
            "\"2025/12/10\",\"国内株式\",\"特定・一般\",\"9907\",\"架空フィナンシャル・グループ\",\"円\",\"50.00\",\"300\",\"15,000\",\"3,047\",\"11,953\"",
        ]);
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.is_empty(),
                preview.rows.len(),
            ),
            (2, 2, true, 2)
        );

        assert!(matches!(
            DividendDomain::preview_csv(b""),
            Err(ApiError::Csv(_))
        ));

        let preview = preview_from_lines(&[
            HEADER,
            "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"\",\"A B C D\",\"円\",\"90.00\",\"200\",\"18000\",\"3656\",\"14344\"",
        ]);
        assert_eq!(
            (
                preview.valid_rows,
                preview.rows[0].security_code.as_str(),
                preview.rows[0].security_name.as_str(),
            ),
            (1, "", "ABCD")
        );

        for (header, row, expected_message) in [
            (
                "入金日,商品,口座,銘柄コード,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]",
                "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"9908\",\"円\",\"90.00\",\"200\",\"18,000\",\"3,656\",\"14,344\"",
                "銘柄",
            ),
            (
                HEADER,
                "\"2025/13/09\",\"国内株式\",\"特定・一般\",\"9908\",\"架空リース\",\"円\",\"90.00\",\"200\",\"18000\",\"3656\",\"14344\"",
                "入金日",
            ),
        ] {
            let preview = preview_from_lines(&[header, row]);
            assert_eq!(
                (
                    preview.total_rows,
                    preview.valid_rows,
                    matches!(preview.errors.as_slice(), [error] if error.message.contains(expected_message)),
                ),
                (1, 0, true)
            );
        }
    }

    #[test]
    fn test_preview_csv_rejects_non_jpy_currency() {
        for currency in ["円", "日本円"] {
            let preview = preview_from_lines(&[
                HEADER,
                &format!(
                    "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"9908\",\"架空リース\",\"{currency}\",\"90.00\",\"200\",\"18,000\",\"3,656\",\"14,344\""
                ),
            ]);
            assert_eq!((preview.valid_rows, preview.errors.len()), (1, 0));
        }

        let preview = preview_from_lines(&[
            HEADER,
            "\"2025/12/09\",\"外国株式\",\"特定・一般\",\"AAPL\",\"アップル\",\"米ドル\",\"200.00\",\"10\",\"1,000\",\"100\",\"900\"",
        ]);
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.len(),
                preview.errors[0].row,
                preview.errors[0].message.contains("米ドル"),
            ),
            (1, 0, 1, 1, true)
        );

        // 通貨混合では円の行だけが残り、外貨行は合計の対象に混ざらない
        let preview = preview_from_lines(&[
            HEADER,
            "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"9908\",\"架空リース\",\"円\",\"90.00\",\"200\",\"18,000\",\"3,656\",\"14,344\"",
            "\"2025/12/09\",\"外国株式\",\"特定・一般\",\"AAPL\",\"アップル\",\"米ドル\",\"200.00\",\"10\",\"1,000\",\"100\",\"900\"",
        ]);
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.len(),
                preview.errors[0].row,
            ),
            (2, 1, 1, 2)
        );
        assert_eq!(preview.rows[0].security_code.as_str(), "9908");
    }

    #[test]
    fn test_preview_csv_new_format_fixture() {
        // 表題行つき・列名変更・税額2列の新形式(Shift-JIS)。架空データの fixture
        let preview = DividendDomain::preview_csv(include_bytes!(
            "../../tests/fixtures/dividend-new-format.csv"
        ))
        .unwrap();
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.is_empty()
            ),
            (5, 5, true)
        );
        let last = &preview.rows[4];
        assert_eq!(last.security_code.as_str(), "9931");
        // 税額は 外国税額500 + 国内税額2,031
        assert_eq!(last.taxes, rust_decimal_macros::dec!(2531));
        assert_eq!(last.net_amount_received, rust_decimal_macros::dec!(7469));
    }

    #[test]
    fn test_preview_csv_title_line_before_old_header() {
        // 表題行があっても旧列名のファイルは取り込める
        let preview = preview_from_lines(&[
            "配当金・分配金・利金一覧",
            HEADER,
            "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"9908\",\"架空リース\",\"円\",\"90.00\",\"200\",\"18,000\",\"3,656\",\"14,344\"",
        ]);
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.is_empty()
            ),
            (1, 1, true)
        );
    }

    #[test]
    fn test_preview_csv_without_header_fails_with_single_error() {
        // 見出し行が見つからないときは行エラーではなくファイル全体で1件のエラー
        let result = DividendDomain::preview_csv("商品,口座\n\"国内株式\",\"特定\"".as_bytes());
        assert!(matches!(
            result,
            Err(ApiError::Csv(crate::errors::CsvError::HeaderNotFound(_)))
        ));
    }

    #[test]
    fn test_preview_csv_empty_currency_message() {
        // 受取通貨が空のときは値を括弧に入れない専用メッセージにする
        let preview = preview_from_lines(&[
            HEADER,
            "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"9908\",\"架空リース\",\"\",\"90.00\",\"200\",\"18,000\",\"3,656\",\"14,344\"",
        ]);
        assert_eq!(
            (
                preview.valid_rows,
                preview.errors.len(),
                preview.errors[0].message.as_str(),
            ),
            (0, 1, "受取通貨が空です")
        );
    }
}
