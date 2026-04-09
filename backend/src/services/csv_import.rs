use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::services::csv_util::decode_bytes;
use std::collections::HashMap;

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
        if is_all_empty_record(&record) {
            continue;
        }
        match parse_row(&record, &header_map, row_num) {
            Ok(item) => items.push(item),
            Err(e) => errors.push(e),
        }
    }

    Ok((items, errors))
}

fn is_all_empty_record(record: &csv::StringRecord) -> bool {
    record.iter().all(|value| value.trim().is_empty())
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
#[cfg(test)] #[rustfmt::skip] mod tests { use {super::*, crate::services::csv_util::{get_cell, parse_required_string}}; fn parse_pair_csv(csv: &str) -> (Vec<String>, Vec<CsvRowError>) { parse_csv::<String, _>(csv.as_bytes(), |record, header_map, _row| { let a = get_cell(record, header_map, "col_a").to_string(); let b = get_cell(record, header_map, "col_b").to_string(); Ok(format!("{a}/{b}")) }).unwrap() } fn strings(values: &[&str]) -> Vec<String> { values.iter().map(|value| (*value).to_string()).collect() } #[test] fn test_parse_csv() { let (items, errors) = parse_pair_csv("col_a,col_b\nfoo,123\nbar,456\n"); assert_eq!((items, errors.is_empty()), (strings(&["foo/123", "bar/456"]), true)); let (items, errors) = parse_csv::<String, _>("name,id\ngood,1\n,2\nbad,3\n".as_bytes(), |record, header_map, row_num| parse_required_string(record, header_map, "name", row_num)).unwrap(); assert_eq!((items, errors.len(), errors.first().map(|error| error.row)), (strings(&["good", "bad"]), 1, Some(2))); let (items, errors) = parse_pair_csv("col_a,col_b\nfoo,123\n,\nbar,456\n"); assert_eq!((items, errors.is_empty()), (strings(&["foo/123", "bar/456"]), true)); let (items, errors) = parse_pair_csv("col_a,col_b\nfoo,123\nbar\nbaz,456\n"); assert_eq!((items, errors.len(), errors.first().map(|error| error.row), errors.first().is_some_and(|error| error.message.contains("CSV行の読み込み"))), (strings(&["foo/123", "baz/456"]), 1, Some(2), true)); let response = finish_csv_upload(crate::models::common::BulkCreateResponse { inserted: 3, skipped: 1 }, vec![CsvRowError { row: 5, message: "エラー".to_string() }]); assert_eq!((response.inserted, response.skipped, response.errors.len(), response.errors.first().map(|error| error.row)), (3, 1, 1, Some(5))); } }
