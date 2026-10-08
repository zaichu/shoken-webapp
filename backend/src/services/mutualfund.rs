use crate::db::{Bind, Db, DbError, QueryBuilder};
use crate::errors::ApiError;
use crate::models::common::{BulkCreateResponse, FacetOption, SearchFacets};
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
use crate::services::domain::bulk::{
    BulkTimer, RowLimit, ensure_user_row_limit_with, lock_user_domain, user_ids_for_bulk_insert,
};
use crate::services::domain::facets::{self, FacetOrder, GroupField};
use crate::services::domain::search::Search;
use crate::services::domain::search_filters::{
    DateAxisFilter, push_search_filters, tokens_from_query,
};
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

    async fn delete_rows(pool: &Db, user_id: UserId) -> Result<u64, DbError> {
        crate::db::query(
            "DELETE FROM mutualfunds WHERE user_id = $1",
            vec![Bind::from(user_id)],
        )
        .execute(pool)
        .await
    }
}

impl CsvImport for MutualfundDomain {
    type Row = CreateMutualfundRequest;
    const CSV_CONFIG: CsvParserConfig = MUTUALFUND_CSV_CONFIG;

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
        transform_mutualfund_rows(table)
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

const MUTUALFUND_CSV_CONFIG: CsvParserConfig = CsvParserConfig {
    skip_header_rows: 0,
    exclude_row_fn: None,
};

/// 投資信託検索条件を SQL 条件へ変換した中間表現
#[derive(Debug)]
pub struct MutualfundFilter {
    date_axis: DateAxisFilter,
    tokens: Vec<String>,
    account: Option<String>,
    fund_name: Option<String>,
    dividends: Option<String>,
}

impl TryFrom<MutualfundSearchQueryParams> for MutualfundFilter {
    type Error = ApiError;

    fn try_from(params: MutualfundSearchQueryParams) -> Result<Self, ApiError> {
        let date_axis = DateAxisFilter::from_search_params(&params.search)?;
        let tokens = tokens_from_query(params.search.q.as_deref());

        Ok(Self {
            date_axis,
            tokens,
            account: params.account,
            fund_name: params.fund_name,
            dividends: params.dividends,
        })
    }
}

impl Search for MutualfundDomain {
    type Data = Mutualfund;
    type Params = MutualfundSearchQueryParams;
    type Filter = MutualfundFilter;
    type Summary = MutualfundSummary;

    const COLUMNS: &'static str = "id, user_id, trade_date, settlement_date, fund_name, dividends, \
        account, shares, exchange_rate, cancellation_unit_price_yen, cancellation_amount_yen, \
        average_acquisition_price_yen, realized_profit_and_loss, taxes, \
        realized_profit_and_loss_after_tax, created_at, updated_at";
    const ORDER_BY: &'static str = " ORDER BY trade_date DESC, id DESC";

    fn push_filters(qb: &mut QueryBuilder, user_id: UserId, filter: &MutualfundFilter) {
        push_search_filters(
            qb,
            user_id,
            Some(("trade_date", &filter.date_axis)),
            &[
                ("account", &filter.account),
                ("fund_name", &filter.fund_name),
                ("dividends", &filter.dividends),
            ],
            &filter.tokens,
            &["account", "fund_name", "dividends"],
        );
    }

    /// 検索条件全体の summary を算出する（ページ内だけでなく検索条件全体の合計）
    async fn fetch_summary(
        pool: &Db,
        user_id: UserId,
        filter: &MutualfundFilter,
    ) -> Result<MutualfundSummary, ApiError> {
        let mut qb = QueryBuilder::new(
            "SELECT COALESCE(SUM(realized_profit_and_loss), 0) AS total_realized_profit_and_loss, \
             COALESCE(SUM(taxes), 0) AS total_taxes, \
             COALESCE(SUM(realized_profit_and_loss_after_tax), 0) AS total_realized_profit_and_loss_after_tax \
             FROM mutualfunds",
        );
        Self::push_filters(&mut qb, user_id, filter);
        Ok(qb
            .build_query_as::<MutualfundSummary>()
            .fetch_one(pool)
            .await?)
    }

    async fn fetch_facets(
        pool: &Db,
        user_id: UserId,
        filter: &MutualfundFilter,
    ) -> Result<SearchFacets, ApiError> {
        let accounts_fut =
            fetch_group_facets(pool, user_id, filter, GroupField::Account, FacetOrder::Asc);
        let funds_fut =
            fetch_group_facets(pool, user_id, filter, GroupField::FundName, FacetOrder::Asc);
        let years_fut = fetch_group_facets(
            pool,
            user_id,
            filter,
            GroupField::TradeDateYear,
            FacetOrder::Desc,
        );
        let year_months_fut = fetch_group_facets(
            pool,
            user_id,
            filter,
            GroupField::TradeDateYearMonth,
            FacetOrder::Desc,
        );

        #[cfg(not(target_arch = "wasm32"))]
        let (accounts, funds, years, year_months) =
            tokio::try_join!(accounts_fut, funds_fut, years_fut, year_months_fut)?;
        #[cfg(target_arch = "wasm32")]
        let (accounts, funds, years, year_months) =
            futures_util::future::try_join4(accounts_fut, funds_fut, years_fut, year_months_fut)
                .await?;

        Ok(SearchFacets {
            products: None,
            accounts: Some(accounts),
            securities: None,
            funds: Some(funds),
            years: Some(years),
            year_months: Some(year_months),
        })
    }
}

async fn fetch_group_facets(
    pool: &Db,
    user_id: UserId,
    filter: &MutualfundFilter,
    group_field: GroupField,
    order: FacetOrder,
) -> Result<Vec<FacetOption>, ApiError> {
    facets::fetch_group_facets(
        pool,
        MutualfundDomain::TABLE,
        group_field.as_sql_expr(),
        order,
        |qb| MutualfundDomain::push_filters(qb, user_id, filter),
    )
    .await
}

/// 投資信託を一括追加（重複はスキップ）
pub async fn bulk_create(
    pool: &Db,
    user_id: UserId,
    items: &[CreateMutualfundRequest],
    limit: RowLimit,
) -> Result<BulkCreateResponse, ApiError> {
    let timer = match BulkTimer::new_with_guard(MutualfundDomain::NAME, items) {
        Ok(t) => t,
        Err(empty) => return Ok(empty),
    };

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

    let mut tx = pool.begin().await?;

    lock_user_domain::<MutualfundDomain>(&mut tx, user_id).await?;

    ensure_user_row_limit_with::<MutualfundDomain, _>(&mut tx, user_id, items.len(), limit).await?;

    let result = crate::db::query(
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
    )
    .execute(&mut tx)
    .await?;

    tx.commit().await?;
    timer.finish_from_affected(result)
}

fn transform_mutualfund_rows(table: &CsvTable) -> (Vec<CreateMutualfundRequest>, Vec<CsvRowError>) {
    validate_csv_rows(table, transform_mutualfund_row)
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
    use crate::models::common::SearchParamsAccessor;
    use crate::services::csv::import::CsvImport;
    use chrono::NaiveDate;
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

    #[test]
    fn test_mutualfund_filter_from_params() {
        let mut params = MutualfundSearchQueryParams::default();
        assert!(!params.should_include_summary());
        assert!(!params.should_include_facets());

        params.search.include_summary = Some(true);
        params.search.include_facets = Some(true);
        assert!(params.should_include_summary());
        assert!(params.should_include_facets());

        let mut params = MutualfundSearchQueryParams::default();
        params.search.date = Some("2026-01-15".to_string());
        params.search.date_from = Some("2026-01-01".to_string());
        params.search.date_to = Some("2026-12-31".to_string());
        params.search.year = Some(2026);
        params.search.year_month = Some("2026-06".to_string());

        let filter =
            MutualfundFilter::try_from(params).expect("正しい日付フォーマットは検証を通過する");

        assert_eq!(
            filter.date_axis.date_eq,
            NaiveDate::from_ymd_opt(2026, 1, 15)
        );
        assert_eq!(
            filter.date_axis.date_from,
            NaiveDate::from_ymd_opt(2026, 1, 1)
        );
        assert_eq!(
            filter.date_axis.date_to,
            NaiveDate::from_ymd_opt(2026, 12, 31)
        );
        assert_eq!(
            filter.date_axis.year_range,
            Some((
                NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2027, 1, 1).unwrap()
            ))
        );
        assert_eq!(
            filter.date_axis.year_month_range,
            Some((
                NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
                NaiveDate::from_ymd_opt(2026, 7, 1).unwrap()
            ))
        );

        type Apply = fn(&mut MutualfundSearchQueryParams);
        let cases: [(&str, Apply); 5] = [
            ("date", |p| p.search.date = Some("2026/01/15".to_string())),
            ("date_from", |p| {
                p.search.date_from = Some("20260101".to_string())
            }),
            ("date_to", |p| {
                p.search.date_to = Some("not-a-date".to_string())
            }),
            ("year", |p| p.search.year = Some(i32::MIN)),
            ("year_month", |p| {
                p.search.year_month = Some("2026-13".to_string())
            }),
        ];

        for (field, apply) in cases {
            let mut params = MutualfundSearchQueryParams::default();
            apply(&mut params);
            let err = MutualfundFilter::try_from(params)
                .expect_err(&format!("{field} は不正値で ValidationError になるべき"));
            assert!(
                matches!(err, ApiError::Validation(_)),
                "field={field} の失敗が ValidationError ではない"
            );
        }
    }

    #[test]
    fn test_push_filters_combines_q_tokens_and_domain_fields() {
        let mut params = MutualfundSearchQueryParams::default();
        params.search.q = Some("AA BB".to_string());
        let filter = MutualfundFilter::try_from(params).expect("q のみなら検証を通過する");

        let mut qb = QueryBuilder::new("SELECT 1 FROM mutualfunds");
        MutualfundDomain::push_filters(&mut qb, UserId::default(), &filter);
        let sql = qb.sql();

        // token ごとに 1 つの AND (...) ブロックが生成され、ブロック内は3カラムの OR になる
        assert_eq!(sql.matches(" AND (").count(), 2);
        assert_eq!(sql.matches("account ILIKE").count(), 2);
        assert_eq!(sql.matches("fund_name ILIKE").count(), 2);
        assert_eq!(sql.matches("dividends ILIKE").count(), 2);

        let params = MutualfundSearchQueryParams {
            account: Some("特定".to_string()),
            fund_name: Some("eMAXIS Slim".to_string()),
            dividends: Some("再投資型".to_string()),
            ..Default::default()
        };
        let filter = MutualfundFilter::try_from(params).expect("フィルタのみなら検証を通過する");

        let mut qb = QueryBuilder::new("SELECT 1 FROM mutualfunds");
        MutualfundDomain::push_filters(&mut qb, UserId::default(), &filter);
        let sql = qb.sql();

        assert!(sql.contains("AND account = "));
        assert!(sql.contains("AND fund_name = "));
        assert!(sql.contains("AND dividends = "));
    }
}
