use super::*;

#[test]
fn save_result_heading_splits_success_partial_and_full_failure() {
    let clean = CsvUploadResponse {
        inserted: 1,
        skipped: 0,
        errors: vec![],
    };
    assert_eq!(save_result_heading(&clean), "保存しました");

    let partial = CsvUploadResponse {
        inserted: 2,
        skipped: 0,
        errors: vec![CsvRowError {
            row: 4,
            message: "数量が数値ではありません".to_string(),
        }],
    };
    assert_eq!(
        save_result_heading(&partial),
        "一部の行を保存できませんでした"
    );

    let rejected = CsvUploadResponse {
        inserted: 0,
        skipped: 1,
        errors: vec![CsvRowError {
            row: 2,
            message: "数量が数値ではありません".to_string(),
        }],
    };
    assert_eq!(save_result_heading(&rejected), "保存できませんでした");
}

#[test]
fn save_result_line_joins_counts_and_mode() {
    let clean = CsvUploadResponse {
        inserted: 12,
        skipped: 0,
        errors: vec![],
    };
    assert_eq!(save_result_line(&clean, "追加保存"), "12件反映・追加保存");

    let partial = CsvUploadResponse {
        inserted: 2,
        skipped: 1,
        errors: vec![CsvRowError {
            row: 4,
            message: "数量が数値ではありません".to_string(),
        }],
    };
    assert_eq!(
        save_result_line(&partial, "全件置換"),
        "2件反映・全件置換・1件スキップ・1件エラー"
    );
}
