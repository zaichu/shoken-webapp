use crate::db::{Bind, Db};
use crate::errors::ApiError;
use crate::models::asset_balance::{
    AssetBalance, AssetBalanceSearchQueryParams, AssetBalanceSummary, CreateAssetBalanceRequest,
};
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::CsvRowError;
use crate::services::csv::import::{CsvImport, validate_csv_rows};
#[cfg(test)]
use crate::services::csv::pipeline::parse_csv_with_config;
use crate::services::csv::pipeline::{CsvParserConfig, CsvTable};
use crate::services::csv::util::{
    CsvRowView, RowNumber, check_max_chars, parse_number, parse_optional_string,
};
use crate::services::domain::bulk::{RowLimit, bulk_replace, user_ids_for_bulk_insert};
use crate::services::domain::facets::{FacetSlot, FacetSpec};
use crate::services::domain::search::Search;
use crate::services::domain::search_filters::{FilterField, NO_FIELD, SearchFilter};
use crate::services::domain::{Domain, WriteMode};
use rust_decimal::Decimal;
use shared::normalize::normalize_security_name;
use shared::value::{SecurityCode, UserId};

/// 資産残高（保有銘柄）ドメイン
pub struct AssetBalanceDomain;

impl Domain for AssetBalanceDomain {
    const NAME: &'static str = "asset_balance";
    const TABLE: &'static str = "asset_balances";
    const WRITE_MODE: WriteMode = WriteMode::Replace;
}

impl CsvImport for AssetBalanceDomain {
    type Row = CreateAssetBalanceRequest;
    const CSV_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 6,
        exclude_row_fn: Some(is_account_summary_row),
        header_marker: None,
    };

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
        let (items, errors) = validate_csv_rows(table, transform_asset_balance_row);
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
    /// （有効行だけで既存の残高・共有価格を置き換えてしまわないように）
    fn ensure_importable(items: &[Self::Row], errors: &[CsvRowError]) -> Result<(), ApiError> {
        if let Some(first) = errors.first() {
            return Err(ApiError::Validation(format!(
                "{}行目を含む{}件のエラーがあるため保有銘柄の取り込みを中止しました（既存のデータは変更されていません）",
                first.row,
                errors.len()
            )));
        }
        if items.is_empty() {
            return Err(ApiError::Validation(
                "取り込める行がないため保有銘柄の取り込みを中止しました（既存のデータは変更されていません）"
                    .to_string(),
            ));
        }
        Ok(())
    }
}

// priced_* は価格あり(current_price > 0)行だけの集計。
// 含み損益の分子・分母から価格なし行を外し、frontend の個別カード計算と同じ対象行にする
// 価格は security_prices 由来のため、評価額は shares × current_price、
// 前日比の合計は保有数量で重み付けした daily_change × shares の合計
const SUMMARY_SQL: &str = "SELECT COALESCE(SUM(shares * current_price), 0) AS total_market_value, \
     COALESCE(SUM(total_purchase_amount), 0) AS total_purchase_amount, \
     COALESCE(SUM(daily_change * shares), 0) AS total_daily_change, \
     COALESCE(SUM(CASE WHEN current_price > 0 THEN current_price * shares ELSE 0 END), 0) \
     AS priced_market_value, \
     COALESCE(SUM(CASE WHEN current_price > 0 THEN total_purchase_amount ELSE 0 END), 0) \
     AS priced_purchase_amount \
     FROM asset_balances_view";

impl Search for AssetBalanceDomain {
    type Data = AssetBalance;
    type Params = AssetBalanceSearchQueryParams;
    type Summary = AssetBalanceSummary;

    const COLUMNS: &'static str = "id, user_id, security_code, security_name, shares, \
        average_purchase_price, total_purchase_amount, current_price, \
        daily_change, price_as_of, created_at, updated_at";
    /// 価格列は共有の security_prices から読むため JOIN 済みビューを使う
    const FROM: &'static str = "asset_balances_view";
    const ORDER_BY: &'static str = " ORDER BY security_code ASC, id ASC";
    /// asset_balances には snapshot 日付がないため、date 系パラメータは解釈しない
    const DATE_COLUMN: Option<&'static str> = None;
    const FILTER_FIELDS: &'static [FilterField] =
        &[FilterField::SecurityCode, FilterField::SecurityName];
    const FACETS: &'static [FacetSpec] = &[FacetSpec::security(FacetSlot::Securities, "id")];

    fn filter_value(params: &AssetBalanceSearchQueryParams, field: FilterField) -> &Option<String> {
        match field {
            FilterField::SecurityCode => &params.security_code,
            FilterField::SecurityName => &params.security_name,
            _ => &NO_FIELD,
        }
    }

    /// 検索条件全体の summary を算出する（ページ内だけでなく検索条件全体の合計）
    async fn fetch_summary(
        pool: &Db,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> Result<AssetBalanceSummary, ApiError> {
        Ok(Self::filtered_query(SUMMARY_SQL, user_id, filter)
            .build_query_as::<AssetBalanceSummary>()
            .fetch_one(pool)
            .await?)
    }
}

/// 保有銘柄を一括登録（既存データを全削除してから挿入）
pub async fn bulk_create(
    pool: &Db,
    user_id: UserId,
    items: &[CreateAssetBalanceRequest],
    limit: RowLimit,
) -> Result<BulkCreateResponse, ApiError> {
    let user_ids = user_ids_for_bulk_insert(user_id, items.len());
    let security_codes: Vec<String> = items
        .iter()
        .map(|i| i.security_code.as_str().to_owned())
        .collect();
    let security_names: Vec<String> = items.iter().map(|i| i.security_name.clone()).collect();
    let shares: Vec<Decimal> = items.iter().map(|i| i.shares).collect();
    let average_purchase_prices: Vec<Decimal> =
        items.iter().map(|i| i.average_purchase_price).collect();
    let total_purchase_amounts: Vec<Decimal> =
        items.iter().map(|i| i.total_purchase_amount).collect();
    // 個人 CSV の現在値・前日比は csv_* 列へ保存する。共有 security_prices 側の確定値は
    // ビューで優先採用されるため、CSV 再取込では共有行に触れない
    let current_prices: Vec<Decimal> = items.iter().map(|i| i.current_price).collect();
    let daily_changes: Vec<Decimal> = items.iter().map(|i| i.daily_change).collect();

    bulk_replace::<AssetBalanceDomain>(
        pool,
        user_id,
        items.len(),
        limit,
        crate::db::query(
            r#"
            INSERT INTO asset_balances (user_id, security_code, security_name, shares,
                                        average_purchase_price, total_purchase_amount,
                                        csv_current_price, csv_daily_change)
            SELECT * FROM UNNEST(
                $1::uuid[], $2::text[], $3::text[], $4::numeric[],
                $5::numeric[], $6::numeric[], $7::numeric[], $8::numeric[]
            )
            "#,
            vec![
                Bind::from(user_ids),
                Bind::from(security_codes),
                Bind::from(security_names),
                Bind::decimal_vec(shares),
                Bind::decimal_vec(average_purchase_prices),
                Bind::decimal_vec(total_purchase_amounts),
                Bind::decimal_vec(current_prices),
                Bind::decimal_vec(daily_changes),
            ],
        ),
    )
    .await
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
    use crate::services::csv::import::CsvImport;
    use rust_decimal_macros::dec;

    type ExpectedRow<'a> = (&'a str, Decimal, Decimal, Decimal, Decimal, Decimal);

    const HEADER: &str = "銘柄コード,銘柄名,保有数量［株］,執行中［株］,平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";
    const ASSET_BALANCE_CSV_HEADER: &str = "銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";

    const TEST_ROW_1: &str = "1234,テスト株式会社,100,0,100,0,1500,150000,1600,10,160000,6.67";
    const TEST_ROW_2: &str = "5678,サンプル株式会社,200,0,200,0,1800,360000,1900,15,380000,5.56";
    const INPEX_ROW: &str = "\"9901\",\"架空資源開発\",\"200\",\"0\",\"200\",\"0\",\"2,000.00\",\"400,000\",\"3,000.0\",\"50.0\",\"600,000\",\"50.00\"";
    const NINTENDO_ROW: &str = "\"9905\",\"架空ゲーム\",\"1,000\",\"0\",\"1,000\",\"0\",\"5,000.00\",\"5,000,000\",\"7,000.0\",\"100.0\",\"7,000,000\",\"40.00\"";
    const ACCOUNT_SUMMARY_ROW: &str = ",,,,,,特定口座合計,\"5,400,000\",,,\"7,600,000\",\"40.74\"";
    const NINTENDO_BAD_SHARES_ROW: &str = "\"9905\",\"架空ゲーム\",\"N/A\",\"0\",\"1,000\",\"0\",\"5,000.00\",\"5,000,000\",\"7,000.0\",\"100.0\",\"7,000,000\",\"40.00\"";
    const INPEX_BAD_PRICE_ROW: &str = "\"9901\",\"架空資源開発\",\"200\",\"0\",\"200\",\"0\",\"2,000.00\",\"400,000\",\"不明\",\"50.0\",\"600,000\",\"50.00\"";

    fn parse_row_csv(row: &str) -> (Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>) {
        let table = parse_csv_with_config(
            format!("{HEADER}\n{row}\n").as_bytes(),
            &CsvParserConfig {
                skip_header_rows: 0,
                exclude_row_fn: None,
                header_marker: None,
            },
        )
        .unwrap();

        AssetBalanceDomain::transform_rows(&table)
    }

    fn make_asset_balance_csv(rows: &[&str]) -> String {
        format!(
            "■現在の評価額合計［円］,,\"7,600,000\"\n\
             ■評価損益合計,前日比［円］,\"150,000\"\n\
             ,前月比［円］,\"-120,000\"\n\
             ,評価損益［円］,\"2,200,000\"\n\
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
                "9904-1,テスト,100,-,1500,150000,1600,10,160000,6.67",
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
                &["9901", "9905"][..],
            ),
        ] {
            let table = parse_csv_with_config(
                make_asset_balance_csv(rows).as_bytes(),
                &AssetBalanceDomain::CSV_CONFIG,
            )
            .unwrap();
            let (items, errors) = AssetBalanceDomain::transform_rows(&table);

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

    /// 置換型のため、エラー行・有効行0件のCSVは DB に触れる前に拒否する。
    /// lazy pool のまま Validation が返れば、bulk_create(全削除+挿入)に
    /// 到達していないことになる
    #[tokio::test]
    async fn test_upload_csv_rejects_invalid_or_empty_before_db() {
        let pool = crate::db::connect_pool_lazy("postgresql://user:password@localhost/test_db", 1)
            .expect("lazy pool の構築に失敗");
        let user_id = UserId::default();

        for (label, csv) in [
            (
                "1行不正",
                make_asset_balance_csv(&[INPEX_ROW, NINTENDO_BAD_SHARES_ROW]),
            ),
            (
                "全行不正",
                make_asset_balance_csv(&[NINTENDO_BAD_SHARES_ROW, INPEX_BAD_PRICE_ROW]),
            ),
            ("有効行0件", make_asset_balance_csv(&[ACCOUNT_SUMMARY_ROW])),
            ("データ行なし", make_asset_balance_csv(&[])),
        ] {
            let err = AssetBalanceDomain::upload_csv(
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

        // 空バイト列は従来通りパース段階の CSV エラー
        let err = AssetBalanceDomain::upload_csv(&pool, user_id, b"", RowLimit::new(i64::MAX))
            .await
            .expect_err("空バイト列");
        assert!(matches!(err, ApiError::Csv(_)), "got {err:?}");
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
        assert!(!is_summary("9904", "架空自動車"));
    }
}
