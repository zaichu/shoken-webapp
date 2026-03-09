use crate::models::csv_import::CsvRowError;
use chrono::NaiveDate;
use encoding_rs::{SHIFT_JIS, UTF_8};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

/// UTF-8 デコードを試み、失敗時は Shift-JIS にフォールバック
pub fn decode_bytes(bytes: &[u8]) -> String {
    // まず UTF-8 を試みる
    let (result, _, had_errors) = UTF_8.decode(bytes);
    if !had_errors {
        return result.into_owned();
    }
    // Shift-JIS にフォールバック
    let (result, _, _) = SHIFT_JIS.decode(bytes);
    result.into_owned()
}

/// 数値文字列をパース（カンマ区切り・括弧マイナス対応）
/// 例: "1,234" → 1234、"(500)" → -500、"-" → 0（値なし）
pub fn parse_number(s: &str) -> Result<Decimal, String> {
    let s = s.trim();
    if s.is_empty() || s == "-" {
        return Ok(Decimal::ZERO);
    }
    // 括弧表記はマイナス
    let (negative, s) = if s.starts_with('(') && s.ends_with(')') {
        (true, &s[1..s.len() - 1])
    } else {
        (false, s)
    };
    let s = s.replace(',', "");
    let value =
        Decimal::from_str(&s).map_err(|_| format!("数値のパースに失敗しました: '{}'", s))?;
    // trailing zero を除去して表記差（"1" vs "1.0"）がハッシュに影響しないよう正規化する
    Ok(if negative { -value } else { value }.normalize())
}

/// 日付文字列をパース（"YYYY/MM/DD" または "YYYY-MM-DD"）
pub fn parse_date(s: &str) -> Result<NaiveDate, String> {
    let s = s.trim();
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y/%m/%d") {
        return Ok(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(d);
    }
    Err(format!("日付のパースに失敗しました: '{}'", s))
}

/// 税金を計算する（特定口座かつ利益がある場合のみ）
pub fn compute_taxes(account: &str, realized_pnl: Decimal) -> (Decimal, Decimal) {
    let tax_rate = Decimal::from_str("0.20315").unwrap();
    if account.contains("特定") && realized_pnl > Decimal::ZERO {
        let taxes = (realized_pnl * tax_rate).floor();
        let after_tax = realized_pnl - taxes;
        (taxes, after_tax)
    } else {
        (Decimal::ZERO, realized_pnl)
    }
}

/// レコードから指定列の値を取得（列が存在しない場合は空文字）
pub fn get_cell<'a>(
    record: &'a csv::StringRecord,
    header_map: &HashMap<String, usize>,
    name: &str,
) -> &'a str {
    header_map
        .get(name)
        .and_then(|&i| record.get(i))
        .unwrap_or("")
}

/// オプション文字列フィールドを取得（空の場合は空文字列）
pub fn parse_optional_string(
    record: &csv::StringRecord,
    header_map: &HashMap<String, usize>,
    col: &str,
) -> String {
    get_cell(record, header_map, col).to_string()
}

/// 必須文字列フィールドを取得（空の場合はエラー）
pub fn parse_required_string(
    record: &csv::StringRecord,
    header_map: &HashMap<String, usize>,
    col: &str,
    row_num: usize,
) -> Result<String, CsvRowError> {
    let val = get_cell(record, header_map, col);
    if val.trim().is_empty() {
        return Err(CsvRowError {
            row: row_num,
            message: format!("必須列 '{}' が空または存在しません", col),
        });
    }
    Ok(val.to_string())
}

/// 必須数値フィールドを取得（空またはパース失敗でエラー）
pub fn parse_required_number(
    record: &csv::StringRecord,
    header_map: &HashMap<String, usize>,
    col: &str,
    row_num: usize,
) -> Result<Decimal, CsvRowError> {
    let raw = get_cell(record, header_map, col);
    if raw.trim().is_empty() {
        return Err(CsvRowError {
            row: row_num,
            message: format!("必須列 '{}' が空または存在しません", col),
        });
    }
    parse_number(raw).map_err(|e| CsvRowError {
        row: row_num,
        message: format!("{}: {}", col, e),
    })
}

/// 必須日付フィールドを取得（パース失敗でエラー）
pub fn parse_required_date(
    record: &csv::StringRecord,
    header_map: &HashMap<String, usize>,
    col: &str,
    row_num: usize,
) -> Result<NaiveDate, CsvRowError> {
    let raw = get_cell(record, header_map, col);
    parse_date(raw).map_err(|e| CsvRowError {
        row: row_num,
        message: format!("{}: {}", col, e),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_number_normal() {
        assert_eq!(parse_number("1,234").unwrap(), dec!(1234));
        assert_eq!(parse_number("500").unwrap(), dec!(500));
        assert_eq!(parse_number("(500)").unwrap(), dec!(-500));
        assert_eq!(parse_number("").unwrap(), Decimal::ZERO);
        // ハイフン単独は「値なし」として 0
        assert_eq!(parse_number("-").unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_parse_optional_string() {
        let record = csv::StringRecord::from(vec!["", "value"]);
        let mut header_map = HashMap::new();
        header_map.insert("empty".to_string(), 0);
        header_map.insert("filled".to_string(), 1);

        assert_eq!(parse_optional_string(&record, &header_map, "empty"), "");
        assert_eq!(
            parse_optional_string(&record, &header_map, "filled"),
            "value"
        );
        // 存在しない列は空文字
        assert_eq!(parse_optional_string(&record, &header_map, "missing"), "");
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(
            parse_date("2024/01/15").unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        );
        assert_eq!(
            parse_date("2024-01-15").unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        );
    }

    #[test]
    fn test_compute_taxes_tokutei_profit() {
        let (taxes, after) = compute_taxes("特定", dec!(10000));
        assert_eq!(taxes, dec!(2031)); // floor(10000 * 0.20315)
        assert_eq!(after, dec!(7969));
    }

    #[test]
    fn test_compute_taxes_loss() {
        let (taxes, after) = compute_taxes("特定", dec!(-5000));
        assert_eq!(taxes, Decimal::ZERO);
        assert_eq!(after, dec!(-5000));
    }

    #[test]
    fn test_compute_taxes_nisa() {
        let (taxes, after) = compute_taxes("NISA", dec!(10000));
        assert_eq!(taxes, Decimal::ZERO);
        assert_eq!(after, dec!(10000));
    }

    #[test]
    fn test_decode_bytes_utf8() {
        let input = "テスト".as_bytes();
        assert_eq!(decode_bytes(input), "テスト");
    }

    #[test]
    fn test_parse_required_string_empty() {
        let record = csv::StringRecord::from(vec!["", "value"]);
        let mut header_map = HashMap::new();
        header_map.insert("col_a".to_string(), 0);
        header_map.insert("col_b".to_string(), 1);

        let err = parse_required_string(&record, &header_map, "col_a", 3).unwrap_err();
        assert_eq!(err.row, 3);
        assert!(err.message.contains("col_a"));

        let ok = parse_required_string(&record, &header_map, "col_b", 1).unwrap();
        assert_eq!(ok, "value");
    }

    #[test]
    fn test_parse_required_number_invalid() {
        let record = csv::StringRecord::from(vec!["abc", "1,234"]);
        let mut header_map = HashMap::new();
        header_map.insert("bad".to_string(), 0);
        header_map.insert("good".to_string(), 1);

        let err = parse_required_number(&record, &header_map, "bad", 5).unwrap_err();
        assert_eq!(err.row, 5);

        let ok = parse_required_number(&record, &header_map, "good", 1).unwrap();
        assert_eq!(ok, dec!(1234));
    }

    #[test]
    fn test_parse_required_date_invalid() {
        let record = csv::StringRecord::from(vec!["not-a-date", "2024/03/01"]);
        let mut header_map = HashMap::new();
        header_map.insert("bad".to_string(), 0);
        header_map.insert("good".to_string(), 1);

        let err = parse_required_date(&record, &header_map, "bad", 2).unwrap_err();
        assert_eq!(err.row, 2);

        let ok = parse_required_date(&record, &header_map, "good", 1).unwrap();
        assert_eq!(ok, NaiveDate::from_ymd_opt(2024, 3, 1).unwrap());
    }
}
