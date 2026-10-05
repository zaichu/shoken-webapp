use crate::errors::ApiError;
use crate::models::asset_balance::{
    AssetBalance, AssetBalanceSearchQueryParams, AssetBalanceSummary, CreateAssetBalanceRequest,
};
use crate::models::common::{BulkCreateResponse, SearchFacets};
use crate::models::csv_import::CsvRowError;
use crate::services::csv::import::{validate_csv_rows, CsvImport};
#[cfg(test)]
use crate::services::csv::pipeline::parse_csv_with_config;
use crate::services::csv::pipeline::{CsvParserConfig, CsvTable};
use crate::services::csv::util::{
    check_max_chars, parse_number, parse_optional_string, CsvCells, CsvRowView, RowNumber,
};
use crate::services::domain::bulk::{
    ensure_user_row_limit_with, lock_user_domain, user_ids_for_bulk_insert, BulkTimer, RowLimit,
};
use crate::services::domain::facets;
use crate::services::domain::search::Search;
use crate::services::domain::search_filters::{push_search_filters, tokens_from_query};
use crate::services::domain::{Domain, WriteMode};
use rust_decimal::Decimal;
use shared::normalize::normalize_security_name;
use shared::value::{SecurityCode, UserId};
use sqlx::postgres::PgQueryResult;
use sqlx::{PgPool, Postgres, QueryBuilder};

/// 資産残高（保有銘柄）ドメイン
pub struct AssetBalanceDomain;

impl Domain for AssetBalanceDomain {
    const NAME: &'static str = "asset_balance";
    const TABLE: &'static str = "asset_balances";
    const WRITE_MODE: WriteMode = WriteMode::Replace;

    async fn delete_rows(pool: &PgPool, user_id: UserId) -> Result<PgQueryResult, sqlx::Error> {
        sqlx::query!(
            "DELETE FROM asset_balances WHERE user_id = $1",
            user_id.get()
        )
        .execute(pool)
        .await
    }
}

impl CsvImport for AssetBalanceDomain {
    type Row = CreateAssetBalanceRequest;
    const CSV_CONFIG: CsvParserConfig = ASSET_BALANCE_CSV_CONFIG;

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
        transform_asset_balance_rows(table)
    }

    async fn bulk_create(
        pool: &PgPool,
        user_id: UserId,
        items: &[Self::Row],
        limit: RowLimit,
    ) -> Result<BulkCreateResponse, ApiError> {
        bulk_create(pool, user_id, items, limit).await
    }
}

const ASSET_BALANCE_CSV_CONFIG: CsvParserConfig = CsvParserConfig {
    skip_header_rows: 6,
    exclude_row_fn: Some(is_account_summary_row),
};

/// 保有銘柄検索条件を SQL 条件へ変換した中間表現
///
/// asset_balances には snapshot 日付がないため、date 系の条件は扱わない。
#[derive(Debug)]
pub struct AssetBalanceFilter {
    tokens: Vec<String>,
    security_code: Option<String>,
    security_name: Option<String>,
}

impl TryFrom<AssetBalanceSearchQueryParams> for AssetBalanceFilter {
    type Error = ApiError;

    fn try_from(params: AssetBalanceSearchQueryParams) -> Result<Self, ApiError> {
        Ok(Self {
            tokens: tokens_from_query(params.search.q.as_deref()),
            security_code: params.security_code,
            security_name: params.security_name,
        })
    }
}

impl Search for AssetBalanceDomain {
    type Data = AssetBalance;
    type Params = AssetBalanceSearchQueryParams;
    type Filter = AssetBalanceFilter;
    type Summary = AssetBalanceSummary;

    const COLUMNS: &'static str = "id, user_id, security_code, security_name, shares, \
        executing_shares, average_purchase_price, total_purchase_amount, current_price, \
        daily_change, created_at, updated_at";
    const ORDER_BY: &'static str = " ORDER BY security_code ASC, id ASC";

    /// asset_balances には snapshot 日付がないため、date axis は渡さない（`None`）。
    fn push_filters(qb: &mut QueryBuilder<Postgres>, user_id: UserId, filter: &AssetBalanceFilter) {
        push_search_filters(
            qb,
            user_id,
            None,
            &[
                ("security_code", &filter.security_code),
                ("security_name", &filter.security_name),
            ],
            &filter.tokens,
            &["security_code", "security_name"],
        );
    }

    /// 検索条件全体の summary を算出する（ページ内だけでなく検索条件全体の合計）
    async fn fetch_summary(
        pool: &PgPool,
        user_id: UserId,
        filter: &AssetBalanceFilter,
    ) -> Result<AssetBalanceSummary, ApiError> {
        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT COALESCE(SUM(market_value), 0) AS total_market_value, \
             COALESCE(SUM(total_purchase_amount), 0) AS total_purchase_amount, \
             COALESCE(SUM(daily_change), 0) AS total_daily_change \
             FROM asset_balances",
        );
        Self::push_filters(&mut qb, user_id, filter);
        Ok(qb
            .build_query_as::<AssetBalanceSummary>()
            .fetch_one(pool)
            .await?)
    }

    async fn fetch_facets(
        pool: &PgPool,
        user_id: UserId,
        filter: &AssetBalanceFilter,
    ) -> Result<SearchFacets, ApiError> {
        let securities = facets::fetch_security_facets(pool, Self::TABLE, "id", |qb| {
            Self::push_filters(qb, user_id, filter);
        })
        .await?;

        Ok(SearchFacets {
            products: None,
            accounts: None,
            securities: Some(securities),
            funds: None,
            years: None,
            year_months: None,
        })
    }
}

/// 保有銘柄を一括登録（既存データを全削除してから挿入）
pub async fn bulk_create(
    pool: &PgPool,
    user_id: UserId,
    items: &[CreateAssetBalanceRequest],
    limit: RowLimit,
) -> Result<BulkCreateResponse, ApiError> {
    let total = items.len();
    let timer = BulkTimer::new(AssetBalanceDomain::NAME, total);

    let user_ids = user_ids_for_bulk_insert(user_id, total);
    let security_codes: Vec<&str> = items.iter().map(|i| i.security_code.as_str()).collect();
    let security_names: Vec<&str> = items.iter().map(|i| i.security_name.as_str()).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let executing_shares: Vec<Decimal> = items.iter().map(|i| i.executing_shares).collect();
    let average_purchase_prices: Vec<Decimal> =
        items.iter().map(|i| i.average_purchase_price).collect();
    let total_purchase_amounts: Vec<Decimal> =
        items.iter().map(|i| i.total_purchase_amount).collect();
    let current_prices: Vec<Decimal> = items.iter().map(|i| i.current_price).collect();
    let daily_changes: Vec<Decimal> = items.iter().map(|i| i.daily_change).collect();
    let market_values: Vec<Decimal> = items.iter().map(|i| i.market_value).collect();
    let profit_loss_rates: Vec<Decimal> = items.iter().map(|i| i.profit_loss_rate).collect();

    let mut tx = pool.begin().await?;

    lock_user_domain::<AssetBalanceDomain>(&mut tx, user_id).await?;

    ensure_user_row_limit_with::<AssetBalanceDomain, _>(&mut *tx, user_id, total, limit).await?;

    sqlx::query!(
        "DELETE FROM asset_balances WHERE user_id = $1",
        user_id.get()
    )
    .execute(&mut *tx)
    .await?;

    if !items.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO asset_balances (user_id, security_code, security_name, shares, executing_shares,
                                        average_purchase_price, total_purchase_amount, current_price,
                                        daily_change, market_value, profit_loss_rate)
            SELECT * FROM UNNEST(
                $1::uuid[], $2::text[], $3::text[], $4::numeric[], $5::numeric[],
                $6::numeric[], $7::numeric[], $8::numeric[], $9::numeric[], $10::numeric[], $11::numeric[]
            )
            "#,
            &user_ids as &[UserId],
            &security_codes as &[&str],
            &security_names as &[&str],
            &shares,
            &executing_shares,
            &average_purchase_prices,
            &total_purchase_amounts,
            &current_prices,
            &daily_changes,
            &market_values,
            &profit_loss_rates
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(timer.finish(total))
}

fn transform_asset_balance_rows(
    table: &CsvTable,
) -> (Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>) {
    let (items, errors) = validate_csv_rows(table, transform_asset_balance_row);
    (items.into_iter().flatten().collect(), errors)
}

/// 保有銘柄 CSV に混ざる「特定口座合計」などの口座集計行を除外する
///
/// 先頭フィールド（銘柄コード）が空の行かつ「口座合計」を含む行のみ除外する。
/// 銘柄名に「口座合計」を含む銘柄を誤除外しないよう、先頭が空であることを条件とする。
fn is_account_summary_row(row: &CsvRowView<'_>) -> bool {
    row.cell("銘柄コード").trim().is_empty() && row.values().any(|value| value.contains("口座合計"))
}

/// 保有銘柄 CSV の1行をパースして CreateAssetBalanceRequest に変換
///
/// 数値パース失敗は CsvRowError として返す。
/// ただし以下の列は "-" / 空欄が仕様上ありうるため 0.0 フォールバックを維持する:
///   - 執行中: 執行中の注文がなければ "-" または空欄
///   - 現在値（前日比）: 変動なし時は 0 または "-"
///   - 評価損益（%）: NISA 等で表示されない場合に "-"
fn transform_asset_balance_row(
    row: &CsvRowView<'_>,
    row_num: RowNumber,
) -> Result<Option<CreateAssetBalanceRequest>, CsvRowError> {
    let num = |col: &str| {
        let raw = parse_optional_string(row, col);
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == "-" {
            return Err(CsvRowError {
                row: row_num.get(),
                message: format!("必須列 '{col}' が空または値なし"),
            });
        }
        parse_number(trimmed).map_err(|e| CsvRowError {
            row: row_num.get(),
            message: format!("{col}: {e}"),
        })
    };

    // 必須の数値を先に検証し、銘柄コードが空でも不正な行はエラーとして報告する
    let shares = num("保有数量［株］")?;
    let average_purchase_price = num("平均取得価額［円］")?;
    let total_purchase_amount = num("取得総額［円］")?;
    let current_price = num("現在値［円］")?;
    let market_value = num("時価評価額［円］")?;

    // 銘柄コードが空の行（「口座合計」以外の集計・罫線行）は取り込み対象外
    let code_raw = parse_optional_string(row, "銘柄コード").replace('"', "");
    if code_raw.trim().is_empty() {
        return Ok(None);
    }
    let security_code = SecurityCode::try_from(code_raw.as_str()).map_err(|e| CsvRowError {
        row: row_num.get(),
        message: format!("銘柄コード: {e}"),
    })?;
    Ok(Some(CreateAssetBalanceRequest {
        security_code,
        security_name: check_max_chars(
            normalize_security_name(&parse_optional_string(row, "銘柄名")),
            "銘柄名",
            200,
            row_num,
        )?,
        shares,
        // 執行中は "-" / 空欄が仕様上ありうるため 0.0 フォールバック
        executing_shares: parse_number(&parse_optional_string(row, "執行中［株］"))
            .unwrap_or(Decimal::ZERO),
        average_purchase_price,
        total_purchase_amount,
        current_price,
        // 前日比は変動なし時に 0 または "-" が仕様上ありうるため 0.0 フォールバック
        daily_change: parse_number(&parse_optional_string(row, "現在値（前日比）［円］"))
            .unwrap_or(Decimal::ZERO),
        market_value,
        // 評価損益は NISA 等で表示されない場合に "-" が仕様上ありうるため 0.0 フォールバック
        profit_loss_rate: parse_number(&parse_optional_string(row, "評価損益［％］"))
            .unwrap_or(Decimal::ZERO),
    }))
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::common::SearchParamsAccessor;
    use crate::services::csv::import::CsvImport;
    use rust_decimal_macros::dec;

    type ExpectedRow<'a> = (
        &'a str,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
        Decimal,
    );

    const HEADER: &str =
        "銘柄コード,銘柄名,保有数量［株］,執行中［株］,平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";
    const ASSET_BALANCE_CSV_HEADER: &str =
        "銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";

    const TEST_ROW_1: &str = "1234,テスト株式会社,100,0,100,0,1500,150000,1600,10,160000,6.67";
    const TEST_ROW_2: &str = "5678,サンプル株式会社,200,0,200,0,1800,360000,1900,15,380000,5.56";
    const INPEX_ROW: &str =
        "\"1605\",\"ＩＮＰＥＸ\",\"200\",\"0\",\"200\",\"0\",\"2,355.00\",\"471,000\",\"3,685.0\",\"65.0\",\"737,000\",\"56.47\"";
    const NINTENDO_ROW: &str =
        "\"7974\",\"任天堂\",\"1,000\",\"0\",\"1,000\",\"0\",\"5,997.60\",\"5,997,600\",\"8,737.0\",\"223.0\",\"8,737,000\",\"45.67\"";
    const ACCOUNT_SUMMARY_ROW: &str =
        ",,,,,,特定口座合計,\"11,245,249\",,,\"14,517,240\",\"29.09\"";

    fn parse_row_csv(row: &str) -> (Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>) {
        let table = parse_csv_with_config(
            format!("{HEADER}\n{row}\n").as_bytes(),
            &CsvParserConfig {
                skip_header_rows: 0,
                exclude_row_fn: None,
            },
        )
        .unwrap();

        transform_asset_balance_rows(&table)
    }

    fn make_asset_balance_csv(rows: &[&str]) -> String {
        format!(
            "■現在の評価額合計［円］,,\"9,474,000\"\n\
             ■評価損益合計,前日比［円］,\"288,000\"\n\
             ,前月比［円］,\"-120,000\"\n\
             ,評価損益［円］,\"2,005,900\"\n\
             ■特定口座\n\n\
             {ASSET_BALANCE_CSV_HEADER}\n{}",
            rows.join("\n")
        )
    }

    fn assert_row_ok(row: &str, expected: ExpectedRow<'_>) {
        let (items, errors) = parse_row_csv(row);

        assert!(errors.is_empty(), "unexpected errors for {row}: {errors:?}");
        assert_eq!(items.len(), 1, "row should produce one item: {row}");

        let item = &items[0];
        assert_eq!(
            (
                item.security_code.as_str(),
                item.shares,
                item.executing_shares,
                item.average_purchase_price,
                item.current_price,
                item.daily_change,
                item.profit_loss_rate,
            ),
            expected
        );
    }

    fn assert_row_error(row: &str, expected_message: &str) {
        let (items, errors) = parse_row_csv(row);

        assert!(items.is_empty(), "invalid row must not be parsed: {row}");
        assert_eq!(errors.len(), 1, "row should produce one error: {row}");
        assert!(errors[0].message.contains(expected_message), "{errors:?}");
    }

    #[test]
    fn test_parse_asset_balance_row() {
        for (row, expected) in [
            (
                "1234,テスト株式会社,100,-,1500,150000,1600,10,160000,6.67",
                (
                    "1234",
                    dec!(100),
                    Decimal::ZERO,
                    dec!(1500),
                    dec!(1600),
                    dec!(10),
                    dec!(6.67),
                ),
            ),
            (
                "5678,ファンド,50,-,2000,100000,2100,-,105000,-",
                (
                    "5678",
                    dec!(50),
                    Decimal::ZERO,
                    dec!(2000),
                    dec!(2100),
                    Decimal::ZERO,
                    Decimal::ZERO,
                ),
            ),
            // 前後の空白は trim してから検証する
            (
                " 1234 ,テスト株式会社,100,-,1500,150000,1600,10,160000,6.67",
                (
                    "1234",
                    dec!(100),
                    Decimal::ZERO,
                    dec!(1500),
                    dec!(1600),
                    dec!(10),
                    dec!(6.67),
                ),
            ),
        ] {
            assert_row_ok(row, expected);
        }

        for (row, expected_message) in [
            ("1234,テスト,N/A,-,1500,150000,1600,0,160000,0", "保有数量"),
            ("1234,テスト,-,-,1500,150000,1600,0,160000,0", "保有数量"),
            ("1234,テスト,100,-,1500,150000,N/A,0,160000,0", "現在値"),
            // 記号入りの銘柄コードは行エラー(newtype 化後の仕様)
            (
                "7203-1,テスト,100,-,1500,150000,1600,10,160000,6.67",
                "銘柄コード",
            ),
        ] {
            assert_row_error(row, expected_message);
        }

        let preview = AssetBalanceDomain::preview_csv(
            make_asset_balance_csv(&[INPEX_ROW, NINTENDO_ROW, ACCOUNT_SUMMARY_ROW]).as_bytes(),
        )
        .unwrap();
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.rows.len(),
                preview.errors.is_empty(),
            ),
            (2, 2, 2, true),
            "unexpected errors: {:?}",
            preview.errors
        );

        assert!(matches!(
            AssetBalanceDomain::preview_csv(b""),
            Err(ApiError::Csv(_))
        ));

        for (rows, expected_codes) in [
            (
                &[TEST_ROW_1, ",,,,,,,,,,,", TEST_ROW_2][..],
                &["1234", "5678"][..],
            ),
            (
                &[INPEX_ROW, NINTENDO_ROW, ACCOUNT_SUMMARY_ROW][..],
                &["1605", "7974"][..],
            ),
        ] {
            let table = parse_csv_with_config(
                make_asset_balance_csv(rows).as_bytes(),
                &ASSET_BALANCE_CSV_CONFIG,
            )
            .unwrap();
            let (items, errors) = transform_asset_balance_rows(&table);

            assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            assert_eq!(
                items
                    .iter()
                    .map(|item| item.security_code.as_str())
                    .collect::<Vec<_>>(),
                expected_codes
            );
        }
    }

    #[test]
    fn test_is_account_summary_row() {
        use crate::services::csv::util::HeaderIndex;

        let index = HeaderIndex::new(["銘柄コード", "銘柄名"]);
        let is_summary = |code: &str, name: &str| {
            let record = csv::StringRecord::from(vec![code, name]);
            is_account_summary_row(&CsvRowView::new(&record, &index))
        };

        assert!(is_summary("", "特定口座合計"));
        // 「口座合計」を含む値があっても銘柄コードが入っていれば集計行ではない
        assert!(!is_summary("9999", "口座合計を含む名称"));
        assert!(!is_summary("", "普通株式"));
        assert!(!is_summary("7203", "トヨタ自動車"));
    }

    #[test]
    fn test_asset_balance_search_query_params_delegate_include_flags() {
        let mut params = AssetBalanceSearchQueryParams::default();
        assert!(!params.should_include_summary());
        assert!(!params.should_include_facets());

        params.search.include_summary = Some(true);
        params.search.include_facets = Some(true);
        assert!(params.should_include_summary());
        assert!(params.should_include_facets());
    }

    #[test]
    fn test_push_filters_combines_q_tokens_and_domain_fields() {
        let mut params = AssetBalanceSearchQueryParams::default();
        params.search.q = Some("AA BB".to_string());
        let filter = AssetBalanceFilter::try_from(params).unwrap();

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM asset_balances");
        AssetBalanceDomain::push_filters(&mut qb, UserId::default(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        // token ごとに 1 つの AND (...) ブロックが生成され、ブロック内は2カラムの OR になる
        assert_eq!(sql.matches(" AND (").count(), 2);
        assert_eq!(sql.matches("security_code ILIKE").count(), 2);
        assert_eq!(sql.matches("security_name ILIKE").count(), 2);

        let params = AssetBalanceSearchQueryParams {
            security_code: Some("1234".to_string()),
            security_name: Some("テスト株式会社".to_string()),
            ..Default::default()
        };
        let filter = AssetBalanceFilter::try_from(params).unwrap();

        let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("SELECT 1 FROM asset_balances");
        AssetBalanceDomain::push_filters(&mut qb, UserId::default(), &filter);
        let sql = qb.sql();
        let sql = sql.as_str();

        assert!(sql.contains("AND security_code = "));
        assert!(sql.contains("AND security_name = "));
    }
}
