use crate::models::asset_balance::CreateAssetBalanceRequest;
use crate::models::csv_import::CsvRowError;
use crate::services::csv_import::parse_csv;
use crate::services::csv_parse::{decode_bytes, parse_number, parse_optional_string};
use csv::StringRecord;
use rust_decimal::Decimal;
use std::collections::HashMap;

/// 保有銘柄 CSV bytes をデコード・ヘッダースキップ・パース・フィルタして返す
/// preview / upload の共通前処理経路
pub(crate) fn parse_asset_balance_csv(
    bytes: &[u8],
) -> Result<(Vec<CreateAssetBalanceRequest>, Vec<CsvRowError>), crate::errors::ApiError> {
    let content = decode_bytes(bytes);
    let stripped = strip_asset_balance_csv_metadata(&content);
    let filtered = strip_account_summary_rows(&stripped);
    let (items, errors) = parse_csv(filtered.as_bytes(), parse_asset_balance_row)?;
    let items = items
        .into_iter()
        .filter(|i| !i.security_code.is_empty())
        .collect();
    Ok((items, errors))
}

/// 保有銘柄 CSV の先頭6行（メタデータ）をスキップした文字列を返す
fn strip_asset_balance_csv_metadata(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() > 6 {
        lines[6..].join("\n")
    } else {
        String::new()
    }
}

/// 保有銘柄 CSV に混ざる「特定口座合計」などの口座集計行を除外する
///
/// 先頭フィールド（銘柄コード）が空の行かつ「口座合計」を含む行のみ除外する。
/// 銘柄名に「口座合計」を含む銘柄を誤除外しないよう、先頭が空であることを条件とする。
fn strip_account_summary_rows(content: &str) -> String {
    content
        .lines()
        .filter(|line| !(line.starts_with(',') && line.contains("口座合計")))
        .collect::<Vec<_>>()
        .join("\n")
}

/// 保有銘柄 CSV の1行をパースして CreateAssetBalanceRequest に変換
///
/// 数値パース失敗は CsvRowError として返す。
/// ただし以下の列は "-" / 空欄が仕様上ありうるため 0.0 フォールバックを維持する:
///   - 執行中: 執行中の注文がなければ "-" または空欄
///   - 現在値（前日比）: 変動なし時は 0 または "-"
///   - 評価損益（%）: NISA 等で表示されない場合に "-"
pub(crate) fn parse_asset_balance_row(
    record: &StringRecord,
    header_map: &HashMap<String, usize>,
    row_num: usize,
) -> Result<CreateAssetBalanceRequest, CsvRowError> {
    // 必須数値列: 空欄・"-" もエラー。パース失敗も CsvRowError に変換する
    let num = |col: &str| {
        let raw = parse_optional_string(record, header_map, col);
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == "-" {
            return Err(CsvRowError {
                row: row_num,
                message: format!("必須列 '{}' が空または値なし", col),
            });
        }
        parse_number(trimmed).map_err(|e| CsvRowError {
            row: row_num,
            message: format!("{}: {}", col, e),
        })
    };

    let security_code = parse_optional_string(record, header_map, "銘柄コード").replace('"', "");
    Ok(CreateAssetBalanceRequest {
        security_code,
        security_name: parse_optional_string(record, header_map, "銘柄名"),
        shares: num("保有数量［株］")?,
        // 執行中は "-" / 空欄が仕様上ありうるため 0.0 フォールバック
        executing_shares: parse_number(&parse_optional_string(record, header_map, "執行中［株］"))
            .unwrap_or(Decimal::ZERO),
        average_purchase_price: num("平均取得価額［円］")?,
        total_purchase_amount: num("取得総額［円］")?,
        current_price: num("現在値［円］")?,
        // 前日比は変動なし時に 0 または "-" が仕様上ありうるため 0.0 フォールバック
        daily_change: parse_number(&parse_optional_string(
            record,
            header_map,
            "現在値（前日比）［円］",
        ))
        .unwrap_or(Decimal::ZERO),
        market_value: num("時価評価額［円］")?,
        // 評価損益は NISA 等で表示されない場合に "-" が仕様上ありうるため 0.0 フォールバック
        profit_loss_rate: parse_number(&parse_optional_string(
            record,
            header_map,
            "評価損益［％］",
        ))
        .unwrap_or(Decimal::ZERO),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::csv_import::parse_csv;
    use rust_decimal_macros::dec;

    fn make_csv(header: &str, row: &str) -> String {
        format!("{}\n{}\n", header, row)
    }

    const HEADER: &str =
        "銘柄コード,銘柄名,保有数量［株］,執行中［株］,平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";
    const ASSET_BALANCE_CSV_HEADER: &str =
        "銘柄コード,銘柄名,保有数量［株］,執行中［株］,(内訳　通常数量[株]),(内訳　積立数量[株]),平均取得価額［円］,取得総額［円］,現在値［円］,現在値（前日比）［円］,時価評価額［円］,評価損益［％］";

    #[test]
    fn test_parse_asset_balance_row_ok() {
        let csv = make_csv(
            HEADER,
            "1234,テスト株式会社,100,-,1500,150000,1600,10,160000,6.67",
        );
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].security_code, "1234");
        assert_eq!(items[0].shares, dec!(100));
        assert_eq!(items[0].executing_shares, Decimal::ZERO); // "-" → 0
        assert_eq!(items[0].average_purchase_price, dec!(1500));
        assert_eq!(items[0].current_price, dec!(1600));
    }

    #[test]
    fn test_parse_asset_balance_row_invalid_shares_is_error() {
        // 保有数量が不正値（N/A など）→ サイレント 0 埋めせず CsvRowError を返す
        let csv = make_csv(HEADER, "1234,テスト,N/A,-,1500,150000,1600,0,160000,0");
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(items.is_empty(), "不正行はアイテムに含まれてはいけない");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("保有数量"));
    }

    #[test]
    fn test_parse_asset_balance_row_dash_in_required_col_is_error() {
        // 保有数量が "-"（値なし）→ 必須列なので CsvRowError
        let csv = make_csv(HEADER, "1234,テスト,-,-,1500,150000,1600,0,160000,0");
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(
            items.is_empty(),
            "必須列が '-' の行はアイテムに含まれてはいけない"
        );
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("保有数量"));
    }

    #[test]
    fn test_parse_asset_balance_row_invalid_price_is_error() {
        // 現在値が不正値（N/A など）→ CsvRowError
        let csv = make_csv(HEADER, "1234,テスト,100,-,1500,150000,N/A,0,160000,0");
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(items.is_empty());
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("現在値"));
    }

    #[test]
    fn test_parse_asset_balance_row_optional_zero_fields() {
        // 前日比・評価損益が "-" でも 0.0 として許容
        let csv = make_csv(HEADER, "5678,ファンド,50,-,2000,100000,2100,-,105000,-");
        let (items, errors) = parse_csv(csv.as_bytes(), parse_asset_balance_row).unwrap();
        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(items[0].daily_change, Decimal::ZERO);
        assert_eq!(items[0].profit_loss_rate, Decimal::ZERO);
    }

    #[test]
    fn test_parse_asset_balance_csv_skips_empty_rows() {
        let csv = [
            "■現在の評価額合計［円］,,\"3,588,300\"",
            "■評価損益合計,前日比［円］,\"59,700\"",
            ",前月比［円］,\"45,000\"",
            ",評価損益［円］,\"684,900\"",
            "■特定口座",
            "",
            ASSET_BALANCE_CSV_HEADER,
            "1234,テスト株式会社,100,0,100,0,1500,150000,1600,10,160000,6.67",
            ",,,,,,,,,,,",
            "5678,サンプル株式会社,200,0,200,0,1800,360000,1900,15,380000,5.56",
        ]
        .join("\n");

        let (items, errors) = parse_asset_balance_csv(csv.as_bytes()).unwrap();

        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].security_code, "1234");
        assert_eq!(items[1].security_code, "5678");
    }

    #[test]
    fn test_parse_asset_balance_csv_skips_account_summary_rows() {
        let csv = [
            "■現在の評価額合計［円］,,\"9,474,000\"",
            "■評価損益合計,前日比［円］,\"288,000\"",
            ",前月比［円］,\"-120,000\"",
            ",評価損益［円］,\"2,005,900\"",
            "■特定口座",
            "",
            ASSET_BALANCE_CSV_HEADER,
            "\"1605\",\"ＩＮＰＥＸ\",\"200\",\"0\",\"200\",\"0\",\"2,355.00\",\"471,000\",\"3,685.0\",\"65.0\",\"737,000\",\"56.47\"",
            "\"7974\",\"任天堂\",\"1,000\",\"0\",\"1,000\",\"0\",\"5,997.60\",\"5,997,600\",\"8,737.0\",\"223.0\",\"8,737,000\",\"45.67\"",
            ",,,,,,特定口座合計,\"11,245,249\",,,\"14,517,240\",\"29.09\"",
        ]
        .join("\n");

        let (items, errors) = parse_asset_balance_csv(csv.as_bytes()).unwrap();

        assert!(errors.is_empty(), "unexpected errors: {errors:?}");
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].security_code, "1605");
        assert_eq!(items[1].security_code, "7974");
    }
}
