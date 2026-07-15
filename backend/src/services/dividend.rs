use crate::errors::ApiError;
use crate::models::common::{
    BulkCreateResponse, FacetOption, PaginatedSearchResponse, SearchFacets, SearchParamsAccessor,
};
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::models::dividend::{
    CreateDividendRequest, Dividend, DividendSearchQueryParams, DividendSummary,
};
use crate::services::bulk_helpers::{
    delete_all_for_user, user_ids_for_bulk_insert, BulkTimer, DeleteTarget,
};
use crate::services::csv_import::{build_csv_preview, run_csv_upload, validate_csv_rows};
use crate::services::csv_pipeline::{CsvParserConfig, CsvRow};
use crate::services::csv_util::{
    normalize_security_name, parse_optional_string_row, parse_required_date_row,
    parse_required_number_row, parse_required_string_row,
};
use crate::services::facets::{self, FacetOrder};
use crate::services::search_filters::{
    push_search_filters, run_paginated_search, tokens_from_query, DateAxisFilter,
};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, QueryBuilder};
use tracing::info;
use uuid::Uuid;

const DIVIDEND_CSV_CONFIG: CsvParserConfig = CsvParserConfig {
    skip_header_rows: 0,
    exclude_row_fn: None,
    required_columns: &[
        "入金日",
        "商品",
        "口座",
        "銘柄コード",
        "銘柄",
        "単価[円/現地通貨]",
        "数量[株/口]",
        "配当・分配金合計（税引前）[円/現地通貨]",
        "税額合計[円/現地通貨]",
        "受取金額[円/現地通貨]",
    ],
};

/// 配当金検索条件を SQL 条件へ変換した中間表現
#[derive(Debug)]
struct DividendFilter {
    date_axis: DateAxisFilter,
    tokens: Vec<String>,
    product: Option<String>,
    account: Option<String>,
    security_code: Option<String>,
    security_name: Option<String>,
}

impl DividendFilter {
    fn from_params(params: &DividendSearchQueryParams) -> Result<Self, ApiError> {
        let search = &params.search;
        let date_axis = DateAxisFilter::from_search_params(search)?;
        let tokens = tokens_from_query(search.q.as_deref());

        Ok(Self {
            date_axis,
            tokens,
            product: params.product.clone(),
            account: params.account.clone(),
            security_code: params.security_code.clone(),
            security_name: params.security_name.clone(),
        })
    }
}

/// user_id と検索条件を WHERE 句として QueryBuilder へ積む
fn push_filters(qb: &mut QueryBuilder<Postgres>, user_id: Uuid, filter: &DividendFilter) {
    push_search_filters(
        qb,
        user_id,
        Some(("settlement_date", &filter.date_axis)),
        &[
            ("product", &filter.product),
            ("account", &filter.account),
            ("security_code", &filter.security_code),
            ("security_name", &filter.security_name),
        ],
        &filter.tokens,
        &["product", "account", "security_code", "security_name"],
    );
}

/// 認証ユーザーの配当金一覧を検索（ページネーション・summary・facets 対応）
pub async fn search(
    pool: &PgPool,
    user_id: Uuid,
    params: &DividendSearchQueryParams,
) -> Result<PaginatedSearchResponse<Dividend, DividendSummary, SearchFacets>, ApiError> {
    info!("[dividend.search] リクエスト受信");
    let filter = DividendFilter::from_params(params)?;

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
        "SELECT COUNT(*) FROM dividends",
        "SELECT id, user_id, settlement_date, product, account, security_code, security_name, \
         unit_price, shares, dividends_before_tax, taxes, net_amount_received, created_at, updated_at \
         FROM dividends",
        " ORDER BY settlement_date DESC, id DESC",
        |qb| push_filters(qb, user_id, &filter),
        params.page(),
        params.per_page(),
        params.offset(),
        summary_fut,
        facets_fut,
    )
    .await
}

async fn fetch_summary(
    pool: &PgPool,
    user_id: Uuid,
    filter: &DividendFilter,
) -> Result<DividendSummary, ApiError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "SELECT COALESCE(SUM(dividends_before_tax), 0) AS total_dividends_before_tax, \
         COALESCE(SUM(taxes), 0) AS total_taxes, \
         COALESCE(SUM(net_amount_received), 0) AS total_net_amount_received \
         FROM dividends",
    );
    push_filters(&mut qb, user_id, filter);
    Ok(qb
        .build_query_as::<DividendSummary>()
        .fetch_one(pool)
        .await?)
}

async fn fetch_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &DividendFilter,
) -> Result<SearchFacets, ApiError> {
    let products_fut =
        fetch_group_facets(pool, user_id, filter, GroupField::Product, FacetOrder::Asc);
    let accounts_fut =
        fetch_group_facets(pool, user_id, filter, GroupField::Account, FacetOrder::Asc);
    let securities_fut = fetch_security_facets(pool, user_id, filter);
    let years_fut = fetch_group_facets(pool, user_id, filter, GroupField::Year, FacetOrder::Desc);
    let year_months_fut = fetch_group_facets(
        pool,
        user_id,
        filter,
        GroupField::YearMonth,
        FacetOrder::Desc,
    );

    let (products, accounts, securities, years, year_months) = tokio::try_join!(
        products_fut,
        accounts_fut,
        securities_fut,
        years_fut,
        year_months_fut
    )?;

    Ok(SearchFacets {
        products: Some(products),
        accounts: Some(accounts),
        securities: Some(securities),
        funds: None,
        years: Some(years),
        year_months: Some(year_months),
    })
}

/// fetch_group_facets で GROUP BY に使える式を限定する集計対象カラム
#[derive(Debug, Clone, Copy)]
enum GroupField {
    Product,
    Account,
    Year,
    YearMonth,
}

impl GroupField {
    /// SQL に埋め込む式（固定の &'static str のみを返す）
    fn as_sql_expr(self) -> &'static str {
        match self {
            GroupField::Product => "product",
            GroupField::Account => "account",
            GroupField::Year => "EXTRACT(YEAR FROM settlement_date)::integer::text",
            GroupField::YearMonth => "TO_CHAR(settlement_date, 'YYYY-MM')",
        }
    }
}

/// group_field の値ごとに件数を集計して FacetOption を返す共通ヘルパー
async fn fetch_group_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &DividendFilter,
    group_field: GroupField,
    order: FacetOrder,
) -> Result<Vec<FacetOption>, ApiError> {
    facets::fetch_group_facets(pool, "dividends", group_field.as_sql_expr(), order, |qb| {
        push_filters(qb, user_id, filter)
    })
    .await
}

/// 銘柄コードごとに最新の銘柄名を label として件数付きで返す
async fn fetch_security_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &DividendFilter,
) -> Result<Vec<FacetOption>, ApiError> {
    facets::fetch_security_facets(pool, "dividends", "settlement_date DESC, id DESC", |qb| {
        push_filters(qb, user_id, filter);
    })
    .await
}

/// 配当金を一括追加（重複はスキップ）
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateDividendRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let timer = match BulkTimer::new_with_guard("dividend", items) {
        Ok(t) => t,
        Err(empty) => return Ok(empty),
    };

    // 各フィールドを配列に変換
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let products: Vec<&str> = items.iter().map(|i| i.product.as_str()).collect();
    let accounts: Vec<&str> = items.iter().map(|i| i.account.as_str()).collect();
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let unit_prices: Vec<Decimal> = items.iter().map(|i| i.unit_price).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let dividends_before_taxes: Vec<Decimal> =
        items.iter().map(|i| i.dividends_before_tax).collect();
    let taxes: Vec<Decimal> = items.iter().map(|i| i.taxes).collect();
    let net_amounts: Vec<Decimal> = items.iter().map(|i| i.net_amount_received).collect();

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
    let result = sqlx::query(
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
    )
    .bind(&user_ids)
    .bind(&settlement_dates)
    .bind(&products)
    .bind(&accounts)
    .bind(&security_codes)
    .bind(&security_names)
    .bind(&unit_prices)
    .bind(&shares)
    .bind(&dividends_before_taxes)
    .bind(&taxes)
    .bind(&net_amounts)
    .execute(pool)
    .await?;

    timer.finish_from_result(result)
}

/// CSV バイト列から配当金をパースしてプレビュー情報を返す（DB 書き込みなし）
pub fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
    build_csv_preview(bytes, &DIVIDEND_CSV_CONFIG, transform_dividend_rows)
}

/// CSV バイト列から配当金をパースして一括挿入
pub async fn upload_csv(
    pool: &PgPool,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<CsvUploadResponse, ApiError> {
    run_csv_upload(
        bytes,
        &DIVIDEND_CSV_CONFIG,
        transform_dividend_rows,
        |items| async move { bulk_create(pool, user_id, &items).await },
    )
    .await
}

fn transform_dividend_rows(rows: &[CsvRow]) -> (Vec<CreateDividendRequest>, Vec<CsvRowError>) {
    validate_csv_rows(rows, transform_dividend_row)
}

fn transform_dividend_row(
    row: &CsvRow,
    row_num: usize,
) -> Result<CreateDividendRequest, CsvRowError> {
    Ok(CreateDividendRequest {
        settlement_date: parse_required_date_row(row, "入金日", row_num)?,
        product: parse_required_string_row(row, "商品", row_num)?,
        account: parse_required_string_row(row, "口座", row_num)?,
        security_code: parse_optional_string_row(row, "銘柄コード"),
        security_name: normalize_security_name(&parse_required_string_row(row, "銘柄", row_num)?),
        unit_price: parse_required_number_row(row, "単価[円/現地通貨]", row_num)?,
        shares: parse_required_number_row(row, "数量[株/口]", row_num)?,
        dividends_before_tax: parse_required_number_row(
            row,
            "配当・分配金合計（税引前）[円/現地通貨]",
            row_num,
        )?,
        taxes: parse_required_number_row(row, "税額合計[円/現地通貨]", row_num)?,
        net_amount_received: parse_required_number_row(row, "受取金額[円/現地通貨]", row_num)?,
    })
}

/// 認証ユーザーの配当金を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    delete_all_for_user(pool, user_id, DeleteTarget::Dividends).await
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    const HEADER: &str =
        "入金日,商品,口座,銘柄コード,銘柄,受取通貨,単価[円/現地通貨],数量[株/口],配当・分配金合計（税引前）[円/現地通貨],税額合計[円/現地通貨],受取金額[円/現地通貨]";

    fn preview_from_lines(lines: &[&str]) -> CsvPreviewResponse {
        preview_csv(lines.join("\n").as_bytes()).unwrap()
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
            preview_csv(b""),
            Err(ApiError::ValidationError(_))
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

    #[test]
    fn test_dividend_search_query_params_delegate_include_flags() {
        let mut params = DividendSearchQueryParams::default();
        assert!(!params.should_include_summary());
        assert!(!params.should_include_facets());

        params.search.include_summary = Some(true);
        params.search.include_facets = Some(true);
        assert!(params.should_include_summary());
        assert!(params.should_include_facets());
    }

    #[test]
    fn test_dividend_filter_from_params_accepts_valid_date_axis_values() {
        let mut params = DividendSearchQueryParams::default();
        params.search.date = Some("2026-01-15".to_string());
        params.search.date_from = Some("2026-01-01".to_string());
        params.search.date_to = Some("2026-12-31".to_string());
        params.search.year = Some(2026);
        params.search.year_month = Some("2026-06".to_string());

        let filter =
            DividendFilter::from_params(&params).expect("正しい日付フォーマットは検証を通過する");

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
    fn test_dividend_filter_from_params_rejects_invalid_date_axis_values() {
        type Apply = fn(&mut DividendSearchQueryParams);
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
            let mut params = DividendSearchQueryParams::default();
            apply(&mut params);
            let err = DividendFilter::from_params(&params)
                .expect_err(&format!("{field} は不正値で ValidationError になるべき"));
            assert!(
                matches!(err, ApiError::ValidationError(_)),
                "field={field} の失敗が ValidationError ではない"
            );
        }
    }

    #[test]
    fn test_push_filters_combines_q_tokens_as_and_of_or_across_all_columns() {
        let mut params = DividendSearchQueryParams::default();
        params.search.q = Some("AA BB".to_string());
        let filter = DividendFilter::from_params(&params).expect("q のみなら検証を通過する");

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM dividends");
        push_filters(&mut qb, Uuid::nil(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        // token ごとに 1 つの AND (...) ブロックが生成され、ブロック内は4カラムの OR になる
        assert_eq!(sql.matches(" AND (").count(), 2);
        assert_eq!(sql.matches("product ILIKE").count(), 2);
        assert_eq!(sql.matches("account ILIKE").count(), 2);
        assert_eq!(sql.matches("security_code ILIKE").count(), 2);
        assert_eq!(sql.matches("security_name ILIKE").count(), 2);
    }

    #[test]
    fn test_push_filters_binds_dividend_specific_field_filters() {
        let params = DividendSearchQueryParams {
            product: Some("国内株式".to_string()),
            account: Some("特定".to_string()),
            security_code: Some("1234".to_string()),
            security_name: Some("テスト株式会社".to_string()),
            ..Default::default()
        };
        let filter = DividendFilter::from_params(&params).expect("フィルタのみなら検証を通過する");

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM dividends");
        push_filters(&mut qb, Uuid::nil(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        assert!(sql.contains("AND product = "));
        assert!(sql.contains("AND account = "));
        assert!(sql.contains("AND security_code = "));
        assert!(sql.contains("AND security_name = "));
    }
}
