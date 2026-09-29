use super::*;

#[test]
fn initials_from_two_word_name() {
    assert_eq!(get_initials(Some("John Doe"), None), "JD");
    assert_eq!(get_initials(Some("田中 太郎"), None), "田太");
}

#[test]
fn initials_from_single_word_name() {
    assert_eq!(get_initials(Some("Taro"), None), "TA");
    assert_eq!(get_initials(Some("taro"), None), "TA");
}

#[test]
fn initials_from_email() {
    assert_eq!(get_initials(None, Some("test@example.com")), "TE");
    assert_eq!(get_initials(Some(""), Some("test@example.com")), "TE");
}

#[test]
fn initials_default_when_unset() {
    assert_eq!(get_initials(None, None), "U");
    assert_eq!(get_initials(None, Some("")), "U");
}

#[test]
fn nav_active_matches_path_or_prefix() {
    assert!(is_nav_active("/receipts", "/receipts"));
    assert!(is_nav_active("/receipts/2024", "/receipts"));
    assert!(is_nav_active("/receipts?tab=domesticstock", "/receipts"));
    assert!(!is_nav_active("/receipt", "/receipts"));
    assert!(!is_nav_active("/", "/receipts"));
}

#[test]
fn alert_variant_classes() {
    assert_eq!(
        AlertVariant::Warning.class(),
        "border-accent-border bg-accent-soft text-accent-text"
    );
    assert_eq!(
        AlertVariant::Danger.class(),
        "border-negative-border bg-negative-soft text-negative-vivid"
    );
}

#[test]
fn spinner_size_classes() {
    assert_eq!(SpinnerSize::Sm.class(), "h-4 w-4");
    assert_eq!(SpinnerSize::Lg.class(), "h-8 w-8");
}
