use crate::errors::ApiError;
use crate::models::common::{
    BulkCreateResponse, FacetOption, PaginatedSearchResponse, SearchFacets, SearchParamsAccessor,
};
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::models::mutualfund::{
    CreateMutualfundRequest, Mutualfund, MutualfundSearchQueryParams, MutualfundSummary,
};
use crate::services::csv_import::{build_csv_preview, run_csv_upload, validate_csv_rows};
use crate::services::csv_pipeline::{CsvParserConfig, CsvRow};
use crate::services::csv_util::{
    compute_taxes, get_row_cell, parse_required_date_row, parse_required_number_row,
    parse_required_string_row,
};
use crate::services::search_filters::{push_search_filters, run_paginated_search};
use crate::services::shared::{
    self, delete_all_for_user, tokens_from_query, user_ids_for_bulk_insert, BulkTimer,
    DateAxisFilter, DeleteTarget, FacetOrder,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::info;
use uuid::Uuid;

const MUTUALFUND_CSV_CONFIG: CsvParserConfig = CsvParserConfig {
    skip_header_rows: 0,
    exclude_row_fn: None,
    required_columns: &[
        "約定日",
        "受渡日",
        "ファンド名",
        "分配金",
        "口座",
        "数量[口]",
        "為替レート［円］",
        "解約単価［円］",
        "解約額［円］",
        "平均取得価額［円］",
        "実現損益［円］",
    ],
};

/// 投資信託検索条件を SQL 条件へ変換した中間表現
#[derive(Debug)]
struct MutualfundFilter {
    date_axis: DateAxisFilter,
    tokens: Vec<String>,
    account: Option<String>,
    fund_name: Option<String>,
    dividends: Option<String>,
}

impl MutualfundFilter {
    fn from_params(params: &MutualfundSearchQueryParams) -> Result<Self, ApiError> {
        let search = &params.search;
        let date_axis = DateAxisFilter::from_search_params(search)?;
        let tokens = tokens_from_query(search.q.as_deref());

        Ok(Self {
            date_axis,
            tokens,
            account: params.account.clone(),
            fund_name: params.fund_name.clone(),
            dividends: params.dividends.clone(),
        })
    }
}

/// user_id と検索条件を WHERE 句として QueryBuilder へ積む
fn push_filters(qb: &mut QueryBuilder<Postgres>, user_id: Uuid, filter: &MutualfundFilter) {
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

/// 認証ユーザーの投資信託一覧を検索（ページネーション・summary・facets 対応）
pub async fn search(
    pool: &PgPool,
    user_id: Uuid,
    params: &MutualfundSearchQueryParams,
) -> Result<PaginatedSearchResponse<Mutualfund, MutualfundSummary, SearchFacets>, ApiError> {
    info!("[mutualfund.search] リクエスト受信");
    let filter = MutualfundFilter::from_params(params)?;

    let summary_fut = async {
        if params.should_include_summary() {
            fetch_summary(pool, user_id, &filter).await.map(Some)
        } else {
            Ok(None)
        }
    };

    let facets_fut = async {
        if params.should_include_facets() {
            fetch_facets(pool, user_id, &filter).await.map(Some)
        } else {
            Ok(None)
        }
    };

    run_paginated_search(
        pool,
        "SELECT COUNT(*) FROM mutualfunds",
        "SELECT id, user_id, trade_date, settlement_date, fund_name, dividends, account, \
         shares, exchange_rate, cancellation_unit_price_yen, cancellation_amount_yen, \
         average_acquisition_price_yen, realized_profit_and_loss, taxes, \
         realized_profit_and_loss_after_tax, created_at, updated_at \
         FROM mutualfunds",
        " ORDER BY trade_date DESC, id DESC",
        |qb| push_filters(qb, user_id, &filter),
        params.page(),
        params.per_page(),
        params.offset(),
        summary_fut,
        facets_fut,
    )
    .await
}

/// 検索条件全体の summary を算出する（ページ内だけでなく検索条件全体の合計）
async fn fetch_summary(
    pool: &PgPool,
    user_id: Uuid,
    filter: &MutualfundFilter,
) -> Result<MutualfundSummary, ApiError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT COALESCE(SUM(realized_profit_and_loss), 0) AS total_realized_profit_and_loss, \
         COALESCE(SUM(taxes), 0) AS total_taxes, \
         COALESCE(SUM(realized_profit_and_loss_after_tax), 0) AS total_realized_profit_and_loss_after_tax \
         FROM mutualfunds",
    );
    push_filters(&mut qb, user_id, filter);
    Ok(qb
        .build_query_as::<MutualfundSummary>()
        .fetch_one(pool)
        .await?)
}

async fn fetch_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &MutualfundFilter,
) -> Result<SearchFacets, ApiError> {
    let accounts_fut =
        fetch_group_facets(pool, user_id, filter, GroupField::Account, FacetOrder::Asc);
    let funds_fut =
        fetch_group_facets(pool, user_id, filter, GroupField::FundName, FacetOrder::Asc);
    let years_fut = fetch_group_facets(pool, user_id, filter, GroupField::Year, FacetOrder::Desc);
    let year_months_fut = fetch_group_facets(
        pool,
        user_id,
        filter,
        GroupField::YearMonth,
        FacetOrder::Desc,
    );

    let (accounts, funds, years, year_months) =
        tokio::try_join!(accounts_fut, funds_fut, years_fut, year_months_fut)?;

    Ok(SearchFacets {
        products: None,
        accounts: Some(accounts),
        securities: None,
        funds: Some(funds),
        years: Some(years),
        year_months: Some(year_months),
    })
}

/// fetch_group_facets で GROUP BY に使える式を限定する集計対象カラム
#[derive(Debug, Clone, Copy)]
enum GroupField {
    Account,
    FundName,
    Year,
    YearMonth,
}

impl GroupField {
    /// SQL に埋め込む式（固定の &'static str のみを返す）
    fn as_sql_expr(self) -> &'static str {
        match self {
            GroupField::Account => "account",
            GroupField::FundName => "fund_name",
            GroupField::Year => "EXTRACT(YEAR FROM trade_date)::integer::text",
            GroupField::YearMonth => "TO_CHAR(trade_date, 'YYYY-MM')",
        }
    }
}

/// group_field の値ごとに件数を集計して FacetOption を返す共通ヘルパー
async fn fetch_group_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &MutualfundFilter,
    group_field: GroupField,
    order: FacetOrder,
) -> Result<Vec<FacetOption>, ApiError> {
    shared::fetch_group_facets(
        pool,
        "mutualfunds",
        group_field.as_sql_expr(),
        order,
        |qb| push_filters(qb, user_id, filter),
    )
    .await
}

/// 投資信託を一括追加（重複はスキップ）
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateMutualfundRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let timer = match BulkTimer::new_with_guard("mutualfund", items) {
        Ok(t) => t,
        Err(empty) => return Ok(empty),
    };

    // 各フィールドを配列に変換
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let trade_dates: Vec<chrono::NaiveDate> = items.iter().map(|i| i.trade_date).collect();
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let fund_names: Vec<&str> = items.iter().map(|i| i.fund_name.as_str()).collect();
    let dividends: Vec<Option<&str>> = items.iter().map(|i| i.dividends.as_deref()).collect();
    let accounts: Vec<&str> = items.iter().map(|i| i.account.as_str()).collect();
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

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
    let result = sqlx::query(
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
    )
    .bind(&user_ids)
    .bind(&trade_dates)
    .bind(&settlement_dates)
    .bind(&fund_names)
    .bind(&dividends)
    .bind(&accounts)
    .bind(&shares)
    .bind(&exchange_rates)
    .bind(&cancellation_unit_prices)
    .bind(&cancellation_amounts)
    .bind(&avg_acquisition_prices)
    .bind(&realized_pls)
    .bind(&taxes)
    .bind(&realized_pls_after_tax)
    .execute(pool)
    .await?;

    timer.finish_from_result(result)
}

/// CSV バイト列から投資信託をパースしてプレビュー情報を返す（DB 書き込みなし）
pub fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
    build_csv_preview(bytes, &MUTUALFUND_CSV_CONFIG, transform_mutualfund_rows)
}

/// CSV バイト列から投資信託をパースして一括挿入
pub async fn upload_csv(
    pool: &PgPool,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<CsvUploadResponse, ApiError> {
    run_csv_upload(
        bytes,
        &MUTUALFUND_CSV_CONFIG,
        transform_mutualfund_rows,
        |items| async move { bulk_create(pool, user_id, &items).await },
    )
    .await
}

fn transform_mutualfund_rows(rows: &[CsvRow]) -> (Vec<CreateMutualfundRequest>, Vec<CsvRowError>) {
    validate_csv_rows(rows, transform_mutualfund_row)
}

fn transform_mutualfund_row(
    row: &CsvRow,
    row_num: usize,
) -> Result<CreateMutualfundRequest, CsvRowError> {
    let trade_date = parse_required_date_row(row, "約定日", row_num)?;
    let settlement_date = parse_required_date_row(row, "受渡日", row_num)?;
    let account = parse_required_string_row(row, "口座", row_num)?;
    let realized_pnl = parse_required_number_row(row, "実現損益［円］", row_num)?;
    let (taxes, realized_pnl_after_tax) = compute_taxes(&account, realized_pnl);
    // 分配金フィールドは空文字を None に変換
    let dividends_raw = get_row_cell(row, "分配金");
    let dividends = if dividends_raw.trim().is_empty() {
        None
    } else {
        Some(dividends_raw.to_string())
    };
    Ok(CreateMutualfundRequest {
        trade_date,
        settlement_date,
        fund_name: parse_required_string_row(row, "ファンド名", row_num)?,
        dividends,
        account,
        shares: parse_required_number_row(row, "数量[口]", row_num)?,
        exchange_rate: parse_required_number_row(row, "為替レート［円］", row_num)?,
        cancellation_unit_price_yen: parse_required_number_row(row, "解約単価［円］", row_num)?,
        cancellation_amount_yen: parse_required_number_row(row, "解約額［円］", row_num)?,
        average_acquisition_price_yen: parse_required_number_row(
            row,
            "平均取得価額［円］",
            row_num,
        )?,
        realized_profit_and_loss: realized_pnl,
        taxes,
        realized_profit_and_loss_after_tax: realized_pnl_after_tax,
    })
}

/// 認証ユーザーの投資信託を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    delete_all_for_user(pool, user_id, DeleteTarget::MutualFunds).await
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    #[test]
    fn test_preview_csv_basic() {
        let csv = concat!("約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n", "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim 米国株式(S&P500)\",\"再投資型\",\"特定\",\"解約\",\"3,721,147\",\"-\",\"19,661\",\"7,316,147\",\"18,005.20\",\"615,849\"");
        let preview = preview_csv(csv.as_bytes()).unwrap();
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.rows.len(),
                preview.errors.is_empty(),
                matches!(preview_csv(b""), Err(ApiError::ValidationError(_)))
            ),
            (1, 1, 1, true, true)
        );
        let csv = concat!("約定日,受渡日,ファンド名,分配金,口座,取引,数量[口],為替レート［円］,解約単価［円］,解約額［円］,平均取得価額［円］,実現損益［円］\n", "\"2022/10/28\",\"2022/11/2\",\"eMAXIS Slim\",\"\",\"特定\",\"解約\",\"1000\",\"1\",\"12000\",\"12000000\",\"10000\",\"615849\"");
        let preview = preview_csv(csv.as_bytes()).unwrap();
        assert_eq!(
            (preview.valid_rows, preview.rows[0]["dividends"].is_null()),
            (1, true)
        );
    }

    #[test]
    fn test_mutualfund_filter_from_params_accepts_valid_date_axis_values() {
        let mut params = MutualfundSearchQueryParams::default();
        params.search.date = Some("2026-01-15".to_string());
        params.search.date_from = Some("2026-01-01".to_string());
        params.search.date_to = Some("2026-12-31".to_string());
        params.search.year = Some(2026);
        params.search.year_month = Some("2026-06".to_string());

        let filter =
            MutualfundFilter::from_params(&params).expect("正しい日付フォーマットは検証を通過する");

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
    }

    #[test]
    fn test_mutualfund_filter_from_params_rejects_invalid_date_axis_values() {
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
            let err = MutualfundFilter::from_params(&params)
                .expect_err(&format!("{field} は不正値で ValidationError になるべき"));
            assert!(
                matches!(err, ApiError::ValidationError(_)),
                "field={field} の失敗が ValidationError ではない"
            );
        }
    }

    #[test]
    fn test_push_filters_combines_q_tokens_as_and_of_or_across_all_columns() {
        let mut params = MutualfundSearchQueryParams::default();
        params.search.q = Some("AA BB".to_string());
        let filter = MutualfundFilter::from_params(&params).expect("q のみなら検証を通過する");

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM mutualfunds");
        push_filters(&mut qb, Uuid::nil(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        // token ごとに 1 つの AND (...) ブロックが生成され、ブロック内は3カラムの OR になる
        assert_eq!(sql.matches(" AND (").count(), 2);
        assert_eq!(sql.matches("account ILIKE").count(), 2);
        assert_eq!(sql.matches("fund_name ILIKE").count(), 2);
        assert_eq!(sql.matches("dividends ILIKE").count(), 2);
    }

    #[test]
    fn test_push_filters_binds_mutualfund_specific_field_filters() {
        let params = MutualfundSearchQueryParams {
            account: Some("特定".to_string()),
            fund_name: Some("eMAXIS Slim".to_string()),
            dividends: Some("再投資型".to_string()),
            ..Default::default()
        };
        let filter =
            MutualfundFilter::from_params(&params).expect("フィルタのみなら検証を通過する");

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM mutualfunds");
        push_filters(&mut qb, Uuid::nil(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        assert!(sql.contains("AND account = "));
        assert!(sql.contains("AND fund_name = "));
        assert!(sql.contains("AND dividends = "));
    }
}
