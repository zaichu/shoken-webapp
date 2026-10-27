use super::*;
use crate::features::csv_page::CsvKind;
use crate::features::csv_sync::KindStatus;

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
fn status_dot_appears_only_for_dated_or_active_status() {
    for key in [
        "fresh",
        "stale",
        "failed",
        "fetching",
        "importing",
        "confirming",
    ] {
        assert!(status_dot_class(key).is_some(), "{key} は点を出す");
    }
    for key in ["never", "unknown", "skipped"] {
        assert!(status_dot_class(key).is_none(), "{key} は点を出さない");
    }
}
