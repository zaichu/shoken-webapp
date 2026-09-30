use super::*;

#[test]
fn save_result_heading_splits_success_partial_and_full_failure() {
    let clean = CsvUploadResponse {
        inserted: 1,
        skipped: 0,
        errors: vec![],
    };
    let success_heading = save_result_heading(&clean);

    let partial = CsvUploadResponse {
        inserted: 2,
        skipped: 0,
        errors: vec![CsvRowError {
            row: 4,
            message: "数量が数値ではありません".to_string(),
        }],
    };
    let partial_heading = save_result_heading(&partial);

    let rejected = CsvUploadResponse {
        inserted: 0,
        skipped: 1,
        errors: vec![CsvRowError {
            row: 2,
            message: "数量が数値ではありません".to_string(),
        }],
    };
    let failure_heading = save_result_heading(&rejected);
    assert_ne!(success_heading, partial_heading);
    assert_ne!(success_heading, failure_heading);
    assert_ne!(partial_heading, failure_heading);
    let skipped_only = CsvUploadResponse {
        inserted: 0,
        skipped: 2,
        errors: vec![],
    };
    assert_eq!(save_result_heading(&skipped_only), success_heading);
    let another_partial = CsvUploadResponse {
        inserted: 10,
        ..partial
    };
    assert_eq!(save_result_heading(&another_partial), partial_heading);
    let another_failure = CsvUploadResponse {
        skipped: 0,
        ..rejected
    };
    assert_eq!(save_result_heading(&another_failure), failure_heading);
}

#[test]
fn save_result_line_joins_counts_and_mode() {
    let clean = CsvUploadResponse {
        inserted: 12,
        skipped: 0,
        errors: vec![],
    };
    let clean_line = save_result_line(&clean, "追加保存");
    assert!(clean_line.contains(&clean.inserted_text()));
    assert!(clean_line.contains("追加保存"));

    let partial = CsvUploadResponse {
        inserted: 2,
        skipped: 1,
        errors: vec![CsvRowError {
            row: 4,
            message: "数量が数値ではありません".to_string(),
        }],
    };
    let partial_line = save_result_line(&partial, "全件置換");
    for fragment in [
        partial.inserted_text(),
        partial.skipped_text().unwrap(),
        partial.error_count_text().unwrap(),
        "全件置換".to_string(),
    ] {
        assert!(partial_line.contains(&fragment));
    }
}
