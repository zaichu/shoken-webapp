use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::CsvRowError;
use crate::models::dividend::{
    CreateDividendRequest, Dividend, DividendSearchQueryParams, DividendSummary,
};
use crate::services::csv::import::{CsvImport, validate_csv_rows};
use crate::services::csv::pipeline::{CsvParserConfig, CsvTable};
use crate::services::csv::util::{
    CsvRowView, RowNumber, check_max_chars, parse_optional_string, parse_required_account,
    parse_required_date, parse_required_number, parse_required_string,
};
use crate::services::domain::bulk::{RowLimit, bulk_insert, user_ids_for_bulk_insert};
use crate::services::domain::facets::{FacetOrder, FacetSlot, FacetSpec, GroupField};
use crate::services::domain::search::Search;
use crate::services::domain::search_filters::{FilterField, NO_FIELD, SearchFilter};
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
    };

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
        validate_csv_rows(table, transform_dividend_row)
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
            _ => &NO_FIELD,
        }
    }

    async fn fetch_summary(
        pool: &Db,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> Result<DividendSummary, ApiError> {
        Ok(Self::filtered_query(SUMMARY_SQL, user_id, filter)
            .build_query_as::<DividendSummary>()
            .fetch_one(pool)
            .await?)
    }
}

/// 配当金を一括追加（重複はスキップ）
pub async fn bulk_create(
    pool: &Db,
    user_id: UserId,
    items: &[CreateDividendRequest],
    limit: RowLimit,
) -> Result<BulkCreateResponse, ApiError> {
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
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

    bulk_insert::<DividendDomain, _>(
        pool,
        user_id,
        items,
        limit,
        crate::db::query(
            r#"
            INSERT INTO dividends (user_id, settlement_date, product, account, security_code,
                                   security_name, unit_price, shares, dividends_before_tax,
                                   taxes, net_amount_received)
            SELECT * FROM UNNEST(
                $1::uuid[], $2::date[], $3::text[], $4::text[], $5::text[],
                $6::text[], $7::numeric[], $8::numeric[], $9::numeric[],
                $10::numeric[], $11::numeric[]
            )
            ON CONFLICT (user_id, settlement_date, security_code, security_name, shares, dividends_before_tax)
            DO NOTHING
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
            ],
        ),
    )
    .await
}

fn transform_dividend_row(
    row: &CsvRowView<'_>,
    row_num: RowNumber,
) -> Result<CreateDividendRequest, CsvRowError> {
    Ok(CreateDividendRequest {
        settlement_date: parse_required_date(row, "入金日", row_num)?,
        product: check_max_chars(
            parse_required_string(row, "商品", row_num)?,
            "商品",
            100,
            row_num,
        )?,
        account: parse_required_account(row, "口座", row_num)?,
        security_code: check_max_chars(
            parse_optional_string(row, "銘柄コード"),
            "銘柄コード",
            10,
            row_num,
        )?,
        security_name: check_max_chars(
            normalize_security_name(&parse_required_string(row, "銘柄", row_num)?),
            "銘柄",
            200,
            row_num,
        )?,
        unit_price: parse_required_number(row, "単価[円/現地通貨]", row_num)?,
        shares: parse_required_number(row, "数量[株/口]", row_num)?,
        dividends_before_tax: parse_required_number(
            row,
            "配当・分配金合計（税引前）[円/現地通貨]",
            row_num,
        )?,
        taxes: parse_required_number(row, "税額合計[円/現地通貨]", row_num)?,
        net_amount_received: parse_required_number(row, "受取金額[円/現地通貨]", row_num)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::csv_import::CsvPreviewResponse;
    use crate::services::csv::import::CsvImport;

    const HEADER: &str = "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]";

    fn preview_from_lines(lines: &[&str]) -> CsvPreviewResponse {
        DividendDomain::preview_csv(lines.join("\n").as_bytes()).unwrap()
    }

    #[test]
    fn test_preview_csv_basic() {
        let preview = preview_from_lines(&[
            HEADER,
            "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8591\",\"オリックス\",\"円\",\"93.76\",\"200\",\"18,752\",\"3,808\",\"14,944\"",
            "\"2025/12/10\",\"国内株式\",\"特定・一般\",\"8306\",\"三菱ＵＦＪフィナンシャル・グループ\",\"円\",\"50.00\",\"300\",\"15,000\",\"3,047\",\"11,953\"",
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
            "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"\",\"K D D I\",\"円\",\"93.76\",\"200\",\"18752\",\"3808\",\"14944\"",
        ]);
        assert_eq!(
            (
                preview.valid_rows,
                preview.rows[0]["security_code"].as_str(),
                preview.rows[0]["security_name"].as_str(),
            ),
            (1, Some(""), Some("KDDI"))
        );

        for (header, row, expected_message) in [
            (
                "入金日,商品,口座,銘柄コード,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]",
                "\"2025/12/09\",\"国内株式\",\"特定・一般\",\"8591\",\"円\",\"93.76\",\"200\",\"18,752\",\"3,808\",\"14,944\"",
                "銘柄",
            ),
            (
                HEADER,
                "\"2025/13/09\",\"国内株式\",\"特定・一般\",\"8591\",\"オリックス\",\"円\",\"93.76\",\"200\",\"18752\",\"3808\",\"14944\"",
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
}
