use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvRowError, CsvUploadResponse};
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
/// 例: "1,234" → 1234.0、"(500)" → -500.0
pub fn parse_number(s: &str) -> Result<f64, String> {
    let s = s.trim();
    if s.is_empty() {
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

/// bulk_create 結果と行エラーから CsvUploadResponse を構築
pub fn finish_csv_upload(result: BulkCreateResponse, errors: Vec<CsvRowError>) -> CsvUploadResponse {
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
}
