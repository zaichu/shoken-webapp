use crate::errors::ApiError;
use crate::models::common::{
    BulkCreateResponse, FacetOption, PaginatedSearchResponse, SearchFacets, SearchParamsAccessor,
};
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::models::domestic_stock::{
    CreateDomesticStockRequest, DomesticStock, DomesticStockSearchQueryParams, DomesticStockSummary,
};
use crate::services::csv_import::{build_csv_preview, run_csv_upload, validate_csv_rows};
use crate::services::csv_pipeline::{CsvParserConfig, CsvRow};
use crate::services::csv_util::{
    compute_taxes, normalize_security_name, parse_required_date_row, parse_required_number_row,
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

const DOMESTIC_STOCK_CSV_CONFIG: CsvParserConfig = CsvParserConfig {
    skip_header_rows: 0,
    exclude_row_fn: None,
    required_columns: &[
        "約定日",
        "受渡日",
        "銘柄コード",
        "銘柄名",
        "口座",
        "数量[株]",
        "売却/決済単価[円]",
        "売却/決済額[円]",
        "平均取得価額[円]",
        "実現損益[円]",
    ],
};

/// 国内株式検索条件を SQL 条件へ変換した中間表現
#[derive(Debug)]
struct DomesticStockFilter {
    date_axis: DateAxisFilter,
    tokens: Vec<String>,
    account: Option<String>,
    security_code: Option<String>,
    security_name: Option<String>,
}

impl DomesticStockFilter {
    fn from_params(params: &DomesticStockSearchQueryParams) -> Result<Self, ApiError> {
        let search = &params.search;
        let date_axis = DateAxisFilter::from_search_params(search)?;
        let tokens = tokens_from_query(search.q.as_deref());

        Ok(Self {
            date_axis,
            tokens,
            account: params.account.clone(),
            security_code: params.security_code.clone(),
            security_name: params.security_name.clone(),
        })
    }
}

/// user_id と検索条件を WHERE 句として QueryBuilder へ積む
fn push_filters(qb: &mut QueryBuilder<Postgres>, user_id: Uuid, filter: &DomesticStockFilter) {
    push_search_filters(
        qb,
        user_id,
        Some(("trade_date", &filter.date_axis)),
        &[
            ("account", &filter.account),
            ("security_code", &filter.security_code),
            ("security_name", &filter.security_name),
        ],
        &filter.tokens,
        &["account", "security_code", "security_name"],
    );
}

/// 認証ユーザーの国内株式取引一覧を検索（ページネーション・summary・facets 対応）
pub async fn search(
    pool: &PgPool,
    user_id: Uuid,
    params: &DomesticStockSearchQueryParams,
) -> Result<PaginatedSearchResponse<DomesticStock, DomesticStockSummary, SearchFacets>, ApiError> {
    info!("[domestic_stock.search] リクエスト受信");
    let filter = DomesticStockFilter::from_params(params)?;

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
        "SELECT COUNT(*) FROM domestic_stocks",
        "SELECT id, user_id, trade_date, settlement_date, security_code, security_name, \
         account, shares, asked_price, proceeds, purchase_price, realized_profit_and_loss, \
         taxes, realized_profit_and_loss_after_tax, created_at, updated_at \
         FROM domestic_stocks",
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

/// 検索条件全体の summary を算出する。
///
/// trade_date ごとに特定口座（account に「特定」を含む）と NISA 等口座の実現損益を分離し、
/// 特定口座合計がプラスの時だけ `floor(合計 * 0.20315)` を日次税額として計算したうえで、
/// 日次結果を合計する（frontend `calculateDailyData` / `calculateDomesticStock` と同一仕様）。
async fn fetch_summary(
    pool: &PgPool,
    user_id: Uuid,
    filter: &DomesticStockFilter,
) -> Result<DomesticStockSummary, ApiError> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
        "WITH filtered AS (SELECT trade_date, account, realized_profit_and_loss FROM domestic_stocks",
    );
    push_filters(&mut qb, user_id, filter);
    qb.push(
        "), daily AS ( \
             SELECT \
                 trade_date, \
                 COALESCE(SUM(realized_profit_and_loss) FILTER (WHERE POSITION('特定' IN account) > 0), 0) AS specific_total, \
                 COALESCE(SUM(realized_profit_and_loss) FILTER (WHERE POSITION('特定' IN account) = 0), 0) AS nisa_total \
             FROM filtered \
             GROUP BY trade_date \
         ), daily_tax AS ( \
             SELECT \
                 specific_total, \
                 nisa_total, \
                 FLOOR(GREATEST(specific_total, 0) * 0.20315) AS tax \
             FROM daily \
         ) \
         SELECT \
             COALESCE(SUM(specific_total + nisa_total), 0) AS total_realized_profit_and_loss, \
             COALESCE(SUM(tax), 0) AS total_taxes, \
             COALESCE(SUM(specific_total - tax + nisa_total), 0) AS total_realized_profit_and_loss_after_tax \
         FROM daily_tax",
    );
    Ok(qb
        .build_query_as::<DomesticStockSummary>()
        .fetch_one(pool)
        .await?)
}

async fn fetch_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &DomesticStockFilter,
) -> Result<SearchFacets, ApiError> {
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

    let (accounts, securities, years, year_months) =
        tokio::try_join!(accounts_fut, securities_fut, years_fut, year_months_fut)?;

    Ok(SearchFacets {
        products: None,
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
    Account,
    Year,
    YearMonth,
}

impl GroupField {
    /// SQL に埋め込む式（固定の &'static str のみを返す）
    fn as_sql_expr(self) -> &'static str {
        match self {
            GroupField::Account => "account",
            GroupField::Year => "EXTRACT(YEAR FROM trade_date)::integer::text",
            GroupField::YearMonth => "TO_CHAR(trade_date, 'YYYY-MM')",
        }
    }
}

/// group_field の値ごとに件数を集計して FacetOption を返す共通ヘルパー
async fn fetch_group_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &DomesticStockFilter,
    group_field: GroupField,
    order: FacetOrder,
) -> Result<Vec<FacetOption>, ApiError> {
    shared::fetch_group_facets(
        pool,
        "domestic_stocks",
        group_field.as_sql_expr(),
        order,
        |qb| push_filters(qb, user_id, filter),
    )
    .await
}

/// 銘柄コードごとに最新の銘柄名を label として件数付きで返す
async fn fetch_security_facets(
    pool: &PgPool,
    user_id: Uuid,
    filter: &DomesticStockFilter,
) -> Result<Vec<FacetOption>, ApiError> {
    shared::fetch_security_facets(pool, "domestic_stocks", "trade_date DESC, id DESC", |qb| {
        push_filters(qb, user_id, filter);
    })
    .await
}

/// 国内株式取引を一括追加（全件挿入）
/// 同一内容の行が複数ある場合も全件保存する。
///
/// 再アップロード防止:
///   content_hash + グローバル occurrence_index（user_id + content_hash 単位の通し番号）により
///   DB に既存の行と同一内容・同一順序の行はスキップされる。
///   対象: 累積CSV（既存分を含む再アップロード）および増分CSV（新規分を追加したアップロード）。
///
/// 注意:
///   純増分CSV（新規分のみ）で既存行と同一ハッシュの行が含まれる場合は
///   batch_index <= existing_count でスキップされる。
///   この挙動を避けるには外部キー（取引ID等）による識別が別途必要。
pub async fn bulk_create(
    pool: &PgPool,
    user_id: Uuid,
    items: &[CreateDomesticStockRequest],
) -> Result<BulkCreateResponse, ApiError> {
    let timer = match BulkTimer::new_with_guard("domestic_stock", items) {
        Ok(t) => t,
        Err(empty) => return Ok(empty),
    };

    // 各フィールドを配列に変換
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let trade_dates: Vec<chrono::NaiveDate> = items.iter().map(|i| i.trade_date).collect();
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let accounts: Vec<&str> = items.iter().map(|i| i.account.as_str()).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let asked_prices: Vec<Decimal> = items.iter().map(|i| i.asked_price).collect();
    let proceeds: Vec<Decimal> = items.iter().map(|i| i.proceeds).collect();
    let purchase_prices: Vec<Decimal> = items.iter().map(|i| i.purchase_price).collect();
    let realized_pls: Vec<Decimal> = items.iter().map(|i| i.realized_profit_and_loss).collect();
    let taxes: Vec<Decimal> = items.iter().map(|i| i.taxes).collect();
    let realized_pls_after_tax: Vec<Decimal> = items
        .iter()
        .map(|i| i.realized_profit_and_loss_after_tax)
        .collect();

    // UNNESTを使ったバルクINSERT（1回のクエリで全件挿入）
    // content_hash は PostgreSQL md5 関数で算出（migration backfill と同一実装）
    // batch_occurrence_index はバッチ内での content_hash 別の連番（WITH ORDINALITY で入力順保持）
    // db_counts は既存 DB の (user_id, content_hash) 単位の件数（他ユーザーに引っ張られない）
    // batch_occurrence_index > existing_count の行のみ挿入し、ON CONFLICT で冪等性を保証
    let result = sqlx::query(
        r#"
        WITH batch_data AS (
            SELECT
                user_id, trade_date, settlement_date, security_code,
                security_name, account, shares, asked_price, proceeds,
                purchase_price, realized_profit_and_loss, taxes,
                realized_profit_and_loss_after_tax,
                ordinality,
                md5(
                    trade_date::text || '|' || settlement_date::text || '|' ||
                    security_code || '|' || security_name || '|' || account || '|' ||
                    shares::text || '|' || asked_price::text || '|' || proceeds::text || '|' ||
                    purchase_price::text || '|' || realized_profit_and_loss::text || '|' ||
                    taxes::text || '|' || realized_profit_and_loss_after_tax::text
                ) AS content_hash
            FROM UNNEST(
                $1::uuid[], $2::date[], $3::date[], $4::text[],
                $5::text[], $6::text[], $7::numeric[], $8::numeric[], $9::numeric[],
                $10::numeric[], $11::numeric[], $12::numeric[], $13::numeric[]
            ) WITH ORDINALITY AS t(user_id, trade_date, settlement_date, security_code,
                   security_name, account, shares, asked_price, proceeds,
                   purchase_price, realized_profit_and_loss, taxes,
                   realized_profit_and_loss_after_tax, ordinality)
        ),
        batch_indexed AS (
            SELECT *,
                ROW_NUMBER() OVER (PARTITION BY content_hash ORDER BY ordinality)::int4
                    AS batch_occurrence_index
            FROM batch_data
        ),
        db_counts AS (
            SELECT content_hash, COUNT(*)::int4 AS existing_count
            FROM domestic_stocks
            WHERE user_id = $14::uuid
            GROUP BY content_hash
        )
        INSERT INTO domestic_stocks (user_id, trade_date, settlement_date, security_code,
                                     security_name, account, shares, asked_price, proceeds,
                                     purchase_price, realized_profit_and_loss, taxes,
                                     realized_profit_and_loss_after_tax,
                                     content_hash, occurrence_index)
        SELECT
            b.user_id, b.trade_date, b.settlement_date, b.security_code,
            b.security_name, b.account, b.shares, b.asked_price, b.proceeds,
            b.purchase_price, b.realized_profit_and_loss, b.taxes,
            b.realized_profit_and_loss_after_tax,
            b.content_hash,
            b.batch_occurrence_index
        FROM batch_indexed b
        LEFT JOIN db_counts d ON b.content_hash = d.content_hash
        WHERE b.batch_occurrence_index > COALESCE(d.existing_count, 0)
        ON CONFLICT (user_id, content_hash, occurrence_index) DO NOTHING
        "#,
    )
    .bind(&user_ids)
    .bind(&trade_dates)
    .bind(&settlement_dates)
    .bind(&security_codes)
    .bind(&security_names)
    .bind(&accounts)
    .bind(&shares)
    .bind(&asked_prices)
    .bind(&proceeds)
    .bind(&purchase_prices)
    .bind(&realized_pls)
    .bind(&taxes)
    .bind(&realized_pls_after_tax)
    .bind(user_id) // $14: スカラーのユーザーID（db_counts WHERE 句用）
    .execute(pool)
    .await?;

    timer.finish_from_result(result)
}

/// CSV バイト列から国内株式取引をパースしてプレビュー情報を返す（DB 書き込みなし）
pub fn preview_csv(bytes: &[u8]) -> Result<CsvPreviewResponse, ApiError> {
    build_csv_preview(
        bytes,
        &DOMESTIC_STOCK_CSV_CONFIG,
        transform_domestic_stock_rows,
    )
}

/// CSV バイト列から国内株式取引をパースして一括挿入
pub async fn upload_csv(
    pool: &PgPool,
    user_id: Uuid,
    bytes: &[u8],
) -> Result<CsvUploadResponse, ApiError> {
    run_csv_upload(
        bytes,
        &DOMESTIC_STOCK_CSV_CONFIG,
        transform_domestic_stock_rows,
        |items| async move { bulk_create(pool, user_id, &items).await },
    )
    .await
}

fn transform_domestic_stock_rows(
    rows: &[CsvRow],
) -> (Vec<CreateDomesticStockRequest>, Vec<CsvRowError>) {
    validate_csv_rows(rows, transform_domestic_stock_row)
}

fn transform_domestic_stock_row(
    row: &CsvRow,
    row_num: usize,
) -> Result<CreateDomesticStockRequest, CsvRowError> {
    let trade_date = parse_required_date_row(row, "約定日", row_num)?;
    let settlement_date = parse_required_date_row(row, "受渡日", row_num)?;
    let account = parse_required_string_row(row, "口座", row_num)?;
    let realized_pnl = parse_required_number_row(row, "実現損益[円]", row_num)?;
    let (taxes, realized_pnl_after_tax) = compute_taxes(&account, realized_pnl);
    Ok(CreateDomesticStockRequest {
        trade_date,
        settlement_date,
        security_code: parse_required_string_row(row, "銘柄コード", row_num)?,
        security_name: normalize_security_name(&parse_required_string_row(row, "銘柄名", row_num)?),
        account,
        shares: parse_required_number_row(row, "数量[株]", row_num)?,
        asked_price: parse_required_number_row(row, "売却/決済単価[円]", row_num)?,
        proceeds: parse_required_number_row(row, "売却/決済額[円]", row_num)?,
        purchase_price: parse_required_number_row(row, "平均取得価額[円]", row_num)?,
        realized_profit_and_loss: realized_pnl,
        taxes,
        realized_profit_and_loss_after_tax: realized_pnl_after_tax,
    })
}

/// 認証ユーザーの国内株式取引を全削除
pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<u64, ApiError> {
    delete_all_for_user(pool, user_id, DeleteTarget::DomesticStocks).await
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;

    const HEADER: &str =
        "約定日,受渡日,銘柄コード,銘柄名,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]";
    const BASIC_ROW: &str =
        "\"2026/02/09\",\"2026/02/12\",\"5020\",\"ＥＮＥＯＳホールディングス\",\"特定\",\"-\",\"売付\",\"100\",\"1,441.0\",\"144,100\",\"1,350.00\",\"9,100\"";
    const NISA_ROW: &str =
        "\"2026/02/09\",\"2026/02/12\",\"9433\",\"K D D I\",\"NISA\",\"-\",\"売付\",\"100\",\"1441.0\",\"144100\",\"1350.00\",\"9100\"";
    const MISSING_NAME_HEADER: &str =
        "約定日,受渡日,銘柄コード,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]";
    const MISSING_NAME_ROW: &str =
        "\"2026/02/09\",\"2026/02/12\",\"5020\",\"特定\",\"-\",\"売付\",\"100\",\"1,441.0\",\"144,100\",\"1,350.00\",\"9,100\"";
    const INVALID_PNL_ROW: &str =
        "\"2026/02/09\",\"2026/02/12\",\"5020\",\"ＥＮＥＯＳ\",\"特定\",\"-\",\"売付\",\"100\",\"1441.0\",\"144100\",\"1350.00\",\"N/A\"";

    fn preview_with_header(header: &str, row: &str) -> CsvPreviewResponse {
        preview_csv(format!("{header}\n{row}").as_bytes()).unwrap()
    }

    fn assert_preview_ok(row: &str) -> CsvPreviewResponse {
        let preview = preview_with_header(HEADER, row);

        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.rows.len(),
                preview.errors.is_empty(),
            ),
            (1, 1, 1, true),
            "unexpected errors: {:?}",
            preview.errors
        );

        preview
    }

    fn assert_preview_error(header: &str, row: &str, expected_message: &str) {
        let preview = preview_with_header(header, row);

        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.len(),
                preview
                    .errors
                    .first()
                    .is_some_and(|error| error.message.contains(expected_message)),
            ),
            (1, 0, 1, true)
        );
    }

    #[test]
    fn test_preview_csv() {
        assert_eq!(
            assert_preview_ok(BASIC_ROW).rows[0]["security_name"],
            "ＥＮＥＯＳホールディングス"
        );

        let preview = assert_preview_ok(NISA_ROW);
        assert_eq!(
            (
                preview.rows[0]["security_name"].as_str(),
                preview.rows[0]["taxes"].as_f64(),
                preview.rows[0]["realized_profit_and_loss_after_tax"].as_f64(),
            ),
            (Some("KDDI"), Some(0.0), Some(9100.0))
        );

        for (header, row, expected_message) in [
            (MISSING_NAME_HEADER, MISSING_NAME_ROW, "銘柄名"),
            (HEADER, INVALID_PNL_ROW, "実現損益[円]"),
        ] {
            assert_preview_error(header, row, expected_message);
        }

        assert!(matches!(
            preview_csv(b""),
            Err(ApiError::ValidationError(_))
        ));
    }

    fn make_test_item() -> CreateDomesticStockRequest {
        CreateDomesticStockRequest {
            trade_date: NaiveDate::from_ymd_opt(2026, 2, 12).unwrap(),
            settlement_date: NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(),
            security_code: "9508".to_string(),
            security_name: "九州電力".to_string(),
            account: "特定".to_string(),
            shares: dec!(100),
            asked_price: dec!(1880),
            proceeds: dec!(188000),
            purchase_price: dec!(1770),
            realized_profit_and_loss: dec!(11000),
            taxes: dec!(2234),
            realized_profit_and_loss_after_tax: dec!(8766),
        }
    }

    #[tokio::test]
    #[ignore = "requires Docker"]
    async fn test_bulk_create_reupload_deduplication() {
        use testcontainers::runners::AsyncRunner;
        use testcontainers_modules::postgres::Postgres;

        let container = Postgres::default().start().await.unwrap();
        let url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            container.get_host().await.unwrap(),
            container.get_host_port_ipv4(5432).await.unwrap()
        );
        let pool = sqlx::PgPool::connect(&url).await.unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(format!("test_google_{user_id}"))
            .bind(format!("test_{user_id}@example.com"))
            .execute(&pool)
            .await
            .unwrap();

        let items = vec![make_test_item(); 5];

        let first = bulk_create(&pool, user_id, &items).await.unwrap();
        assert_eq!((first.inserted, first.skipped), (5, 0));

        let second = bulk_create(&pool, user_id, &items).await.unwrap();
        assert_eq!((second.inserted, second.skipped), (0, 5));
    }

    #[test]
    fn test_domestic_stock_search_query_params_delegate_include_flags() {
        let mut params = DomesticStockSearchQueryParams::default();
        assert!(!params.should_include_summary());
        assert!(!params.should_include_facets());

        params.search.include_summary = Some(true);
        params.search.include_facets = Some(true);
        assert!(params.should_include_summary());
        assert!(params.should_include_facets());
    }

    #[test]
    fn test_domestic_stock_filter_from_params_accepts_valid_date_axis_values() {
        let mut params = DomesticStockSearchQueryParams::default();
        params.search.date = Some("2026-01-15".to_string());
        params.search.date_from = Some("2026-01-01".to_string());
        params.search.date_to = Some("2026-12-31".to_string());
        params.search.year = Some(2026);
        params.search.year_month = Some("2026-06".to_string());

        let filter = DomesticStockFilter::from_params(&params)
            .expect("正しい日付フォーマットは検証を通過する");

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
    fn test_domestic_stock_filter_from_params_rejects_invalid_date_axis_values() {
        type Apply = fn(&mut DomesticStockSearchQueryParams);
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
            let mut params = DomesticStockSearchQueryParams::default();
            apply(&mut params);
            let err = DomesticStockFilter::from_params(&params)
                .expect_err(&format!("{field} は不正値で ValidationError になるべき"));
            assert!(
                matches!(err, ApiError::ValidationError(_)),
                "field={field} の失敗が ValidationError ではない"
            );
        }
    }

    #[test]
    fn test_push_filters_combines_q_tokens_as_and_of_or_across_all_columns() {
        let mut params = DomesticStockSearchQueryParams::default();
        params.search.q = Some("AA BB".to_string());
        let filter = DomesticStockFilter::from_params(&params).expect("q のみなら検証を通過する");

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM domestic_stocks");
        push_filters(&mut qb, Uuid::nil(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        // token ごとに 1 つの AND (...) ブロックが生成され、ブロック内は3カラムの OR になる
        assert_eq!(sql.matches(" AND (").count(), 2);
        assert_eq!(sql.matches("account ILIKE").count(), 2);
        assert_eq!(sql.matches("security_code ILIKE").count(), 2);
        assert_eq!(sql.matches("security_name ILIKE").count(), 2);
    }

    #[test]
    fn test_push_filters_binds_domestic_stock_specific_field_filters() {
        let params = DomesticStockSearchQueryParams {
            account: Some("特定".to_string()),
            security_code: Some("1234".to_string()),
            security_name: Some("テスト株式会社".to_string()),
            ..Default::default()
        };
        let filter =
            DomesticStockFilter::from_params(&params).expect("フィルタのみなら検証を通過する");

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM domestic_stocks");
        push_filters(&mut qb, Uuid::nil(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        assert!(sql.contains("AND account = "));
        assert!(sql.contains("AND security_code = "));
        assert!(sql.contains("AND security_name = "));
    }

    fn make_summary_item(
        trade_date: NaiveDate,
        account: &str,
        security_code: &str,
        realized_profit_and_loss: Decimal,
    ) -> CreateDomesticStockRequest {
        CreateDomesticStockRequest {
            trade_date,
            settlement_date: trade_date,
            security_code: security_code.to_string(),
            security_name: "テスト株式会社".to_string(),
            account: account.to_string(),
            shares: dec!(100),
            asked_price: dec!(1000),
            proceeds: dec!(100000),
            purchase_price: dec!(900),
            realized_profit_and_loss,
            taxes: dec!(0),
            realized_profit_and_loss_after_tax: realized_profit_and_loss,
        }
    }

    #[tokio::test]
    #[ignore = "requires Docker"]
    async fn test_search_summary_matches_frontend_daily_tax_calculation() {
        use testcontainers::runners::AsyncRunner;
        use testcontainers_modules::postgres::Postgres as PgContainer;

        let container = PgContainer::default().start().await.unwrap();
        let url = format!(
            "postgres://postgres:postgres@{}:{}/postgres",
            container.get_host().await.unwrap(),
            container.get_host_port_ipv4(5432).await.unwrap()
        );
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        let user_id = Uuid::new_v4();
        sqlx::query("INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)")
            .bind(user_id)
            .bind(format!("test_google_{user_id}"))
            .bind(format!("test_{user_id}@example.com"))
            .execute(&pool)
            .await
            .unwrap();

        let day1 = NaiveDate::from_ymd_opt(2026, 2, 10).unwrap();
        let day2 = NaiveDate::from_ymd_opt(2026, 2, 11).unwrap();

        // day1: 特定口座 +10000 / +5000（合計 +15000、税額 floor(15000*0.20315)=3047）、NISA +2000（非課税）
        // day2: 特定口座 -3000（マイナスのため非課税）
        let items = vec![
            make_summary_item(day1, "特定", "5020", dec!(10000)),
            make_summary_item(day1, "特定", "5020", dec!(5000)),
            make_summary_item(day1, "NISA", "9433", dec!(2000)),
            make_summary_item(day2, "特定", "5020", dec!(-3000)),
        ];
        bulk_create(&pool, user_id, &items).await.unwrap();

        let mut params = DomesticStockSearchQueryParams::default();
        params.search.include_summary = Some(true);

        let result = search(&pool, user_id, &params).await.unwrap();
        let summary = result
            .summary
            .expect("include_summary=true で summary を返す");

        assert_eq!(summary.total_realized_profit_and_loss, dec!(14000));
        assert_eq!(summary.total_taxes, dec!(3047));
        assert_eq!(
            summary.total_realized_profit_and_loss_after_tax,
            dec!(10953)
        );
    }
}
