use super::*;
use crate::api::dto::CsvRowError;
use crate::features::csv_page::CsvKind;
use crate::features::csv_sync::KindStatus;
use crate::support::csv_flow::CsvPreviewMeta;

#[test]
fn dated_status_key_marks_fresh_and_stale_by_kind() {
    let key = |kind: CsvKind, days: i64| dated_status(kind, days, "10/20", "12:00").2;
    assert_eq!(key(CsvKind::AssetStock, 0), "fresh");
    assert_eq!(key(CsvKind::AssetFund, 1), "stale");
    assert_eq!(key(CsvKind::Dividend, 7), "fresh");
    assert_eq!(key(CsvKind::DomesticStock, 8), "stale");
    assert_eq!(key(CsvKind::MutualFund, 40), "stale");
}

#[test]
fn compute_status_key_distinguishes_failure_unknown_and_never() {
    let failed = compute_status(
        CsvKind::Dividend,
        &Some(KindStatus::Failed {
            reason: "理由".to_string(),
        }),
        &None,
        true,
    );
    assert_eq!(failed.4, "failed");
    assert!(failed.2);

    let unknown = compute_status(CsvKind::Dividend, &None, &None, false);
    assert_eq!(unknown.4, "unknown");
    assert!(!unknown.2);

    let never = compute_status(CsvKind::Dividend, &None, &None, true);
    assert_eq!(never.4, "never");
    assert!(!never.2);
}

#[test]
fn save_disabled_by_preview_blocks_erroring_preview_only_for_all_or_nothing() {
    let error = || CsvRowError {
        row: 2,
        message: "数量が数値ではありません".to_string(),
    };
    let meta = |errors: Vec<CsvRowError>, has_rows: bool| CsvTabMeta {
        preview: Some(CsvPreviewMeta {
            valid_rows: 1,
            errors,
            has_rows,
        }),
        ..Default::default()
    };

    // 追加保存は有効行があればエラー付きでも保存できる
    assert!(!save_disabled_by_preview(&meta(vec![error()], true), false));
    // 全件置換はエラー行を含む CSV を backend が拒否するため保存を止める
    assert!(save_disabled_by_preview(&meta(vec![error()], true), true));
    // エラーが無ければ全件置換でも保存できる
    assert!(!save_disabled_by_preview(&meta(vec![], true), true));
    // 検証前・有効行0はどちらの方式でも保存不可
    assert!(save_disabled_by_preview(&meta(vec![], false), true));
    assert!(save_disabled_by_preview(&CsvTabMeta::default(), false));
}

#[test]
fn save_result_heading_splits_success_no_addition_and_failure() {
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

    // スキップだけで終わった保存は成功と同じ見た目にしない
    let skipped_only = CsvUploadResponse {
        inserted: 0,
        skipped: 2,
        errors: vec![],
    };
    let no_addition_heading = save_result_heading(&skipped_only);
    assert_ne!(no_addition_heading, success_heading);
    assert_ne!(no_addition_heading, failure_heading);
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
fn save_result_tone_distinguishes_failure_no_addition_and_success() {
    let clean = CsvUploadResponse {
        inserted: 1,
        skipped: 0,
        errors: vec![],
    };
    let partial = CsvUploadResponse {
        inserted: 2,
        skipped: 0,
        errors: vec![CsvRowError {
            row: 4,
            message: "数量が数値ではありません".to_string(),
        }],
    };
    let rejected = CsvUploadResponse {
        inserted: 0,
        skipped: 0,
        errors: vec![CsvRowError {
            row: 2,
            message: "数量が数値ではありません".to_string(),
        }],
    };

    // 各結果がどのトーンかを固定する(ペアの不一致だけでは ==/!= の入替を検出できない)
    assert_eq!(save_result_tone(&clean), SaveResultTone::Success);
    assert_eq!(save_result_tone(&partial), SaveResultTone::Failure);
    assert_eq!(save_result_tone(&rejected), SaveResultTone::Failure);

    // スキップだけでエラーが無い結果は成功の緑にしない
    let skipped_only = CsvUploadResponse {
        inserted: 0,
        skipped: 2,
        errors: vec![],
    };
    assert_eq!(save_result_tone(&skipped_only), SaveResultTone::NoAddition);
}

#[test]
fn save_result_line_joins_counts_without_mode() {
    let clean = CsvUploadResponse {
        inserted: 12,
        skipped: 0,
        errors: vec![],
    };
    assert_eq!(save_result_line(&clean), "12件反映");

    let partial = CsvUploadResponse {
        inserted: 2,
        skipped: 1,
        errors: vec![CsvRowError {
            row: 4,
            message: "数量が数値ではありません".to_string(),
        }],
    };
    let line = save_result_line(&partial);
    assert_eq!(
        line,
        format!(
            "{}・{}・{}",
            partial.inserted_text(),
            partial.skipped_text().unwrap(),
            partial.error_count_text().unwrap()
        )
    );
    assert!(
        !line.contains("追加保存"),
        "方式は画面上部に既出のため連結しない"
    );
}
