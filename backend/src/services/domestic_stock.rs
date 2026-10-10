use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::CsvRowError;
use crate::models::domestic_stock::{
    CreateDomesticStockRequest, DomesticStock, DomesticStockSearchQueryParams, DomesticStockSummary,
};
use crate::services::csv::import::{CsvImport, validate_csv_rows};
use crate::services::csv::pipeline::{CsvParserConfig, CsvTable};
use crate::services::csv::util::{
    CsvRowView, RowNumber, check_max_chars, parse_required_account, parse_required_date,
    parse_required_number, parse_required_security_code, parse_required_string,
};
use crate::services::domain::bulk::{RowLimit, bulk_insert, user_ids_for_bulk_insert};
use crate::services::domain::facets::{FacetOrder, FacetSlot, FacetSpec, GroupField};
use crate::services::domain::search::Search;
use crate::services::domain::search_filters::{FilterField, NO_FIELD, SearchFilter};
use crate::services::domain::{Domain, WriteMode};
use rust_decimal::Decimal;
use shared::normalize::normalize_security_name;
use shared::tax::{SPECIFIC_ACCOUNT_KEYWORD, TAX_RATE, compute_taxes};
use shared::value::UserId;

/// 国内株式ドメイン
pub struct DomesticStockDomain;

impl Domain for DomesticStockDomain {
    const NAME: &'static str = "domestic_stock";
    const TABLE: &'static str = "domestic_stocks";
    const WRITE_MODE: WriteMode = WriteMode::Append;
}

impl CsvImport for DomesticStockDomain {
    type Row = CreateDomesticStockRequest;
    const CSV_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 0,
        exclude_row_fn: None,
    };

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
        validate_csv_rows(table, transform_domestic_stock_row)
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

impl Search for DomesticStockDomain {
    type Data = DomesticStock;
    type Params = DomesticStockSearchQueryParams;
    type Summary = DomesticStockSummary;

    const COLUMNS: &'static str = "id, user_id, trade_date, settlement_date, security_code, \
        security_name, account, shares, asked_price, proceeds, purchase_price, \
        realized_profit_and_loss, taxes, realized_profit_and_loss_after_tax, created_at, \
        updated_at";
    const ORDER_BY: &'static str = " ORDER BY trade_date DESC, id DESC";
    const DATE_COLUMN: Option<&'static str> = Some("trade_date");
    const FILTER_FIELDS: &'static [FilterField] = &[
        FilterField::Account,
        FilterField::SecurityCode,
        FilterField::SecurityName,
    ];
    const FACETS: &'static [FacetSpec] = &[
        FacetSpec::group(FacetSlot::Accounts, GroupField::Account, FacetOrder::Asc),
        FacetSpec::security(FacetSlot::Securities, "trade_date DESC, id DESC"),
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

    fn filter_value(
        params: &DomesticStockSearchQueryParams,
        field: FilterField,
    ) -> &Option<String> {
        match field {
            FilterField::Account => &params.account,
            FilterField::SecurityCode => &params.security_code,
            FilterField::SecurityName => &params.security_name,
            _ => &NO_FIELD,
        }
    }

    /// 検索条件全体の summary を算出する。
    ///
    /// trade_date ごとに特定口座（account に「特定」を含む）と NISA 等口座の実現損益を分離し、
    /// 特定口座合計がプラスの時だけ `floor(合計 * 税率)` を日次税額として計算したうえで、
    /// 日次結果を合計する。キーワードと税率は shared::tax の正準定数を使う。
    async fn fetch_summary(
        pool: &Db,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> Result<DomesticStockSummary, ApiError> {
        let mut qb = Self::filtered_query(
            "WITH filtered AS (SELECT trade_date, account, realized_profit_and_loss FROM domestic_stocks",
            user_id,
            filter,
        );
        qb.push("), daily AS (SELECT trade_date, COALESCE(SUM(realized_profit_and_loss) FILTER (WHERE POSITION(")
            .push_bind(SPECIFIC_ACCOUNT_KEYWORD)
            .push(" IN account) > 0), 0) AS specific_total, COALESCE(SUM(realized_profit_and_loss) FILTER (WHERE POSITION(")
            .push_bind(SPECIFIC_ACCOUNT_KEYWORD)
            .push(
                " IN account) = 0), 0) AS nisa_total FROM filtered GROUP BY trade_date), daily_tax AS (SELECT specific_total, nisa_total, FLOOR(GREATEST(specific_total, 0) * ",
            )
            .push_bind(TAX_RATE)
            .push(
                ") AS tax FROM daily) \
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
    pool: &Db,
    user_id: UserId,
    items: &[CreateDomesticStockRequest],
    limit: RowLimit,
) -> Result<BulkCreateResponse, ApiError> {
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let trade_dates: Vec<chrono::NaiveDate> = items.iter().map(|i| i.trade_date).collect();
    let settlement_dates: Vec<chrono::NaiveDate> =
        items.iter().map(|i| i.settlement_date).collect();
    let security_codes: Vec<String> = items
        .iter()
        .map(|i| i.security_code.as_str().to_owned())
        .collect();
    let security_names: Vec<String> = items.iter().map(|i| i.security_name.clone()).collect();
    let accounts: Vec<String> = items
        .iter()
        .map(|i| i.account.as_str().to_owned())
        .collect();
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

    // content_hash は PostgreSQL md5 関数で算出（migration backfill と同一実装）
    // batch_occurrence_index はバッチ内での content_hash 別の連番（WITH ORDINALITY で入力順保持）
    // db_counts は既存 DB の (user_id, content_hash) 単位の件数（他ユーザーに引っ張られない）
    // batch_occurrence_index > existing_count の行のみ挿入し、ON CONFLICT で冪等性を保証
    bulk_insert::<DomesticStockDomain, _>(
        pool,
        user_id,
        items,
        limit,
        crate::db::query(
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
            vec![
                Bind::from(user_ids),
                Bind::from(trade_dates),
                Bind::from(settlement_dates),
                Bind::from(security_codes),
                Bind::from(security_names),
                Bind::from(accounts),
                Bind::decimal_vec(shares),
                Bind::decimal_vec(asked_prices),
                Bind::decimal_vec(proceeds),
                Bind::decimal_vec(purchase_prices),
                Bind::decimal_vec(realized_pls),
                Bind::decimal_vec(taxes),
                Bind::decimal_vec(realized_pls_after_tax),
                // $14: スカラーのユーザーID（db_counts WHERE 句用）
                Bind::from(user_id),
            ],
        ),
    )
    .await
}

fn transform_domestic_stock_row(
    row: &CsvRowView<'_>,
    row_num: RowNumber,
) -> Result<CreateDomesticStockRequest, CsvRowError> {
    let trade_date = parse_required_date(row, "約定日", row_num)?;
    let settlement_date = parse_required_date(row, "受渡日", row_num)?;
    let account = parse_required_account(row, "口座", row_num)?;
    let realized_pnl = parse_required_number(row, "実現損益[円]", row_num)?;
    let tax = compute_taxes(&account, realized_pnl);
    Ok(CreateDomesticStockRequest {
        trade_date,
        settlement_date,
        security_code: parse_required_security_code(row, "銘柄コード", row_num)?,
        security_name: check_max_chars(
            normalize_security_name(&parse_required_string(row, "銘柄名", row_num)?),
            "銘柄名",
            200,
            row_num,
        )?,
        account,
        shares: parse_required_number(row, "数量[株]", row_num)?,
        asked_price: parse_required_number(row, "売却/決済単価[円]", row_num)?,
        proceeds: parse_required_number(row, "売却/決済額[円]", row_num)?,
        purchase_price: parse_required_number(row, "平均取得価額[円]", row_num)?,
        realized_profit_and_loss: realized_pnl,
        taxes: tax.taxes,
        realized_profit_and_loss_after_tax: tax.realized_profit_and_loss_after_tax,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::csv_import::CsvPreviewResponse;
    use crate::services::csv::import::CsvImport;
    use crate::services::domain::search::search;
    use chrono::NaiveDate;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    const HEADER: &str = "約定日,受渡日,銘柄コード,銘柄名,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]";
    const BASIC_ROW: &str = "\"2026/02/09\",\"2026/02/12\",\"5020\",\"ＥＮＥＯＳホールディングス\",\"特定\",\"-\",\"売付\",\"100\",\"1,441.0\",\"144,100\",\"1,350.00\",\"9,100\"";
    const NISA_ROW: &str = "\"2026/02/09\",\"2026/02/12\",\"9433\",\"K D D I\",\"NISA\",\"-\",\"売付\",\"100\",\"1441.0\",\"144100\",\"1350.00\",\"9100\"";
    const MISSING_NAME_HEADER: &str = "約定日,受渡日,銘柄コード,口座,信用区分,取引,数量[株],売却/決済単価[円],売却/決済額[円],平均取得価額[円],実現損益[円]";
    const MISSING_NAME_ROW: &str = "\"2026/02/09\",\"2026/02/12\",\"5020\",\"特定\",\"-\",\"売付\",\"100\",\"1,441.0\",\"144,100\",\"1,350.00\",\"9,100\"";
    const INVALID_PNL_ROW: &str = "\"2026/02/09\",\"2026/02/12\",\"5020\",\"ＥＮＥＯＳ\",\"特定\",\"-\",\"売付\",\"100\",\"1441.0\",\"144100\",\"1350.00\",\"N/A\"";

    fn preview_with_header(header: &str, row: &str) -> CsvPreviewResponse {
        DomesticStockDomain::preview_csv(format!("{header}\n{row}").as_bytes()).unwrap()
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
            DomesticStockDomain::preview_csv(b""),
            Err(ApiError::Csv(_))
        ));
    }

    #[test]
    fn test_preview_csv_security_code_whitespace_and_symbols() {
        // 前後の空白は trim してから検証する
        let preview = assert_preview_ok(&BASIC_ROW.replace("\"5020\"", "\" 7203 \""));
        assert_eq!(preview.rows[0]["security_code"], "7203");

        // 記号入りの銘柄コードは行エラー(newtype 化後の仕様)
        assert_preview_error(
            HEADER,
            &BASIC_ROW.replace("\"5020\"", "\"7203-1\""),
            "銘柄コード",
        );
    }

    fn make_test_item() -> CreateDomesticStockRequest {
        CreateDomesticStockRequest {
            trade_date: NaiveDate::from_ymd_opt(2026, 2, 12).unwrap(),
            settlement_date: NaiveDate::from_ymd_opt(2026, 2, 16).unwrap(),
            security_code: "9508".parse().unwrap(),
            security_name: "九州電力".to_string(),
            account: "特定".parse().unwrap(),
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
        let (pool, _node) = crate::test_db::start_test_pool().await;

        let user_id = UserId::from(Uuid::new_v4());
        crate::db::query(
            "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
            vec![
                Bind::from(user_id),
                Bind::from(format!("test_google_{user_id}")),
                Bind::from(format!("test_{user_id}@example.com")),
            ],
        )
        .execute(&pool)
        .await
        .unwrap();

        let items = vec![make_test_item(); 5];

        let first = bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
            .await
            .unwrap();
        assert_eq!((first.inserted, first.skipped), (5, 0));

        let second = bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
            .await
            .unwrap();
        assert_eq!((second.inserted, second.skipped), (0, 5));
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
            security_code: security_code.parse().unwrap(),
            security_name: "テスト株式会社".to_string(),
            account: account.parse().unwrap(),
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
        let (pool, _node) = crate::test_db::start_test_pool().await;

        let user_id = UserId::from(Uuid::new_v4());
        crate::db::query(
            "INSERT INTO users (id, google_id, email) VALUES ($1, $2, $3)",
            vec![
                Bind::from(user_id),
                Bind::from(format!("test_google_{user_id}")),
                Bind::from(format!("test_{user_id}@example.com")),
            ],
        )
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
        bulk_create(&pool, user_id, &items, RowLimit::new(i64::MAX))
            .await
            .unwrap();

        let mut params = DomesticStockSearchQueryParams::default();
        params.search.include_summary = Some(true);

        let result = search::<DomesticStockDomain>(&pool, user_id, params)
            .await
            .unwrap();
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
