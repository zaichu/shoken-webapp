use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use chrono::NaiveDate;
use encoding_rs::{SHIFT_JIS, UTF_8};
use std::collections::HashMap;

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
/// 例: "1,234" → 1234.0、"(500)" → -500.0、"-" → 0.0（値なし）
pub fn parse_number(s: &str) -> Result<f64, String> {
    let s = s.trim();
    if s.is_empty() || s == "-" {
        return Ok(0.0);
    }
    // 括弧表記はマイナス
    let (negative, s) = if s.starts_with('(') && s.ends_with(')') {
        (true, &s[1..s.len() - 1])
    } else {
        (false, s)
    };
    let s = s.replace(',', "");
    let value: f64 = s
        .parse()
        .map_err(|_| format!("数値のパースに失敗しました: '{}'", s))?;
    Ok(if negative { -value } else { value })
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
pub fn compute_taxes(account: &str, realized_pnl: f64) -> (f64, f64) {
    const TAX_RATE: f64 = 0.20315;
    if account.contains("特定") && realized_pnl > 0.0 {
        let taxes = (realized_pnl * TAX_RATE).floor();
        let after_tax = realized_pnl - taxes;
        (taxes, after_tax)
    } else {
        (0.0, realized_pnl)
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
    if val.is_empty() {
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
) -> Result<f64, CsvRowError> {
    let raw = get_cell(record, header_map, col);
    if raw.is_empty() {
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

/// CSV bytes をデコードして行ごとにパース
/// parse_row が Err を返した行はエラーとして収集し、items には含めない
pub fn parse_csv<T, F>(bytes: &[u8], parse_row: F) -> Result<(Vec<T>, Vec<CsvRowError>), ApiError>
where
    F: Fn(&csv::StringRecord, &HashMap<String, usize>, usize) -> Result<T, CsvRowError>,
{
    let content = decode_bytes(bytes);
    let mut reader = csv::Reader::from_reader(content.as_bytes());

    let header_map: HashMap<String, usize> = reader
        .headers()
        .map_err(|e| {
            ApiError::ValidationError(format!("CSVヘッダーの読み込みに失敗しました: {}", e))
        })?
        .iter()
        .enumerate()
        .map(|(i, h)| (h.trim().to_string(), i))
        .collect();

    let mut items = Vec::new();
    let mut errors = Vec::new();

    for (row_idx, result) in reader.records().enumerate() {
        let row_num = row_idx + 1;
        let record = match result {
            Ok(r) => r,
            Err(e) => {
                errors.push(CsvRowError {
                    row: row_num,
                    message: format!("CSV行の読み込みに失敗しました: {}", e),
                });
                continue;
            }
        };
        match parse_row(&record, &header_map, row_num) {
            Ok(item) => items.push(item),
            Err(e) => errors.push(e),
        }
    }

    Ok((items, errors))
}

/// CSV bytes をパースしてプレビュー情報を返す（DB 書き込みなし）
pub fn build_preview<T, F>(bytes: &[u8], parse_row: F) -> Result<CsvPreviewResponse, ApiError>
where
    T: serde::Serialize,
    F: Fn(&csv::StringRecord, &HashMap<String, usize>, usize) -> Result<T, CsvRowError>,
{
    let (items, errors) = parse_csv(bytes, parse_row)?;
    let rows = items
        .iter()
        .map(|item| serde_json::to_value(item).unwrap_or(serde_json::Value::Null))
        .collect();
    Ok(CsvPreviewResponse {
        total_rows: items.len() + errors.len(),
        valid_rows: items.len(),
        errors,
        rows,
    })
}

/// bulk_create 結果と行エラーから CsvUploadResponse を構築
pub fn finish_csv_upload(
    result: BulkCreateResponse,
    errors: Vec<CsvRowError>,
) -> CsvUploadResponse {
    CsvUploadResponse {
        inserted: result.inserted,
        skipped: result.skipped,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number_normal() {
        assert_eq!(parse_number("1,234").unwrap(), 1234.0);
        assert_eq!(parse_number("500").unwrap(), 500.0);
        assert_eq!(parse_number("(500)").unwrap(), -500.0);
        assert_eq!(parse_number("").unwrap(), 0.0);
        // ハイフン単独は「値なし」として 0.0
        assert_eq!(parse_number("-").unwrap(), 0.0);
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
        let (taxes, after) = compute_taxes("特定", 10000.0);
        assert_eq!(taxes, 2031.0); // floor(10000 * 0.20315)
        assert_eq!(after, 7969.0);
    }

    #[test]
    fn test_compute_taxes_loss() {
        let (taxes, after) = compute_taxes("特定", -5000.0);
        assert_eq!(taxes, 0.0);
        assert_eq!(after, -5000.0);
    }

    #[test]
    fn test_compute_taxes_nisa() {
        let (taxes, after) = compute_taxes("NISA", 10000.0);
        assert_eq!(taxes, 0.0);
        assert_eq!(after, 10000.0);
    }

    #[test]
    fn test_decode_bytes_utf8() {
        let input = "テスト".as_bytes();
        assert_eq!(decode_bytes(input), "テスト");
    }

    #[test]
    fn test_parse_csv_ok() {
        let csv = "col_a,col_b\nfoo,123\nbar,456\n";
        let (items, errors) = parse_csv::<String, _>(csv.as_bytes(), |record, header_map, _row| {
            let a = get_cell(record, header_map, "col_a").to_string();
            let b = get_cell(record, header_map, "col_b").to_string();
            Ok(format!("{}/{}", a, b))
        })
        .unwrap();
        assert_eq!(items, vec!["foo/123", "bar/456"]);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_parse_csv_row_error_collected() {
        // name が空の行（2行目）はエラーとして収集され、items には含まれない
        let csv = "name,id\ngood,1\n,2\nbad,3\n";
        let (items, errors) =
            parse_csv::<String, _>(csv.as_bytes(), |record, header_map, row_num| {
                parse_required_string(record, header_map, "name", row_num)
            })
            .unwrap();
        assert_eq!(items, vec!["good", "bad"]);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].row, 2);
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
        assert_eq!(ok, 1234.0);
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

    #[test]
    fn test_finish_csv_upload() {
        use crate::models::csv_import::CsvRowError;
        let result = crate::models::common::BulkCreateResponse {
            inserted: 3,
            skipped: 1,
        };
        let errors = vec![CsvRowError {
            row: 5,
            message: "エラー".to_string(),
        }];
        let response = finish_csv_upload(result, errors);
        assert_eq!(response.inserted, 3);
        assert_eq!(response.skipped, 1);
        assert_eq!(response.errors.len(), 1);
        assert_eq!(response.errors[0].row, 5);
    }
}
