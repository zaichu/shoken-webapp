use crate::models::csv_import::CsvRowError;
use chrono::NaiveDate;
use encoding_rs::SHIFT_JIS;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;

/// UTF-8 デコードを試み、失敗時は Shift-JIS にフォールバック
pub fn decode_bytes(bytes: &[u8]) -> String {
    // std::str::from_utf8 はアロケーションなしで UTF-8 妥当性を検証する
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.strip_prefix('\u{FEFF}').unwrap_or(s).to_string();
    }
    // UTF-8 でない場合は Shift-JIS にフォールバック
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

/// 銘柄名を正規化する
///
/// 証券会社のCSVによっては "K D D I" のように各文字間にスペースが挿入される場合がある。
/// すべてのトークンが1文字の場合はスペースを除去して結合する（例: "K D D I" → "KDDI"）。
/// 複数文字のトークンが含まれる場合（例: "eMAXIS Slim 全世界株式"）はそのまま返す。
pub fn normalize_security_name(name: &str) -> String {
    let tokens: Vec<&str> = name.split_whitespace().collect();
    if tokens.len() > 1 && tokens.iter().all(|t| t.chars().count() == 1) {
        tokens.join("")
    } else {
        name.trim().to_string()
    }
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
#[cfg(test)] #[rustfmt::skip] mod tests {
    use {super::*, rust_decimal_macros::dec};
    fn time_n(label: &str, n: usize, mut f: impl FnMut()) { let start = std::time::Instant::now(); for _ in 0..n { f(); } println!("[timing] {} × {}回: {:.2}ms", label, n, start.elapsed().as_secs_f64() * 1000.0); } fn make_header_map(cols: &[&str]) -> HashMap<String, usize> { cols.iter().enumerate().map(|(i, col)| ((*col).to_string(), i)).collect() }
    #[test] fn test_parse_utilities() { for (input, expected) in [("1,234", dec!(1234)), ("500", dec!(500)), ("(500)", dec!(-500)), ("", Decimal::ZERO), ("-", Decimal::ZERO)] { assert_eq!(parse_number(input).unwrap(), expected); } let (record, header_map) = (csv::StringRecord::from(vec!["", "value"]), make_header_map(&["empty", "filled"])); for (col, expected) in [("empty", ""), ("filled", "value"), ("missing", "")] { assert_eq!(parse_optional_string(&record, &header_map, col), expected); } let expected_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(); for input in ["2024/01/15", "2024-01-15"] { assert_eq!(parse_date(input).unwrap(), expected_date); } for (account, realized_pnl, expected_taxes, expected_after) in [("特定", dec!(10000), dec!(2031), dec!(7969)), ("特定", dec!(-5000), Decimal::ZERO, dec!(-5000)), ("NISA", dec!(10000), Decimal::ZERO, dec!(10000))] { assert_eq!(compute_taxes(account, realized_pnl), (expected_taxes, expected_after)); } use encoding_rs::SHIFT_JIS; assert_eq!(decode_bytes("テスト".as_bytes()), "テスト"); assert_eq!(decode_bytes(b"\xEF\xBB\xBF\xE3\x83\x86\xE3\x82\xB9\xE3\x83\x88"), "テスト"); let (bytes, _, _) = SHIFT_JIS.encode("テスト"); assert_eq!(decode_bytes(&bytes), "テスト"); let (record, header_map) = (csv::StringRecord::from(vec!["value"]), make_header_map(&["present"])); assert_eq!(get_cell(&record, &header_map, "present"), "value"); assert_eq!(get_cell(&record, &header_map, "missing"), ""); for (input, expected) in [("K D D I", "KDDI"), ("I N P E X", "INPEX"), ("eMAXIS Slim 全世界株式", "eMAXIS Slim 全世界株式"), ("任天堂", "任天堂"), ("  KDDI  ", "KDDI"), ("", "")] { assert_eq!(normalize_security_name(input), expected); } let (record, header_map) = (csv::StringRecord::from(vec!["", "value"]), make_header_map(&["col_a", "col_b"])); let err = parse_required_string(&record, &header_map, "col_a", 3).unwrap_err(); assert_eq!((err.row, err.message.contains("col_a")), (3, true)); assert_eq!(parse_required_string(&record, &header_map, "col_b", 1).unwrap(), "value"); let (record, hm) = (csv::StringRecord::from(vec!["abc", "1,234", ""]), make_header_map(&["invalid", "valid", "empty"])); assert_eq!(parse_required_number(&record, &hm, "invalid", 5).unwrap_err().row, 5); assert_eq!(parse_required_number(&record, &hm, "valid", 1).unwrap(), dec!(1234)); let err = parse_required_number(&record, &hm, "empty", 7).unwrap_err(); assert_eq!((err.row, err.message.contains("empty")), (7, true)); let (record, hm) = (csv::StringRecord::from(vec!["not-a-date", "2024/03/01", "2024/13/40"]), make_header_map(&["invalid", "valid", "out_of_range"])); assert_eq!(parse_required_date(&record, &hm, "invalid", 2).unwrap_err().row, 2); assert_eq!(parse_required_date(&record, &hm, "valid", 1).unwrap(), NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()); let err = parse_required_date(&record, &hm, "out_of_range", 9).unwrap_err(); assert_eq!((err.row, err.message.contains("out_of_range")), (9, true)); }
    #[test] #[ignore = "タイミング計測専用。cargo test --lib -- timing_csv_util --ignored --nocapture で実行"] fn timing_csv_util() { use encoding_rs::SHIFT_JIS; const ROWS: usize = 1_000; let csv_utf8: String = { let mut s = String::from("日付,銘柄コード,金額\n"); for i in 0..ROWS { s.push_str(&format!("2024/{:02}/{:02},1234,{}\n", (i % 12) + 1, (i % 28) + 1, i * 100)); } s }; let bytes_utf8 = csv_utf8.as_bytes(); let (bytes_sjis_cow, _, _) = SHIFT_JIS.encode(&csv_utf8); let bytes_sjis = bytes_sjis_cow.into_owned(); time_n(&format!("decode_bytes (UTF-8, {}行)", ROWS), 10, || { std::hint::black_box(decode_bytes(std::hint::black_box(bytes_utf8))); }); time_n(&format!("decode_bytes (Shift-JIS フォールバック, {}行)", ROWS), 10, || { std::hint::black_box(decode_bytes(std::hint::black_box(bytes_sjis.as_slice()))); }); let samples = ["1,234,567", "0", "(1,000)", "3.14159", "-"]; time_n("parse_number", 10_000, || { for s in &samples { let _ = std::hint::black_box(parse_number(std::hint::black_box(s))); } }); let date_samples = ["2024/03/01", "2024-12-31", "2023/01/01"]; time_n("parse_date", 10_000, || { for s in &date_samples { let _ = std::hint::black_box(parse_date(std::hint::black_box(s))); } }); }
}
