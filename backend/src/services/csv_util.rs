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

/// レコードから指定列の値を取得（列が存在しない場合は空文字）
#[cfg(test)]
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

/// 行マップから指定列の値を取得（列が存在しない場合は空文字）
pub fn get_row_cell<'a>(row: &'a HashMap<String, String>, name: &str) -> &'a str {
    row.get(name).map(String::as_str).unwrap_or("")
}

/// オプション文字列フィールドを取得（空の場合は空文字列）
#[cfg(test)]
pub fn parse_optional_string(
    record: &csv::StringRecord,
    header_map: &HashMap<String, usize>,
    col: &str,
) -> String {
    get_cell(record, header_map, col).to_string()
}

/// 行マップからオプション文字列フィールドを取得（空の場合は空文字列）
pub fn parse_optional_string_row(row: &HashMap<String, String>, col: &str) -> String {
    get_row_cell(row, col).to_string()
}

/// 必須文字列フィールドを取得（空の場合はエラー）
#[cfg(test)]
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

/// 行マップから必須文字列フィールドを取得（空の場合はエラー）
pub fn parse_required_string_row(
    row: &HashMap<String, String>,
    col: &str,
    row_num: usize,
) -> Result<String, CsvRowError> {
    let val = get_row_cell(row, col);
    if val.trim().is_empty() {
        return Err(CsvRowError {
            row: row_num,
            message: format!("必須列 '{}' が空または存在しません", col),
        });
    }
    Ok(val.to_string())
}

/// 必須数値フィールドを取得（空またはパース失敗でエラー）
#[cfg(test)]
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

/// 行マップから必須数値フィールドを取得（空またはパース失敗でエラー）
pub fn parse_required_number_row(
    row: &HashMap<String, String>,
    col: &str,
    row_num: usize,
) -> Result<Decimal, CsvRowError> {
    let raw = get_row_cell(row, col);
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
#[cfg(test)]
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

/// 行マップから必須日付フィールドを取得（パース失敗でエラー）
pub fn parse_required_date_row(
    row: &HashMap<String, String>,
    col: &str,
    row_num: usize,
) -> Result<NaiveDate, CsvRowError> {
    let raw = get_row_cell(row, col);
    parse_date(raw).map_err(|e| CsvRowError {
        row: row_num,
        message: format!("{}: {}", col, e),
    })
}
#[cfg(test)]
mod tests {
    use shared::{normalize::normalize_security_name, tax::compute_taxes};
    use {super::*, rust_decimal_macros::dec};
    fn time_n(label: &str, n: usize, mut f: impl FnMut()) {
        let start = std::time::Instant::now();
        for _ in 0..n {
            f();
        }
        println!(
            "[timing] {} × {}回: {:.2}ms",
            label,
            n,
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
    fn make_header_map(cols: &[&str]) -> HashMap<String, usize> {
        cols.iter()
            .enumerate()
            .map(|(i, col)| ((*col).to_string(), i))
            .collect()
    }
    #[test]
    fn test_parse_utilities() {
        for (input, expected) in [
            ("1,234", dec!(1234)),
            ("500", dec!(500)),
            ("(500)", dec!(-500)),
            ("", Decimal::ZERO),
            ("-", Decimal::ZERO),
        ] {
            assert_eq!(parse_number(input).unwrap(), expected);
        }
        for input in ["(123", "123)", "(", ")"] {
            assert!(parse_number(input).is_err());
        }
        let (record, header_map) = (
            csv::StringRecord::from(vec!["", "value"]),
            make_header_map(&["empty", "filled"]),
        );
        for (col, expected) in [("empty", ""), ("filled", "value"), ("missing", "")] {
            assert_eq!(parse_optional_string(&record, &header_map, col), expected);
        }
        let expected_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
        for input in ["2024/01/15", "2024-01-15"] {
            assert_eq!(parse_date(input).unwrap(), expected_date);
        }
        for (account, realized_pnl, expected_taxes, expected_after) in [
            ("特定", dec!(10000), dec!(2031), dec!(7969)),
            ("特定", dec!(-5000), Decimal::ZERO, dec!(-5000)),
            ("NISA", dec!(10000), Decimal::ZERO, dec!(10000)),
        ] {
            assert_eq!(
                compute_taxes(account, realized_pnl),
                (expected_taxes, expected_after)
            );
        }
        use encoding_rs::SHIFT_JIS;
        assert_eq!(decode_bytes("テスト".as_bytes()), "テスト");
        assert_eq!(
            decode_bytes(b"\xEF\xBB\xBF\xE3\x83\x86\xE3\x82\xB9\xE3\x83\x88"),
            "テスト"
        );
        let (bytes, _, _) = SHIFT_JIS.encode("テスト");
        assert_eq!(decode_bytes(&bytes), "テスト");
        let (record, header_map) = (
            csv::StringRecord::from(vec!["value"]),
            make_header_map(&["present"]),
        );
        assert_eq!(get_cell(&record, &header_map, "present"), "value");
        assert_eq!(get_cell(&record, &header_map, "missing"), "");
        for (input, expected) in [
            ("K D D I", "KDDI"),
            ("I N P E X", "INPEX"),
            ("eMAXIS Slim 全世界株式", "eMAXIS Slim 全世界株式"),
            ("任天堂", "任天堂"),
            ("  KDDI  ", "KDDI"),
            ("", ""),
        ] {
            assert_eq!(normalize_security_name(input), expected);
        }
        let (record, header_map) = (
            csv::StringRecord::from(vec!["", "value"]),
            make_header_map(&["col_a", "col_b"]),
        );
        let err = parse_required_string(&record, &header_map, "col_a", 3).unwrap_err();
        assert_eq!((err.row, err.message.contains("col_a")), (3, true));
        assert_eq!(
            parse_required_string(&record, &header_map, "col_b", 1).unwrap(),
            "value"
        );
        let (record, hm) = (
            csv::StringRecord::from(vec!["abc", "1,234", ""]),
            make_header_map(&["invalid", "valid", "empty"]),
        );
        assert_eq!(
            parse_required_number(&record, &hm, "invalid", 5)
                .unwrap_err()
                .row,
            5
        );
        assert_eq!(
            parse_required_number(&record, &hm, "valid", 1).unwrap(),
            dec!(1234)
        );
        let err = parse_required_number(&record, &hm, "empty", 7).unwrap_err();
        assert_eq!((err.row, err.message.contains("empty")), (7, true));
        let (record, hm) = (
            csv::StringRecord::from(vec!["not-a-date", "2024/03/01", "2024/13/40"]),
            make_header_map(&["invalid", "valid", "out_of_range"]),
        );
        assert_eq!(
            parse_required_date(&record, &hm, "invalid", 2)
                .unwrap_err()
                .row,
            2
        );
        assert_eq!(
            parse_required_date(&record, &hm, "valid", 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 3, 1).unwrap()
        );
        let err = parse_required_date(&record, &hm, "out_of_range", 9).unwrap_err();
        assert_eq!((err.row, err.message.contains("out_of_range")), (9, true));
    }
    fn make_row(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn test_decode_bytes() {
        use encoding_rs::SHIFT_JIS;
        // UTF-8 バイト列はそのまま返る
        assert_eq!(decode_bytes("テスト".as_bytes()), "テスト");
        // BOM 付き UTF-8 は BOM が除去される
        assert_eq!(
            decode_bytes(b"\xEF\xBB\xBF\xE3\x83\x86\xE3\x82\xB9\xE3\x83\x88"),
            "テスト"
        );
        // Shift-JIS バイト列は UTF-8 文字列に変換される
        let (bytes, _, _) = SHIFT_JIS.encode("テスト");
        assert_eq!(decode_bytes(&bytes), "テスト");
    }

    #[test]
    fn test_normalize_security_name() {
        // 通常の文字列は trim されて返る
        assert_eq!(normalize_security_name("  任天堂  "), "任天堂");
        // 全角1文字ずつのスペース区切りは結合され、全角英数は半角に正規化される
        assert_eq!(normalize_security_name("Ａ Ｂ Ｃ"), "ABC");
        // 全角カナ1文字ずつのスペース区切りは結合される
        assert_eq!(normalize_security_name("ト ヨ タ"), "トヨタ");
        // 複数文字トークンが含まれる場合はそのまま
        assert_eq!(normalize_security_name("トヨタ 自動車"), "トヨタ 自動車");
    }

    #[test]
    fn test_parse_optional_string_row() {
        let row = make_row(&[("名前", "テスト")]);
        // 存在するカラムは値を返す
        assert_eq!(parse_optional_string_row(&row, "名前"), "テスト");
        // 存在しないカラムは空文字を返す
        assert_eq!(parse_optional_string_row(&row, "missing"), "");
    }

    #[test]
    fn test_parse_required_string_row() {
        let row = make_row(&[("名前", "テスト"), ("空欄", "")]);
        // 存在するカラムで値あり → Ok(値)
        assert_eq!(
            parse_required_string_row(&row, "名前", 1).unwrap(),
            "テスト"
        );
        // 空のカラム → Err(CsvRowError)
        let err = parse_required_string_row(&row, "空欄", 2).unwrap_err();
        assert_eq!((err.row, err.message.contains("空欄")), (2, true));
        // 存在しないカラム → Err(CsvRowError)
        let err = parse_required_string_row(&row, "missing", 3).unwrap_err();
        assert_eq!((err.row, err.message.contains("missing")), (3, true));
    }

    #[test]
    fn test_parse_required_number_row() {
        let row = make_row(&[("数値", "1,234"), ("空欄", ""), ("不正", "abc")]);
        // 有効な数値 "1,234" → Ok(dec!(1234))
        assert_eq!(
            parse_required_number_row(&row, "数値", 1).unwrap(),
            dec!(1234)
        );
        // 空文字 → Err
        let err = parse_required_number_row(&row, "空欄", 2).unwrap_err();
        assert_eq!((err.row, err.message.contains("空欄")), (2, true));
        // 不正な文字列 "abc" → Err
        assert_eq!(
            parse_required_number_row(&row, "不正", 3).unwrap_err().row,
            3
        );
    }

    #[test]
    fn test_parse_required_date_row() {
        let row = make_row(&[("日付", "2024/01/15"), ("不正", "not-a-date")]);
        // 有効な日付 "2024/01/15" → Ok
        assert_eq!(
            parse_required_date_row(&row, "日付", 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()
        );
        // 不正な文字列 "not-a-date" → Err
        let err = parse_required_date_row(&row, "不正", 2).unwrap_err();
        assert_eq!((err.row, err.message.contains("不正")), (2, true));
    }

    #[test]
    #[ignore = "タイミング計測専用。cargo test --lib -- timing_csv_util --ignored --nocapture で実行"]
    fn timing_csv_util() {
        use encoding_rs::SHIFT_JIS;
        const ROWS: usize = 1_000;
        let csv_utf8: String = {
            let mut s = String::from("日付,銘柄コード,金額\n");
            for i in 0..ROWS {
                s.push_str(&format!(
                    "2024/{:02}/{:02},1234,{}\n",
                    (i % 12) + 1,
                    (i % 28) + 1,
                    i * 100
                ));
            }
            s
        };
        let bytes_utf8 = csv_utf8.as_bytes();
        let (bytes_sjis_cow, _, _) = SHIFT_JIS.encode(&csv_utf8);
        let bytes_sjis = bytes_sjis_cow.into_owned();
        time_n(&format!("decode_bytes (UTF-8, {}行)", ROWS), 10, || {
            std::hint::black_box(decode_bytes(std::hint::black_box(bytes_utf8)));
        });
        time_n(
            &format!("decode_bytes (Shift-JIS フォールバック, {}行)", ROWS),
            10,
            || {
                std::hint::black_box(decode_bytes(std::hint::black_box(bytes_sjis.as_slice())));
            },
        );
        let samples = ["1,234,567", "0", "(1,000)", "3.14159", "-"];
        time_n("parse_number", 10_000, || {
            for s in &samples {
                let _ = std::hint::black_box(parse_number(std::hint::black_box(s)));
            }
        });
        let date_samples = ["2024/03/01", "2024-12-31", "2023/01/01"];
        time_n("parse_date", 10_000, || {
            for s in &date_samples {
                let _ = std::hint::black_box(parse_date(std::hint::black_box(s)));
            }
        });
    }

    #[test]
    fn test_compute_taxes() {
        assert_eq!(
            compute_taxes("特定口座", dec!(10000)),
            (dec!(2031), dec!(7969))
        );
        assert_eq!(compute_taxes("特定口座", dec!(0)), (dec!(0), dec!(0)));
        assert_eq!(compute_taxes("特定口座", dec!(-500)), (dec!(0), dec!(-500)));
        assert_eq!(
            compute_taxes("NISA口座", dec!(10000)),
            (dec!(0), dec!(10000))
        );
        assert_eq!(
            compute_taxes("一般口座", dec!(10000)),
            (dec!(0), dec!(10000))
        );
    }

    fn add_thousands_separators(unsigned: &str) -> String {
        let (integer, fraction) = unsigned
            .split_once('.')
            .map_or((unsigned, None), |(i, f)| (i, Some(f)));
        let mut grouped = String::new();
        for (index, c) in integer.chars().enumerate() {
            if index > 0 && (integer.len() - index) % 3 == 0 {
                grouped.push(',');
            }
            grouped.push(c);
        }
        match fraction {
            Some(fraction) => format!("{grouped}.{fraction}"),
            None => grouped,
        }
    }

    proptest::proptest! {
        #[test]
        fn prop_parse_number_roundtrips_formatted_values(
            mantissa in -9_999_999_999_999i64..9_999_999_999_999i64,
            scale in 0u32..=4u32,
            with_commas in proptest::bool::ANY,
            with_parens in proptest::bool::ANY,
        ) {
            let value = Decimal::new(mantissa, scale);
            let plain = value.to_string();
            let negative = plain.starts_with('-');
            let unsigned = plain.strip_prefix('-').unwrap_or(&plain);
            let body = if with_commas {
                add_thousands_separators(unsigned)
            } else {
                unsigned.to_string()
            };
            let input = match (negative, with_parens) {
                (true, true) => format!("({body})"),
                (true, false) => format!("-{body}"),
                (false, true) => format!("({body})"),
                (false, false) => body,
            };
            let expected = match (negative, with_parens) {
                (true, _) | (false, true) => -value.abs(),
                _ => value,
            };
            proptest::prop_assert_eq!(
                parse_number(&input).unwrap(),
                expected.normalize(),
                "input={}",
                input
            );
        }

        #[test]
        fn prop_parse_number_matches_naive_model_or_errors(input in ".*") {
            let naive = {
                let trimmed = input.trim();
                let inner = trimmed
                    .strip_prefix('(')
                    .and_then(|s| s.strip_suffix(')'));
                let (negative, body) = inner.map_or((false, trimmed), |s| (true, s));
                let sanitized = body.replace(',', "");
                sanitized
                    .parse::<Decimal>()
                    .ok()
                    .map(|v| if negative { -v } else { v })
            };
            let result = parse_number(&input);
            match (result, naive) {
                (Ok(actual), Some(expected)) => {
                    proptest::prop_assert_eq!(actual, expected.normalize(), "input={}", input);
                }
                // "-" と空文字だけは実装が 0 を返すが素朴モデルは失敗する
                (Ok(actual), None) => {
                    let trimmed = input.trim();
                    proptest::prop_assert!(
                        trimmed.is_empty() || trimmed == "-",
                        "input={} が Ok なのに素朴モデルは失敗した: actual={}",
                        input,
                        actual
                    );
                }
                (Err(_), Some(_)) => {
                    // Decimal::from_str の受理域は .parse::<Decimal>() と同一のはず
                    panic!("input={input} が Err なのに素朴モデルは成功した");
                }
                (Err(_), None) => {}
            }
        }

        #[test]
        fn prop_compute_taxes_invariants(
            account in "[特定一般NISA口座 ]{0,12}",
            mantissa in -9_999_999_999_999i64..9_999_999_999_999i64,
            scale in 0u32..=3u32,
        ) {
            let pnl = Decimal::new(mantissa, scale);
            let (taxes, after_tax) = compute_taxes(&account, pnl);
            proptest::prop_assert!(taxes >= Decimal::ZERO);
            proptest::prop_assert_eq!(taxes + after_tax, pnl);
            if !account.contains("特定") || pnl <= Decimal::ZERO {
                proptest::prop_assert_eq!(taxes, Decimal::ZERO);
            } else {
                proptest::prop_assert_eq!(taxes, (pnl * dec!(0.20315)).floor());
            }
        }

        #[test]
        fn prop_parse_date_accepts_valid_and_rejects_invalid(
            year in 1970i32..2100i32,
            month in 1u32..=12u32,
            day in 1u32..=31u32,
            slash in proptest::bool::ANY,
        ) {
            let separator = if slash { "/" } else { "-" };
            let input = format!("{year:04}{separator}{month:02}{separator}{day:02}");
            let days_in_month = match month {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
                2 => 28,
                _ => unreachable!(),
            };
            if day <= days_in_month {
                proptest::prop_assert_eq!(
                    parse_date(&input).unwrap(),
                    NaiveDate::from_ymd_opt(year, month, day).unwrap(),
                    "input={}",
                    input
                );
            } else {
                proptest::prop_assert!(parse_date(&input).is_err(), "input={}", input);
            }
        }

        #[test]
        fn prop_decode_bytes_utf8_roundtrip(input in ".*") {
            let expected = input.strip_prefix('\u{FEFF}').unwrap_or(&input).to_string();
            proptest::prop_assert_eq!(decode_bytes(input.as_bytes()), expected);
            let with_bom = format!("\u{FEFF}{input}");
            proptest::prop_assert_eq!(decode_bytes(with_bom.as_bytes()), input);
        }
    }
}
