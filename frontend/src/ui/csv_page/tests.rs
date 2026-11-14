use super::*;
use crate::features::csv_page::CsvKind;
use crate::features::csv_sync::KindStatus;

#[test]
fn stale_status_explains_why_same_age_has_different_freshness() {
    for days in 1..=7 {
        let stale = dated_status(CsvKind::AssetStock, days, "10/20", "12:00");
        let fresh = dated_status(CsvKind::Dividend, days, "10/20", "12:00");
        assert_ne!(stale.2, fresh.2);
        assert_ne!(stale.0, fresh.0);
    }
}

#[test]
fn dated_status_key_marks_fresh_and_stale_by_kind() {
    for (kind, days, key, needs_update) in [
        (CsvKind::AssetStock, 0, "fresh", false),
        (CsvKind::AssetFund, 1, "stale", true),
        (CsvKind::Dividend, 7, "fresh", false),
        (CsvKind::DomesticStock, 8, "stale", true),
        (CsvKind::MutualFund, 40, "stale", true),
    ] {
        let status = dated_status(kind, days, "10/20", "12:00");
        assert_eq!(status.2, key);
        assert_eq!(
            status.0.contains(labels::CSV_MANAGE_STATUS_UPDATE_REQUIRED),
            needs_update,
        );
    }
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

#[test]
fn saved_count_clamps_missing_and_negative_to_zero() {
    let row = |count| CsvKindRowData {
        kind: CsvKind::Dividend,
        count,
        latest_imported_at: None,
        sync_status: None,
    };
    assert_eq!(row(None).saved_count(), 0);
    assert_eq!(row(Some(-5)).saved_count(), 0);
    assert_eq!(row(Some(12)).saved_count(), 12);
    assert_eq!(
        row(Some(i64::MAX)).saved_count(),
        usize::try_from(i64::MAX).unwrap_or(usize::MAX)
    );
}
