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
    assert_eq!(get_initials(Some("田中太郎"), None), "田中");
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
    assert!(is_nav_active(
        "/receipts/2024?tab=dividend#summary",
        "/receipts"
    ));
    assert!(!is_nav_active("/receipts-other?tab=dividend", "/receipts"));
    assert!(!is_nav_active("/search?next=/receipts", "/receipts"));
}
