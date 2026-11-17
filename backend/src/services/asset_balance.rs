use crate::db::{Bind, Db, Tx};
use crate::errors::ApiError;
use crate::models::asset_balance::{
    AssetBalance, AssetBalanceSearchQueryParams, AssetBalanceSummary, CreateAssetBalanceRequest,
};
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::CsvRowError;
use crate::services::csv::import::CsvImport;
#[cfg(test)]
use crate::services::csv::pipeline;
use crate::services::csv::pipeline::{CsvParserConfig, CsvTable};
use crate::services::csv::util;
use crate::services::csv::util::{CsvRowView, RowNumber};
use crate::services::domain::bulk;
use crate::services::domain::bulk::RowLimit;
use crate::services::domain::facets::{FacetSlot, FacetSpec};
use crate::services::domain::search::Search;
use crate::services::domain::search_filters;
use crate::services::domain::search_filters::{FilterField, SearchFilter};
use crate::services::domain::{Domain, WriteMode};
use rust_decimal::Decimal;
use shared::normalize::normalize_security_name;
use shared::value::{Account, SecurityCode, UserId};

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
        skip_header_rows: 0,
        exclude_row_fn: None,
        // 前置き行数は口座数で変わるため、「銘柄コード」で始まる見出し行を探す。
        // 「■口座名」見出し・「口座合計」区切り・口座ごとの見出し繰り返しも
        // 口座判定の材料になるため、パースでは除かず transform_rows で処理する
        header_marker: Some("銘柄コード"),
    };

    fn transform_rows(table: &CsvTable) -> (Vec<Self::Row>, Vec<CsvRowError>) {
        let rows: Vec<CsvRowView<'_>> = table.rows().collect();
        let accounts = assign_row_accounts(&rows);
        convert_holding_rows(&rows, &accounts)
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
// has_missing_daily_change は前日終値が取れない行(共有価格あり・prev_close なし)の有無。
// 前日比は合計で 0 として扱われるため、合計自体は正しいが「除いた」注記の判定に使う
const SUMMARY_SQL: &str = "SELECT COALESCE(SUM(shares * current_price), 0) AS total_market_value, \
     COALESCE(SUM(total_purchase_amount), 0) AS total_purchase_amount, \
     COALESCE(SUM(daily_change * shares), 0) AS total_daily_change, \
     COALESCE(SUM(CASE WHEN current_price > 0 THEN current_price * shares ELSE 0 END), 0) \
     AS priced_market_value, \
     COALESCE(SUM(CASE WHEN current_price > 0 THEN total_purchase_amount ELSE 0 END), 0) \
     AS priced_purchase_amount, \
     COALESCE(BOOL_OR(price_as_of IS NOT NULL AND prev_close IS NULL), false) \
     AS has_missing_daily_change \
     FROM asset_balances_view";

impl Search for AssetBalanceDomain {
    type Data = AssetBalance;
    type Params = AssetBalanceSearchQueryParams;
    type Summary = AssetBalanceSummary;

    const COLUMNS: &'static str = "id, user_id, security_code, security_name, account, shares, \
        average_purchase_price, total_purchase_amount, current_price, \
        daily_change, price_as_of, prev_close, created_at, updated_at";
    /// 価格列は共有の security_prices から読むため JOIN 済みビューを使う
    const FROM: &'static str = "asset_balances_view";
    const ORDER_BY: &'static str = " ORDER BY security_code ASC, id ASC";
    /// asset_balances には snapshot 日付がないため、date 系パラメータは解釈しない
    const DATE_COLUMN: Option<&'static str> = None;
    const FILTER_FIELDS: &'static [FilterField] = &[
        FilterField::SecurityCode,
        FilterField::SecurityName,
        FilterField::Account,
    ];
    // 口座の group facet は追加しない。旧行は account が NULL で FacetOption
    // (String 非NULL)に乗せられず、現状 facet を参照する画面もないため
    const FACETS: &'static [FacetSpec] = &[FacetSpec::security(FacetSlot::Securities, "id")];

    fn filter_value(params: &AssetBalanceSearchQueryParams, field: FilterField) -> &Option<String> {
        match field {
            FilterField::SecurityCode => &params.security_code,
            FilterField::SecurityName => &params.security_name,
            FilterField::Account => &params.account,
            _ => &search_filters::NO_FIELD,
        }
    }

    /// 検索条件全体の summary を算出する（ページ内だけでなく検索条件全体の合計）
    async fn fetch_summary(
        tx: &mut Tx,
        user_id: UserId,
        filter: &SearchFilter,
    ) -> Result<AssetBalanceSummary, ApiError> {
        Ok(Self::filtered_query(SUMMARY_SQL, user_id, filter)
            .build_query_as::<AssetBalanceSummary>()
            .fetch_one(&mut *tx)
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
    let user_ids = bulk::user_ids_for_bulk_insert(user_id, items.len());
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
    let accounts: Vec<Option<String>> = items
        .iter()
        .map(|i| i.account.as_ref().map(|a| a.as_str().to_owned()))
        .collect();

    bulk::bulk_replace::<AssetBalanceDomain>(
        pool,
        user_id,
        items.len(),
        limit,
        crate::db::query(
            r#"
            INSERT INTO asset_balances (user_id, security_code, security_name, shares,
                                        average_purchase_price, total_purchase_amount,
                                        csv_current_price, csv_daily_change, account)
            SELECT * FROM UNNEST(
                $1::uuid[], $2::text[], $3::text[], $4::numeric[],
                $5::numeric[], $6::numeric[], $7::numeric[], $8::numeric[],
                $9::text[]
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
                Bind::from(accounts),
            ],
        ),
    )
    .await
}

/// 保有銘柄 CSV に混ざる「特定口座合計」などの口座集計行かどうか
///
/// 先頭フィールド（銘柄コード）が空の行かつ「口座合計」を含む行のみ集計行とする。
/// 銘柄名に「口座合計」を含む銘柄を誤判定しないよう、先頭が空であることを条件とする。
fn is_account_summary_row(row: &CsvRowView<'_>) -> bool {
    row.cell("銘柄コード").trim().is_empty() && row.values().any(|value| value.contains("口座合計"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RowKind {
    /// 保有行
    Holding,
    /// 「○○口座合計」の口座区切り行
    Separator,
    /// 「■口座名」のセクション見出し
    Heading,
    /// 口座ごとに繰り返される列見出し
    Structural,
}

fn classify_row(row: &CsvRowView<'_>) -> RowKind {
    let first = row.cell("銘柄コード");
    if first.starts_with('■') {
        RowKind::Heading
    } else if first == "銘柄コード" {
        RowKind::Structural
    } else if is_account_summary_row(row) {
        RowKind::Separator
    } else {
        RowKind::Holding
    }
}

/// 「○○口座合計」の区切り行から口座名を取り出す。名前が取れない区切りは None
fn separator_account_name(row: &CsvRowView<'_>) -> Option<Account> {
    let raw = row.values().find(|value| value.contains("口座合計"))?;
    let name = raw
        .strip_suffix("口座合計")
        .or_else(|| raw.strip_suffix("合計"))
        .unwrap_or(raw)
        .trim();
    Account::try_from(name).ok()
}

/// 「■○○口座」のセクション見出しから口座名を取り出す。口座見出し以外は None
fn heading_account_name(row: &CsvRowView<'_>) -> Option<Account> {
    let raw = row.cell("銘柄コード").strip_prefix('■')?.trim();
    let name = raw.strip_suffix("口座")?.trim();
    Account::try_from(name).ok()
}

/// 各行に口座を付ける。区切りが無い・区切りより前の行は口座を持たない(推測しない)
///
/// 実際の証券会社 CSV は「■口座名」見出し → 列見出し → 保有行 → 「口座合計」集計行の
/// ブロック構造なので、見出し・区切りのどちらが出てもその口座名を後続の保有行へ付ける。
fn assign_row_accounts(rows: &[CsvRowView<'_>]) -> Vec<Option<Account>> {
    let kinds: Vec<RowKind> = rows.iter().map(classify_row).collect();
    let mut cursor = AccountCursor::default();
    let mut accounts: Vec<Option<Account>> = vec![None; rows.len()];
    for (index, row) in rows.iter().enumerate() {
        match kinds[index] {
            RowKind::Heading => cursor.apply_heading(row),
            RowKind::Separator => {
                apply_separator(&mut accounts, &kinds, &mut cursor, rows[index], index);
            }
            RowKind::Structural => {}
            RowKind::Holding => accounts[index] = cursor.holding_account(),
        }
    }
    accounts
}

#[derive(Default)]
struct AccountCursor {
    current: Option<Account>,
}

impl AccountCursor {
    fn apply_heading(&mut self, row: &CsvRowView<'_>) {
        self.current = heading_account_name(row);
    }

    fn holding_account(&self) -> Option<Account> {
        self.current.clone()
    }
}

/// 区切り行を現在口座へ反映する。末尾の区切りは直前の未割当の保有行を閉じる集計行と
/// みなし、後続ブロック(■見出し・保有行)が続く区切りは前後どちらの区切りか分からない
/// ため直前の行にはさかのぼって付けない
fn apply_separator(
    accounts: &mut [Option<Account>],
    kinds: &[RowKind],
    cursor: &mut AccountCursor,
    row: CsvRowView<'_>,
    index: usize,
) {
    let name = separator_account_name(&row);
    if is_trailing_summary(kinds, index)
        && let Some(name) = &name
    {
        backfill_unassigned_holdings(accounts, kinds, index, name);
    }
    cursor.current = name;
}

fn is_trailing_summary(kinds: &[RowKind], index: usize) -> bool {
    matches!(kinds.get(index + 1), None | Some(RowKind::Structural))
}

fn backfill_unassigned_holdings(
    accounts: &mut [Option<Account>],
    kinds: &[RowKind],
    separator_index: usize,
    name: &Account,
) {
    for i in (0..separator_index).rev() {
        if kinds[i] != RowKind::Holding || accounts[i].is_some() {
            break;
        }
        accounts[i] = Some(name.clone());
    }
}

/// 保有行だけを変換する。行番号は表全体での位置を通す
fn convert_holding_rows(
    rows: &[CsvRowView<'_>],
    accounts: &[Option<Account>],
) -> (Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>) {
    let mut items = Vec::new();
    let mut errors = Vec::new();
    for (index, (row, account)) in rows.iter().zip(accounts).enumerate() {
        if classify_row(row) != RowKind::Holding {
            continue;
        }
        match transform_asset_balance_row(row, RowNumber::new(index + 1)) {
            Ok(Some(item)) => items.push(CreateAssetBalanceRequest {
                account: account.clone(),
                ..item
            }),
            Ok(None) => {}
            Err(error) => errors.push(error),
        }
    }
    (items, errors)
}

fn required_amount(
    row: &CsvRowView<'_>,
    col: &str,
    row_num: RowNumber,
) -> Result<Decimal, CsvRowError> {
    let raw = util::parse_optional_string(row, col);
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed == "-" {
        return Err(CsvRowError {
            row: row_num.get(),
            message: format!("必須列 '{col}' が空または値なし"),
        });
    }
    util::parse_number(trimmed).map_err(|e| CsvRowError {
        row: row_num.get(),
        message: format!("{col}: {e}"),
    })
}

fn fallback_amount(row: &CsvRowView<'_>, col: &str) -> Decimal {
    util::parse_number(&util::parse_optional_string(row, col)).unwrap_or(Decimal::ZERO)
}

fn holding_security_code(
    row: &CsvRowView<'_>,
    row_num: RowNumber,
) -> Result<Option<SecurityCode>, CsvRowError> {
    let code_raw = util::parse_optional_string(row, "銘柄コード").replace('"', "");
    if code_raw.trim().is_empty() {
        return Ok(None);
    }
    SecurityCode::try_from(code_raw.as_str())
        .map(Some)
        .map_err(|e| CsvRowError {
            row: row_num.get(),
            message: format!("銘柄コード: {e}"),
        })
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
    // 必須の数値を先に検証し、銘柄コードが空でも不正な行はエラーとして報告する
    let shares = required_amount(row, "保有数量［株］", row_num)?;
    let average_purchase_price = required_amount(row, "平均取得価額［円］", row_num)?;
    let total_purchase_amount = required_amount(row, "取得総額［円］", row_num)?;
    let current_price = required_amount(row, "現在値［円］", row_num)?;
    let market_value = required_amount(row, "時価評価額［円］", row_num)?;

    // 銘柄コードが空の行（「口座合計」以外の集計・罫線行）は取り込み対象外
    let Some(security_code) = holding_security_code(row, row_num)? else {
        return Ok(None);
    };
    Ok(Some(CreateAssetBalanceRequest {
        security_code,
        // 口座は行単位では持たず、区切り行からの付与で transform_rows が上書きする
        account: None,
        security_name: util::check_max_chars(
            normalize_security_name(&util::parse_optional_string(row, "銘柄名")),
            "銘柄名",
            200,
            row_num,
        )?,
        shares,
        average_purchase_price,
        total_purchase_amount,
        current_price,
        // 前日比は変動なし時に 0 または "-" が仕様上ありうるため 0.0 フォールバック
        daily_change: fallback_amount(row, "現在値（前日比）［円］"),
        market_value,
        // 評価損益は NISA 等で表示されない場合に "-" が仕様上ありうるため 0.0 フォールバック
        profit_loss_rate: fallback_amount(row, "評価損益［％］"),
    }))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::services::csv::import::CsvImport;
    use rust_decimal_macros::dec;

    type ExpectedRow<'a> = (&'a str, Decimal, Decimal, Decimal, Decimal, Decimal);

    const HEADER: &str = "銘柄コード,銘柄名,保有数量［株］,執行中［株］,平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";
    const ASSET_BALANCE_CSV_HEADER: &str = "銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";

    const TEST_ROW_1: &str = "9931,テスト株式会社,100,0,100,0,1500,150000,1600,10,160000,6.67";
    const TEST_ROW_2: &str = "9932,サンプル株式会社,200,0,200,0,1800,360000,1900,15,380000,5.56";
    const SYN_RESOURCE_ROW: &str = "\"9901\",\"架空資源開発\",\"200\",\"0\",\"200\",\"0\",\"2,000.00\",\"400,000\",\"3,000.0\",\"50.0\",\"600,000\",\"50.00\"";
    const SYN_GAME_ROW: &str = "\"9905\",\"架空ゲーム\",\"1,000\",\"0\",\"1,000\",\"0\",\"5,000.00\",\"5,000,000\",\"7,000.0\",\"100.0\",\"7,000,000\",\"40.00\"";
    const ACCOUNT_SUMMARY_ROW: &str = ",,,,,,特定口座合計,\"5,400,000\",,,\"7,600,000\",\"40.74\"";
    const SYN_GAME_BAD_SHARES_ROW: &str = "\"9905\",\"架空ゲーム\",\"N/A\",\"0\",\"1,000\",\"0\",\"5,000.00\",\"5,000,000\",\"7,000.0\",\"100.0\",\"7,000,000\",\"40.00\"";
    const SYN_RESOURCE_BAD_PRICE_ROW: &str = "\"9901\",\"架空資源開発\",\"200\",\"0\",\"200\",\"0\",\"2,000.00\",\"400,000\",\"不明\",\"50.0\",\"600,000\",\"50.00\"";

    fn parse_row_csv(row: &str) -> (Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>) {
        let table = pipeline::parse_csv_with_config(
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
                "9931,テスト株式会社,100,-,1500,150000,1600,10,160000,6.67",
                (
                    "9931",
                    dec!(100),
                    dec!(1500),
                    dec!(1600),
                    dec!(10),
                    dec!(6.67),
                ),
            ),
            (
                "9932,ファンド,50,-,2000,100000,2100,-,105000,-",
                (
                    "9932",
                    dec!(50),
                    dec!(2000),
                    dec!(2100),
                    Decimal::ZERO,
                    Decimal::ZERO,
                ),
            ),
            // 前後の空白は trim してから検証する
            (
                " 9931 ,テスト株式会社,100,-,1500,150000,1600,10,160000,6.67",
                (
                    "9931",
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
            ("9931,テスト,N/A,-,1500,150000,1600,0,160000,0", "保有数量"),
            ("9931,テスト,-,-,1500,150000,1600,0,160000,0", "保有数量"),
            ("9931,テスト,100,-,1500,150000,N/A,0,160000,0", "現在値"),
            // 記号入りの銘柄コードは行エラー(newtype 化後の仕様)
            (
                "9904-1,テスト,100,-,1500,150000,1600,10,160000,6.67",
                "銘柄コード",
            ),
        ] {
            assert_row_error(row, expected_message);
        }

        let preview = AssetBalanceDomain::preview_csv(
            make_asset_balance_csv(&[SYN_RESOURCE_ROW, SYN_GAME_ROW, ACCOUNT_SUMMARY_ROW])
                .as_bytes(),
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
                &["9931", "9932"][..],
            ),
            (
                &[SYN_RESOURCE_ROW, SYN_GAME_ROW, ACCOUNT_SUMMARY_ROW][..],
                &["9901", "9905"][..],
            ),
        ] {
            let table = pipeline::parse_csv_with_config(
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
                make_asset_balance_csv(&[SYN_RESOURCE_ROW, SYN_GAME_BAD_SHARES_ROW]),
            ),
            (
                "全行不正",
                make_asset_balance_csv(&[SYN_GAME_BAD_SHARES_ROW, SYN_RESOURCE_BAD_PRICE_ROW]),
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

    /// 口座区切りの有無・位置ごとに、行へ付く口座を確かめる(架空データのみ)
    #[test]
    fn test_assign_row_accounts() {
        const SEP_TOKUTEI: &str = ",,,,,,特定口座合計,,,,,";
        const SEP_NISA: &str = ",,,,,,NISA口座合計,,,,,";
        const SEP_NISA_GROWTH: &str = ",,,,,,NISA成長投資枠口座合計,,,,,";

        let acc = |s: Option<&str>| s.map(str::to_string);
        let accounts_of = |lines: &[&str]| {
            let table = pipeline::parse_csv_with_config(
                format!("{HEADER}\n{}", lines.join("\n")).as_bytes(),
                &AssetBalanceDomain::CSV_CONFIG,
            )
            .unwrap();
            let (items, errors) = AssetBalanceDomain::transform_rows(&table);
            assert!(errors.is_empty(), "unexpected errors: {errors:?}");
            items
                .iter()
                .map(|item| item.account.as_ref().map(|a| a.as_str().to_string()))
                .collect::<Vec<_>>()
        };

        // 区切りが1つ: 区切りに続く保有に口座を付ける
        assert_eq!(
            accounts_of(&[SEP_TOKUTEI, TEST_ROW_1, TEST_ROW_2]),
            [acc(Some("特定")), acc(Some("特定"))]
        );
        // 区切りが複数: 各区切りに続く保有にそれぞれ付ける
        assert_eq!(
            accounts_of(&[SEP_TOKUTEI, TEST_ROW_1, SEP_NISA, TEST_ROW_2]),
            [acc(Some("特定")), acc(Some("NISA"))]
        );
        // 区切りが無い: どの行にも口座を付けない
        assert_eq!(accounts_of(&[TEST_ROW_1, TEST_ROW_2]), [None, None]);
        // 区切りより前の行には口座を付けない
        assert_eq!(
            accounts_of(&[TEST_ROW_1, SEP_TOKUTEI, TEST_ROW_2]),
            [acc(None), acc(Some("特定"))]
        );
        // 実際のCSV形式: 「口座合計」がブロック末尾の集計行のときは直前の保有に付く
        assert_eq!(
            accounts_of(&[TEST_ROW_1, TEST_ROW_2, SEP_TOKUTEI]),
            [acc(Some("特定")), acc(Some("特定"))]
        );
        // 「■口座名」見出し: 見出しに続く保有に口座を付ける
        assert_eq!(
            accounts_of(&["■特定口座", HEADER, TEST_ROW_1, TEST_ROW_2]),
            [acc(Some("特定")), acc(Some("特定"))]
        );
        // 実際のCSV形式・複数口座: 「■口座名」見出しごとに口座が切り替わる
        assert_eq!(
            accounts_of(&[
                "■特定口座",
                HEADER,
                TEST_ROW_1,
                SEP_TOKUTEI,
                "■NISA口座",
                HEADER,
                TEST_ROW_2,
                SEP_NISA
            ]),
            [acc(Some("特定")), acc(Some("NISA"))]
        );
        // 区切りの直後に別の口座ブロック(■見出し)が続くときは、前後どちらの区切りか
        // 分からないため直前の行には口座を付けない
        assert_eq!(
            accounts_of(&[TEST_ROW_1, SEP_TOKUTEI, "■NISA口座", HEADER, TEST_ROW_2]),
            [acc(None), acc(Some("NISA"))]
        );
        // 「○○口座合計」以外の区切り名も取り出せる
        assert_eq!(
            accounts_of(&[SEP_NISA_GROWTH, TEST_ROW_1]),
            [acc(Some("NISA成長投資枠"))]
        );
        // 区切り行自体は取り込み行にならない
        let table = pipeline::parse_csv_with_config(
            format!("{HEADER}\n{SEP_TOKUTEI}\n{TEST_ROW_1}\n").as_bytes(),
            &AssetBalanceDomain::CSV_CONFIG,
        )
        .unwrap();
        assert_eq!(AssetBalanceDomain::transform_rows(&table).0.len(), 1);
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

    #[test]
    fn test_classify_row() {
        use crate::services::csv::util::HeaderIndex;

        let index = HeaderIndex::new(["銘柄コード", "銘柄名"]);
        let kind_of = |code: &str, name: &str| {
            let record = csv::StringRecord::from(vec![code, name]);
            classify_row(&CsvRowView::new(&record, &index))
        };

        assert_eq!(kind_of("■特定口座", ""), RowKind::Heading);
        assert_eq!(kind_of("銘柄コード", ""), RowKind::Structural);
        assert_eq!(kind_of("", "特定口座合計"), RowKind::Separator);
        assert_eq!(kind_of("9901", "架空資源開発"), RowKind::Holding);
        // コード入りで銘柄名に「口座合計」を含めても保有行のまま
        assert_eq!(kind_of("9999", "口座合計を含む名称"), RowKind::Holding);
        // コードも集計文言も無い行は保有行として変換側へ回す
        assert_eq!(kind_of("", "普通株式"), RowKind::Holding);
        assert_eq!(kind_of("", ""), RowKind::Holding);
    }

    #[test]
    fn test_row_accounts_with_headings_blank_and_invalid_rows() {
        const SEP_TOKUTEI: &str = ",,,,,,特定口座合計,,,,,";
        const BAD_SHARES_ROW: &str =
            "9905,架空ゲーム,N/A,0,1000,5000,5000000,7000,100,7000000,40.00";

        let accounts_and_errors = |lines: &[&str]| {
            let table = pipeline::parse_csv_with_config(
                format!("{HEADER}\n{}", lines.join("\n")).as_bytes(),
                &AssetBalanceDomain::CSV_CONFIG,
            )
            .unwrap();
            let (items, errors) = AssetBalanceDomain::transform_rows(&table);
            (
                items
                    .iter()
                    .map(|item| {
                        (
                            item.security_code.as_str().to_string(),
                            item.account.as_ref().map(|a| a.as_str().to_string()),
                        )
                    })
                    .collect::<Vec<_>>(),
                errors.len(),
            )
        };

        // 空行はパース段階で除かれるため口座割当に影響しない
        let (items, errors) = accounts_and_errors(&[
            "■特定口座",
            HEADER,
            TEST_ROW_1,
            "",
            ",,,,,,,,,,,",
            TEST_ROW_2,
        ]);
        assert_eq!(errors, 0);
        assert_eq!(
            items,
            [
                ("9931".to_string(), Some("特定".to_string())),
                ("9932".to_string(), Some("特定".to_string())),
            ]
        );

        // 不正行はエラーになり、有効な前後の行の口座は変わらない
        let (items, errors) =
            accounts_and_errors(&[SEP_TOKUTEI, BAD_SHARES_ROW, TEST_ROW_1, TEST_ROW_2]);
        assert_eq!(errors, 1);
        assert_eq!(
            items,
            [
                ("9931".to_string(), Some("特定".to_string())),
                ("9932".to_string(), Some("特定".to_string())),
            ]
        );

        // 口座名の取れない見出しは現在口座をリセットする
        let (items, errors) =
            accounts_and_errors(&["■特定口座", HEADER, TEST_ROW_1, "■メモ", HEADER, TEST_ROW_2]);
        assert_eq!(errors, 0);
        assert_eq!(
            items,
            [
                ("9931".to_string(), Some("特定".to_string())),
                ("9932".to_string(), None),
            ]
        );

        // 見出し直後の列見出し繰り返しは取り込み行にならない
        let (items, errors) = accounts_and_errors(&["■特定口座", HEADER, HEADER, TEST_ROW_1]);
        assert_eq!(errors, 0);
        assert_eq!(items, [("9931".to_string(), Some("特定".to_string())),]);
    }
}
