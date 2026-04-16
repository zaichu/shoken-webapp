#[cfg(test)]
use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::services::csv_pipeline::CsvRow;
#[cfg(test)]
use crate::services::csv_util::decode_bytes;
#[cfg(test)]
use std::collections::HashMap;

/// CSV bytes をデコードして行ごとにパース
/// parse_row が Err を返した行はエラーとして収集し、items には含めない
#[cfg(test)]
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

#[cfg(test)]
fn is_all_empty_record(record: &csv::StringRecord) -> bool {
    record.iter().all(|value| value.trim().is_empty())
}

pub fn validate_csv_rows<T, F>(rows: &[CsvRow], transform_row: F) -> (Vec<T>, Vec<CsvRowError>)
where
    F: Fn(&CsvRow, usize) -> Result<T, CsvRowError>,
{
    let mut items = Vec::new();
    let mut errors = Vec::new();

    for (index, row) in rows.iter().enumerate() {
        let row_num = index + 1;
        match transform_row(row, row_num) {
            Ok(item) => items.push(item),
            Err(error) => errors.push(error),
        }
    }

    (items, errors)
}

pub fn build_preview_response<T>(items: &[T], errors: Vec<CsvRowError>) -> CsvPreviewResponse
where
    T: serde::Serialize,
{
    let rows = items
        .iter()
        .map(|item| serde_json::to_value(item).unwrap_or(serde_json::Value::Null))
        .collect();

    CsvPreviewResponse {
        total_rows: items.len() + errors.len(),
        valid_rows: items.len(),
        errors,
        rows,
    }
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
    use {
        super::*,
        crate::services::csv_util::{get_cell, parse_required_string},
    };
    fn parse_pair_csv(csv: &str) -> (Vec<String>, Vec<CsvRowError>) {
        parse_csv::<String, _>(csv.as_bytes(), |record, header_map, _row| {
            let a = get_cell(record, header_map, "col_a").to_string();
            let b = get_cell(record, header_map, "col_b").to_string();
            Ok(format!("{a}/{b}"))
        })
        .unwrap()
    }
    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }
    #[test]
    fn test_parse_csv() {
        let (items, errors) = parse_pair_csv("col_a,col_b\nfoo,123\nbar,456\n");
        assert_eq!(
            (items, errors.is_empty()),
            (strings(&["foo/123", "bar/456"]), true)
        );
        let (items, errors) = parse_csv::<String, _>(
            "name,id\ngood,1\n,2\nbad,3\n".as_bytes(),
            |record, header_map, row_num| {
                parse_required_string(record, header_map, "name", row_num)
            },
        )
        .unwrap();
        assert_eq!(
            (items, errors.len(), errors.first().map(|error| error.row)),
            (strings(&["good", "bad"]), 1, Some(2))
        );
        let (items, errors) = parse_pair_csv("col_a,col_b\nfoo,123\n,\nbar,456\n");
        assert_eq!(
            (items, errors.is_empty()),
            (strings(&["foo/123", "bar/456"]), true)
        );
        let (items, errors) = parse_pair_csv("col_a,col_b\nfoo,123\nbar\nbaz,456\n");
        assert_eq!(
            (
                items,
                errors.len(),
                errors.first().map(|error| error.row),
                errors
                    .first()
                    .is_some_and(|error| error.message.contains("CSV行の読み込み"))
            ),
            (strings(&["foo/123", "baz/456"]), 1, Some(2), true)
        );
        let response = finish_csv_upload(
            crate::models::common::BulkCreateResponse {
                inserted: 3,
                skipped: 1,
            },
            vec![CsvRowError {
                row: 5,
                message: "エラー".to_string(),
            }],
        );
        assert_eq!(
            (
                response.inserted,
                response.skipped,
                response.errors.len(),
                response.errors.first().map(|error| error.row)
            ),
            (3, 1, 1, Some(5))
        );
    }

    #[test]
    fn test_validate_csv_rows() {
        let rows: Vec<CsvRow> = vec![
            [("key".to_string(), "a".to_string())]
                .iter()
                .cloned()
                .collect(),
            [("key".to_string(), "b".to_string())]
                .iter()
                .cloned()
                .collect(),
        ];
        let (items, errors): (Vec<String>, _) = validate_csv_rows(&rows, |row, _row_num| {
            Ok(row.get("key").cloned().unwrap_or_default())
        });
        assert_eq!((items, errors.is_empty()), (strings(&["a", "b"]), true));

        let rows: Vec<CsvRow> = vec![
            [("key".to_string(), "ok".to_string())]
                .iter()
                .cloned()
                .collect(),
            [("key".to_string(), "bad".to_string())]
                .iter()
                .cloned()
                .collect(),
        ];
        let (items, errors): (Vec<String>, _) = validate_csv_rows(&rows, |row, row_num| {
            let value = row.get("key").cloned().unwrap_or_default();
            if value == "bad" {
                Err(CsvRowError {
                    row: row_num,
                    message: "invalid".to_string(),
                })
            } else {
                Ok(value)
            }
        });
        assert_eq!(items, strings(&["ok"]));
        assert_eq!(
            (errors.len(), errors.first().map(|error| error.row)),
            (1, Some(2))
        );

        let (items, errors): (Vec<String>, _) = validate_csv_rows(&[], |_, _| Ok("".to_string()));
        assert_eq!((items.is_empty(), errors.is_empty()), (true, true));
    }

    #[test]
    fn test_build_preview_response() {
        let items = vec!["alpha", "beta"];
        let errors = vec![CsvRowError {
            row: 3,
            message: "error".to_string(),
        }];
        let response = build_preview_response(&items, errors);
        assert_eq!(
            (
                response.total_rows,
                response.valid_rows,
                response.errors.len(),
                response.rows.len()
            ),
            (3, 2, 1, 2)
        );
        assert_eq!(response.errors[0].row, 3);

        let empty: Vec<String> = vec![];
        let response = build_preview_response(&empty, vec![]);
        assert_eq!((response.total_rows, response.valid_rows), (0, 0));
    }
}
