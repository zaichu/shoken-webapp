use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::CsvRowError;
use crate::models::mutualfund::{
    CreateMutualfundRequest, Mutualfund, MutualfundSearchQueryParams, MutualfundSummary,
};
use crate::services::csv::import::{CsvImport, validate_csv_rows};
use crate::services::csv::pipeline::{CsvParserConfig, CsvTable};
use crate::services::csv::util::{
    CsvCells, CsvRowView, RowNumber, check_max_chars, parse_required_account, parse_required_date,
    parse_required_number, parse_required_string,
};
use crate::services::domain::bulk::{RowLimit, bulk_insert, user_ids_for_bulk_insert};
use crate::services::domain::facets::{FacetOrder, FacetSlot, FacetSpec, GroupField};
use crate::services::domain::search::Search;
use crate::services::domain::search_filters::{FilterField, NO_FIELD, SearchFilter};
use crate::services::domain::{Domain, WriteMode};
use rust_decimal::Decimal;
use shared::tax::compute_taxes;
use shared::value::UserId;

/// 投資信託ドメイン
pub struct MutualfundDomain;

impl Domain for MutualfundDomain {
    const NAME: &'static str = "mutualfund";
    const TABLE: &'static str = "mutualfunds";
    const WRITE_MODE: WriteMode = WriteMode::Append;
}

impl CsvImport for MutualfundDomain {
    type Row = CreateMutualfundRequest;
    const CSV_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 0,
        exclude_row_fn: None,
    };

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
        validate_csv_rows(table, transform_mutualfund_row)
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

const SUMMARY_SQL: &str = "SELECT COALESCE(SUM(realized_profit_and_loss), 0) AS total_realized_profit_and_loss, \
     COALESCE(SUM(taxes), 0) AS total_taxes, \
     COALESCE(SUM(realized_profit_and_loss_after_tax), 0) AS total_realized_profit_and_loss_after_tax \
     FROM mutualfunds";

impl Search for MutualfundDomain {
    type Data = Mutualfund;
    type Params = MutualfundSearchQueryParams;
    type Summary = MutualfundSummary;

    const COLUMNS: &'static str = "id, user_id, trade_date, settlement_date, fund_name, dividends, \
        account, shares, exchange_rate, cancellation_unit_price_yen, cancellation_amount_yen, \
        average_acquisition_price_yen, realized_profit_and_loss, taxes, \
        realized_profit_and_loss_after_tax, created_at, updated_at";
    const ORDER_BY: &'static str = " ORDER BY trade_date DESC, id DESC";
    const DATE_COLUMN: Option<&'static str> = Some("trade_date");
    const FILTER_FIELDS: &'static [FilterField] = &[
        FilterField::Account,
        FilterField::FundName,
        FilterField::Dividends,
    ];
    const FACETS: &'static [FacetSpec] = &[
        FacetSpec::group(FacetSlot::Accounts, GroupField::Account, FacetOrder::Asc),
        FacetSpec::group(FacetSlot::Funds, GroupField::FundName, FacetOrder::Asc),
        FacetSpec::group(
            FacetSlot::Years,
            GroupField::TradeDateYear,
            FacetOrder::Desc,
        ),
        FacetSpec::group(
            FacetSlot::YearMonths,
            GroupField::TradeDateYearMonth,
            FacetOrder::Desc,
        ),
    ];

    fn filter_value(params: &MutualfundSearchQueryParams, field: FilterField) -> &Option<String> {
        match field {
            FilterField::Account => &params.account,
            FilterField::FundName => &params.fund_name,
            FilterField::Dividends => &params.dividends,
            _ => &NO_FIELD,
        }
    }

    /// 検索条件全体の summary を算出する（ページ内だけでなく検索条件全体の合計）
    async fn fetch_summary(
        pool: &Db,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> Result<MutualfundSummary, ApiError> {
        Ok(Self::filtered_query(SUMMARY_SQL, user_id, filter)
            .build_query_as::<MutualfundSummary>()
            .fetch_one(pool)
            .await?)
    }
}

/// 投資信託を一括追加（重複はスキップ）
pub async fn bulk_create(
    pool: &Db,
    user_id: UserId,
    items: &[CreateMutualfundRequest],
    limit: RowLimit,
) -> Result<BulkCreateResponse, ApiError> {
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let trade_dates: Vec<chrono::NaiveDate> = items.iter().map(|i| i.trade_date).collect();
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let fund_names: Vec<String> = items.iter().map(|i| i.fund_name.clone()).collect();
    let dividends: Vec<Option<String>> = items.iter().map(|i| i.dividends.clone()).collect();
    let accounts: Vec<String> = items
        .iter()
        .map(|i| i.account.as_str().to_owned())
        .collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let exchange_rates: Vec<Decimal> = items.iter().map(|i| i.exchange_rate).collect();
    let cancellation_unit_prices: Vec<Decimal> = items
        .iter()
        .map(|i| i.cancellation_unit_price_yen)
        .collect();
    let cancellation_amounts: Vec<Decimal> =
        items.iter().map(|i| i.cancellation_amount_yen).collect();
    let avg_acquisition_prices: Vec<Decimal> = items
        .iter()
        .map(|i| i.average_acquisition_price_yen)
        .collect();
    let realized_pls: Vec<Decimal> = items.iter().map(|i| i.realized_profit_and_loss).collect();
    let taxes: Vec<Decimal> = items.iter().map(|i| i.taxes).collect();
    let realized_pls_after_tax: Vec<Decimal> = items
        .iter()
        .map(|i| i.realized_profit_and_loss_after_tax)
        .collect();

    bulk_insert::<MutualfundDomain, _>(
        pool,
        user_id,
        items,
        limit,
        crate::db::query(
            r#"
            INSERT INTO mutualfunds (user_id, trade_date, settlement_date, fund_name, dividends,
                                     account, shares, exchange_rate, cancellation_unit_price_yen,
                                     cancellation_amount_yen, average_acquisition_price_yen,
                                     realized_profit_and_loss, taxes, realized_profit_and_loss_after_tax)
            SELECT * FROM UNNEST(
                $1::uuid[], $2::date[], $3::date[], $4::text[], $5::text[],
                $6::text[], $7::numeric[], $8::numeric[], $9::numeric[],
                $10::numeric[], $11::numeric[], $12::numeric[], $13::numeric[], $14::numeric[]
            )
            ON CONFLICT (user_id, trade_date, fund_name, shares, cancellation_amount_yen)
            DO NOTHING
            "#,
            vec![
                Bind::from(user_ids),
                Bind::from(trade_dates),
                Bind::from(settlement_dates),
                Bind::from(fund_names),
                Bind::from(dividends),
                Bind::from(accounts),
                Bind::decimal_vec(shares),
                Bind::decimal_vec(exchange_rates),
                Bind::decimal_vec(cancellation_unit_prices),
                Bind::decimal_vec(cancellation_amounts),
                Bind::decimal_vec(avg_acquisition_prices),
                Bind::decimal_vec(realized_pls),
                Bind::decimal_vec(taxes),
                Bind::decimal_vec(realized_pls_after_tax),
            ],
        ),
    )
    .await
}

fn transform_mutualfund_row(
    row: &CsvRowView<'_>,
    row_num: RowNumber,
) -> Result<CreateMutualfundRequest, CsvRowError> {
    let trade_date = parse_required_date(row, "約定日", row_num)?;
    let settlement_date = parse_required_date(row, "受渡日", row_num)?;
    let account = parse_required_account(row, "口座", row_num)?;
    let realized_pnl = parse_required_number(row, "実現損益［円］", row_num)?;
    let tax = compute_taxes(&account, realized_pnl);
    let dividends_raw = row.cell("分配金");
    let dividends = if dividends_raw.trim().is_empty() {
        None
    } else {
        Some(dividends_raw.to_string())
    };
    Ok(CreateMutualfundRequest {
        trade_date,
        settlement_date,
        fund_name: check_max_chars(
            parse_required_string(row, "ファンド名", row_num)?,
            "ファンド名",
            300,
            row_num,
        )?,
        dividends,
        account,
        shares: parse_required_number(row, "数量[口]", row_num)?,
        exchange_rate: parse_required_number(row, "為替レート［円］", row_num)?,
        cancellation_unit_price_yen: parse_required_number(row, "解約単価［円］", row_num)?,
        cancellation_amount_yen: parse_required_number(row, "解約額［円］", row_num)?,
        average_acquisition_price_yen: parse_required_number(row, "平均取得価額［円］", row_num)?,
        realized_profit_and_loss: realized_pnl,
        taxes: tax.taxes,
        realized_profit_and_loss_after_tax: tax.realized_profit_and_loss_after_tax,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::csv::import::CsvImport;

    #[test]
    fn test_preview_csv_basic() {
        let csv = concat!(
            "約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n",
            "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim 米国株式(S&P500)\",\"再投資型\",\"特定\",\"解約\",\"3,721,147\",\"-\",\"19,661\",\"7,316,147\",\"18,005.20\",\"615,849\""
        );
        let preview = MutualfundDomain::preview_csv(csv.as_bytes()).unwrap();
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.rows.len(),
                preview.errors.is_empty(),
                matches!(MutualfundDomain::preview_csv(b""), Err(ApiError::Csv(_)))
            ),
            (1, 1, 1, true, true)
        );
        let csv = concat!(
            "約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n",
            "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim\",\"\",\"特定\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"615849\""
        );
        let preview = MutualfundDomain::preview_csv(csv.as_bytes()).unwrap();
        assert_eq!(
            (preview.valid_rows, preview.rows[0]["dividends"].is_null()),
            (1, true)
        );
    }
}
