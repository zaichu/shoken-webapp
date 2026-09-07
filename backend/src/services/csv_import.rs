use crate::errors::ApiError;
use crate::models::common::BulkCreateResponse;
use crate::models::csv_import::{CsvPreviewResponse, CsvRowError, CsvUploadResponse};
use crate::services::csv_pipeline::{parse_csv_with_config, CsvParserConfig, CsvRow};
use std::future::Future;

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

/// CSV bytes をパース → 行 transform → preview response 化を共通化する
pub fn build_csv_preview<T, F>(
    bytes: &[u8],
    config: &CsvParserConfig,
    transform_rows: F,
) -> Result<CsvPreviewResponse, ApiError>
where
    T: serde::Serialize,
    F: FnOnce(&[CsvRow]) -> (Vec<T>, Vec<CsvRowError>),
{
    let rows = parse_csv_with_config(bytes, config)?;
    let (items, errors) = transform_rows(&rows);
    Ok(build_preview_response(&items, errors))
}

/// CSV bytes をパース → 行 transform → bulk_create → upload response 化を共通化する
pub async fn run_csv_upload<T, F, BulkFn, BulkFut>(
    bytes: &[u8],
    config: &CsvParserConfig,
    transform_rows: F,
    bulk_create: BulkFn,
) -> Result<CsvUploadResponse, ApiError>
where
    F: FnOnce(&[CsvRow]) -> (Vec<T>, Vec<CsvRowError>),
    BulkFn: FnOnce(Vec<T>) -> BulkFut,
    BulkFut: Future<Output = Result<BulkCreateResponse, ApiError>>,
{
    let rows = parse_csv_with_config(bytes, config)?;
    let (items, errors) = transform_rows(&rows);
    let result = bulk_create(items).await?;
    Ok(finish_csv_upload(result, errors))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn test_finish_csv_upload() {
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

    const PREVIEW_TEST_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 0,
        exclude_row_fn: None,
        required_columns: &["name", "amount"],
    };

    fn collect_names(rows: &[CsvRow]) -> (Vec<String>, Vec<CsvRowError>) {
        let mut items = Vec::new();
        let mut errors = Vec::new();
        for (index, row) in rows.iter().enumerate() {
            let name = row.get("name").cloned().unwrap_or_default();
            if name.is_empty() {
                errors.push(CsvRowError {
                    row: index + 1,
                    message: "name is empty".to_string(),
                });
            } else {
                items.push(name);
            }
        }
        (items, errors)
    }

    #[test]
    fn test_build_csv_preview() {
        let preview = build_csv_preview(
            "name,amount\nfoo,100\n,200\nbar,300\n".as_bytes(),
            &PREVIEW_TEST_CONFIG,
            collect_names,
        )
        .unwrap();
        assert_eq!(
            (
                preview.total_rows,
                preview.valid_rows,
                preview.errors.len(),
                preview.errors.first().map(|error| error.row),
                preview.rows.len(),
            ),
            (3, 2, 1, Some(2), 2)
        );

        assert!(matches!(
            build_csv_preview(b"", &PREVIEW_TEST_CONFIG, collect_names),
            Err(ApiError::ValidationError(_))
        ));
    }

    #[tokio::test]
    async fn test_run_csv_upload() {
        let response = run_csv_upload(
            "name,amount\nfoo,1\n,2\nbar,3\n".as_bytes(),
            &PREVIEW_TEST_CONFIG,
            collect_names,
            |items| async move {
                Ok(BulkCreateResponse {
                    inserted: items.len(),
                    skipped: 0,
                })
            },
        )
        .await
        .unwrap();
        assert_eq!(
            (
                response.inserted,
                response.skipped,
                response.errors.len(),
                response.errors.first().map(|error| error.row),
            ),
            (2, 0, 1, Some(2))
        );

        let bulk_invoked = std::cell::Cell::new(false);
        let result = run_csv_upload(
            b"",
            &PREVIEW_TEST_CONFIG,
            collect_names,
            |_items: Vec<String>| {
                bulk_invoked.set(true);
                async {
                    Ok(BulkCreateResponse {
                        inserted: 0,
                        skipped: 0,
                    })
                }
            },
        )
        .await;
        assert!(matches!(result, Err(ApiError::ValidationError(_))));
        assert!(
            !bulk_invoked.get(),
            "bulk_create must not run when parse fails"
        );
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
