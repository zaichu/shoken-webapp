use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::CsvRowError;
use crate::models::fund_balance::{
    CreateFundBalanceRequest, FundBalance, FundBalanceSearchQueryParams, FundBalanceSummary,
};
use crate::services::csv::import::{CsvImport, validate_csv_rows};
#[cfg(test)]
use crate::services::csv::pipeline::parse_csv_with_config;
use crate::services::csv::pipeline::{CsvParserConfig, CsvTable};
use crate::services::csv::util::{
    CsvRowView, RowNumber, check_max_chars, parse_number, parse_optional_string,
};
use crate::services::domain::bulk::{RowLimit, bulk_replace, user_ids_for_bulk_insert};
use crate::services::domain::facets::{FacetOrder, FacetSlot, FacetSpec, GroupField};
use crate::services::domain::search::Search;
use crate::services::domain::search_filters::{FilterField, NO_FIELD, SearchFilter};
use crate::services::domain::{Domain, WriteMode};
use rust_decimal::Decimal;
use shared::normalize::normalize_security_name;
use shared::value::{Account, UserId};

/// 投資信託残高（保有ファンド）ドメイン
pub struct FundBalanceDomain;

impl Domain for FundBalanceDomain {
    const NAME: &'static str = "fund_balance";
    const TABLE: &'static str = "fund_balances";
    const WRITE_MODE: WriteMode = WriteMode::Replace;
}

impl CsvImport for FundBalanceDomain {
    type Row = CreateFundBalanceRequest;
    const CSV_CONFIG: CsvParserConfig = CsvParserConfig {
        // 見出し行「投資信託種別,...」を表紙行と認識して前後の説明行を読み飛ばす
        skip_header_rows: 0,
        exclude_row_fn: Some(is_fund_summary_row),
        header_marker: Some("投資信託種別"),
    };

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
        let (items, errors) = validate_csv_rows(table, transform_fund_balance_row);
        (items.into_iter().flatten().collect(), errors)
    }

    async fn bulk_create(
        pool: &Db,
        user_id: UserId,
        items: &[Self::Row],
        limit: RowLimit,
    ) -> Result<BulkCreateResponse, ApiError> {
        bulk_create(pool, user_id, items, limit).await
    }

    /// 全置換のため、エラー行や有効行0件のCSVは保存前に拒否する
    /// （有効行だけで既存の残高を置き換えてしまわないように）
    fn ensure_importable(items: &[Self::Row], errors: &[CsvRowError]) -> Result<(), ApiError> {
        if let Some(first) = errors.first() {
            return Err(ApiError::Validation(format!(
                "{}行目を含む{}件のエラーがあるため投資信託残高の取り込みを中止しました（既存のデータは変更されていません）",
                first.row,
                errors.len()
            )));
        }
        if items.is_empty() {
            return Err(ApiError::Validation(
                "取り込める行がないため投資信託残高の取り込みを中止しました（既存のデータは変更されていません）"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

// 評価額・評価損益は CSV の時価評価額・評価損益をそのまま合計する。
// 前日比合計は基準価額(1万口あたり)の前日比を保有口数で重み付けして円換算する。
// 前日比を持たない行は集計から外さず 0 として足し込む（行ごとの欠損で合計が過小にならないよう）
const SUMMARY_SQL: &str = "SELECT COALESCE(SUM(market_value), 0) AS total_market_value, \
     COALESCE(SUM(total_purchase_amount), 0) AS total_purchase_amount, \
     COALESCE(SUM(shares * COALESCE(base_price_day_change, 0) / 10000), 0) AS total_daily_change \
     FROM fund_balances";

impl Search for FundBalanceDomain {
    type Data = FundBalance;
    type Params = FundBalanceSearchQueryParams;
    type Summary = FundBalanceSummary;

    const COLUMNS: &'static str = "id, user_id, fund_name, account, dividends_course, shares, \
        average_purchase_price, total_purchase_amount, base_price, base_price_day_change, \
        base_price_month_change, market_value, profit_loss, profit_loss_rate, total_return, \
        price_as_of, created_at, updated_at";
    const ORDER_BY: &'static str = " ORDER BY fund_name ASC, account ASC, id ASC";
    const DATE_COLUMN: Option<&'static str> = None;
    const FILTER_FIELDS: &'static [FilterField] = &[FilterField::FundName, FilterField::Account];
    const FACETS: &'static [FacetSpec] = &[
        FacetSpec::group(FacetSlot::Accounts, GroupField::Account, FacetOrder::Asc),
        FacetSpec::group(FacetSlot::Funds, GroupField::FundName, FacetOrder::Asc),
    ];

    fn filter_value(params: &FundBalanceSearchQueryParams, field: FilterField) -> &Option<String> {
        match field {
            FilterField::FundName => &params.fund_name,
            FilterField::Account => &params.account,
            _ => &NO_FIELD,
        }
    }

    /// 検索条件全体の summary を算出する（ページ内だけでなく検索条件全体の合計）
    async fn fetch_summary(
        pool: &Db,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> Result<FundBalanceSummary, ApiError> {
        Ok(Self::filtered_query(SUMMARY_SQL, user_id, filter)
            .build_query_as::<FundBalanceSummary>()
            .fetch_one(pool)
            .await?)
    }
}

/// 投資信託残高を一括登録（既存データを全削除してから挿入）
pub async fn bulk_create(
    pool: &Db,
    user_id: UserId,
    items: &[CreateFundBalanceRequest],
    limit: RowLimit,
) -> Result<BulkCreateResponse, ApiError> {
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let fund_names: Vec<String> = items.iter().map(|i| i.fund_name.clone()).collect();
    let accounts: Vec<String> = items
        .iter()
        .map(|i| i.account.as_str().to_owned())
        .collect();
    let dividends_courses: Vec<Option<String>> =
        items.iter().map(|i| i.dividends_course.clone()).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let average_purchase_prices: Vec<Decimal> =
        items.iter().map(|i| i.average_purchase_price).collect();
    let total_purchase_amounts: Vec<Decimal> =
        items.iter().map(|i| i.total_purchase_amount).collect();
    let base_prices: Vec<Decimal> = items.iter().map(|i| i.base_price).collect();
    let base_price_day_changes: Vec<Option<Decimal>> =
        items.iter().map(|i| i.base_price_day_change).collect();
    let base_price_month_changes: Vec<Option<Decimal>> =
        items.iter().map(|i| i.base_price_month_change).collect();
    let market_values: Vec<Decimal> = items.iter().map(|i| i.market_value).collect();
    let profit_losses: Vec<Decimal> = items.iter().map(|i| i.profit_loss).collect();
    let profit_loss_rates: Vec<Option<Decimal>> =
        items.iter().map(|i| i.profit_loss_rate).collect();
    let total_returns: Vec<Option<Decimal>> = items.iter().map(|i| i.total_return).collect();

    bulk_replace::<FundBalanceDomain>(
        pool,
        user_id,
        items.len(),
        limit,
        crate::db::query(
            r#"
            INSERT INTO fund_balances (user_id, fund_name, account, dividends_course, shares,
                                       average_purchase_price, total_purchase_amount, base_price,
                                       base_price_day_change, base_price_month_change,
                                       market_value, profit_loss, profit_loss_rate, total_return,
                                       price_as_of)
            SELECT *, (now() AT TIME ZONE 'Asia/Tokyo')::date FROM UNNEST(
                $1::uuid[], $2::text[], $3::text[], $4::text[],
                $5::numeric[], $6::numeric[], $7::numeric[], $8::numeric[],
                $9::numeric[], $10::numeric[], $11::numeric[], $12::numeric[],
                $13::numeric[], $14::numeric[]
            )
            "#,
            vec![
                Bind::from(user_ids),
                Bind::from(fund_names),
                Bind::from(accounts),
                Bind::from(dividends_courses),
                Bind::decimal_vec(shares),
                Bind::decimal_vec(average_purchase_prices),
                Bind::decimal_vec(total_purchase_amounts),
                Bind::decimal_vec(base_prices),
                Bind::optional_decimal_vec(base_price_day_changes),
                Bind::optional_decimal_vec(base_price_month_changes),
                Bind::decimal_vec(market_values),
                Bind::decimal_vec(profit_losses),
                Bind::optional_decimal_vec(profit_loss_rates),
                Bind::optional_decimal_vec(total_returns),
            ],
        ),
    )
    .await
}

/// 投資信託残高 CSV に混ざる合計・罫線行を除外する。
/// 「ファンド」が空の行は保有行になりえない（口座ごとの合計行や空行）ため取り込み対象外。
/// 種別欄が空・または「合計」を含む値がある行だけに絞り、ファンド名を持つ行は誤除外しない
fn is_fund_summary_row(row: &CsvRowView<'_>) -> bool {
    row.cell("ファンド").trim().is_empty()
        && (row.cell("投資信託種別").trim().is_empty()
            || row.values().any(|value| value.contains("合計")))
}

/// 投資信託残高 CSV の1行を CreateFundBalanceRequest に変換
///
/// "-" は値なしを表す。分配金コース・基準価額(前日比/前月比)・評価損益[％]・
/// トータルリターンは "-" が仕様上ありうるため Option で保持する
fn transform_fund_balance_row(
    row: &CsvRowView<'_>,
    row_num: RowNumber,
) -> Result<Option<CreateFundBalanceRequest>, CsvRowError> {
    let required_num = |col: &str| -> Result<Decimal, CsvRowError> {
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
    let optional_num = |col: &str| -> Result<Option<Decimal>, CsvRowError> {
        let raw = parse_optional_string(row, col);
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == "-" {
            return Ok(None);
        }
        parse_number(trimmed).map(Some).map_err(|e| CsvRowError {
            row: row_num.get(),
            message: format!("{col}: {e}"),
        })
    };
    let optional_str = |col: &str| -> String {
        let raw = parse_optional_string(row, col);
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == "-" {
            String::new()
        } else {
            trimmed.to_string()
        }
    };

    let fund_name = check_max_chars(
        normalize_security_name(&optional_str("ファンド")),
        "ファンド",
        200,
        row_num,
    )?;
    if fund_name.is_empty() {
        return Ok(None);
    }
    let account_raw = optional_str("口座区分");
    let account = Account::try_from(account_raw.as_str()).map_err(|e| CsvRowError {
        row: row_num.get(),
        message: format!("口座区分: {e}"),
    })?;
    Ok(Some(CreateFundBalanceRequest {
        fund_name,
        account,
        dividends_course: {
            let course = optional_str("分配金コース");
            (!course.is_empty()).then_some(course)
        },
        shares: required_num("保有数量[口]")?,
        average_purchase_price: required_num("平均取得価額[円]")?,
        total_purchase_amount: required_num("取得総額[円]")?,
        base_price: required_num("基準価額[円]")?,
        base_price_day_change: optional_num("基準価額(前日比)[円]")?,
        base_price_month_change: optional_num("基準価額(前月比)[円]")?,
        market_value: required_num("時価評価額[円]")?,
        profit_loss: required_num("評価損益[円]")?,
        profit_loss_rate: optional_num("評価損益[％]")?,
        total_return: optional_num("トータルリターン[円]")?,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::csv::import::CsvImport;
    use rust_decimal_macros::dec;

    const FUND_CSV_HEADER: &str = "投資信託種別,口座区分,ファンド,分配金コース,保有数量[口],(内訳　通常数量[口]),(内訳　積立数量[口]),平均取得価額[円],取得総額[円],基準価額[円],基準価額(前日比)[円],基準価額(前月比)[円],時価評価額[円],評価損益[円],評価損益[％],トータルリターン[円],通貨単位,未収分配金,参考為替レート,時価評価額[外貨],合計額[円]";
    const ROW_A: &str = "\"投資信託\",\"特定\",\"架空米国高配当ファンド(四半期決算型)\",\"受取型\",\"820,000\",\"-\",\"-\",\"10,487.80\",\"860,000\",\"11,050\",\"35\",\"210\",\"906,100\",\"46,100\",\"5.36\",\"58,300\",\"-\",\"-\",\"-\",\"-\",\"-\"";
    const ROW_B: &str = "\"投資信託\",\"NISAつみたて投資枠\",\"架空グローバル株式インデックス\",\"再投資型\",\"412,345\",\"0\",\"412,345\",\"20,808.62\",\"858,000\",\"27,160\",\"120\",\"-450\",\"1,119,929\",\"261,929\",\"30.53\",\"261,929\",\"-\",\"-\",\"-\",\"-\",\"-\"";
    const ROW_DASH: &str = "\"投資信託\",\"旧NISA\",\"架空バランスファンド\",\"-\",\"100,000\",\"-\",\"-\",\"10,000\",\"1,000,000\",\"10,500\",\"-\",\"-\",\"1,050,000\",\"50,000\",\"-\",\"-\",\"-\",\"-\",\"-\",\"-\",\"-\"";
    const ROW_BAD_SHARES: &str = "\"投資信託\",\"特定\",\"架空テストファンド\",\"再投資型\",\"N/A\",\"-\",\"-\",\"10,000\",\"1,000,000\",\"10,500\",\"35\",\"210\",\"1,050,000\",\"50,000\",\"5.00\",\"50,000\",\"-\",\"-\",\"-\",\"-\",\"-\"";
    const TOTAL_ROW: &str = "\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"19,982,000\",\"1,234,567\",\"\",\"\",\"\",\"\",\"\",\"\",\"\"";
    const ACCOUNT_TOTAL_ROW: &str = "\"\",\"特定合計\",\"\",\"\",\"\",\"\",\"\",\"\",\"3,000,000\",\"\",\"\",\"\",\"3,150,000\",\"150,000\",\"\",\"\",\"\",\"\",\"\",\"\",\"\"";

    fn make_fund_csv(rows: &[&str]) -> String {
        format!("{FUND_CSV_HEADER}\n{}", rows.join("\n"))
    }

    fn transform(rows: &[&str]) -> (Vec<CreateFundBalanceRequest>, Vec<CsvRowError>) {
        let table = parse_csv_with_config(
            make_fund_csv(rows).as_bytes(),
            &FundBalanceDomain::CSV_CONFIG,
        )
        .unwrap();
        FundBalanceDomain::transform_rows(&table)
    }

    #[test]
    fn test_parse_fund_balance_row() {
        let (items, errors) = transform(&[ROW_A]);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        let item = &items[0];
        assert_eq!(item.fund_name, "架空米国高配当ファンド(四半期決算型)");
        assert_eq!(item.account.as_str(), "特定");
        assert_eq!(item.dividends_course.as_deref(), Some("受取型"));
        assert_eq!(
            (
                item.shares,
                item.average_purchase_price,
                item.total_purchase_amount,
                item.base_price,
                item.base_price_day_change,
                item.base_price_month_change,
                item.market_value,
                item.profit_loss,
                item.profit_loss_rate,
                item.total_return,
            ),
            (
                dec!(820000),
                dec!(10487.80),
                dec!(860000),
                dec!(11050),
                Some(dec!(35)),
                Some(dec!(210)),
                dec!(906100),
                dec!(46100),
                Some(dec!(5.36)),
                Some(dec!(58300)),
            )
        );
    }

    #[test]
    fn test_parse_dash_as_none() {
        let (items, errors) = transform(&[ROW_DASH]);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        let item = &items[0];
        assert_eq!(item.dividends_course, None);
        assert_eq!(item.base_price_day_change, None);
        assert_eq!(item.base_price_month_change, None);
        assert_eq!(item.profit_loss_rate, None);
        assert_eq!(item.total_return, None);
    }

    #[test]
    fn test_parse_error_row() {
        let (items, errors) = transform(&[ROW_BAD_SHARES]);
        assert!(items.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("保有数量"), "{errors:?}");
    }

    #[test]
    fn test_summary_rows_excluded() {
        let (items, errors) = transform(&[ROW_A, ROW_B, TOTAL_ROW, ACCOUNT_TOTAL_ROW]);
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn test_preview_with_preamble_lines() {
        // 実際の出力ファイル同様に説明行が先頭にあっても見出しで検出する
        let csv = format!("■投資信託残高\n\n{}\n{}", FUND_CSV_HEADER, ROW_A);
        let preview = FundBalanceDomain::preview_csv(csv.as_bytes()).unwrap();
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.is_empty(),
            ),
            (1, 1, true),
            "unexpected errors: {:?}",
            preview.errors
        );
    }

    /// 置換型のため、エラー行・有効行0件のCSVは DB に触れる前に拒否する。
    /// lazy pool のまま Validation が返れば、bulk_create(全削除+挿入)に
    /// 到達していないことになる
    #[tokio::test]
    async fn test_upload_csv_rejects_invalid_or_empty_before_db() {
        let pool = crate::db::connect_pool_lazy("postgresql://user:password@localhost/test_db", 1)
            .expect("lazy pool の構築に失敗");
        let user_id = UserId::default();

        for (label, csv) in [
            ("1行不正", make_fund_csv(&[ROW_A, ROW_BAD_SHARES])),
            ("有効行0件", make_fund_csv(&[TOTAL_ROW])),
            ("データ行なし", make_fund_csv(&[])),
        ] {
            let err = FundBalanceDomain::upload_csv(
                &pool,
                user_id,
                csv.as_bytes(),
                RowLimit::new(i64::MAX),
            )
            .await
            .expect_err(label);
            assert!(
                matches!(err, ApiError::Validation(_)),
                "{label}: expected Validation, got {err:?}"
            );
        }
    }

    #[test]
    fn test_fund_balance_search_query_params_delegate_include_flags() {
        use crate::models::common::SearchParamsAccessor;
        let mut params = FundBalanceSearchQueryParams::default();
        assert!(!params.should_include_summary());
        assert!(!params.should_include_facets());

        params.search.include_summary = Some(true);
        params.search.include_facets = Some(true);
        assert!(params.should_include_summary());
        assert!(params.should_include_facets());
    }
}
