use super::*;

#[test]
fn nav_items_cover_main_pages_with_labels() {
    let labels: Vec<&str> = NAV_ITEMS.iter().map(|(_, label)| *label).collect();
    assert_eq!(labels, ["ホーム", "資産管理", "取引明細", "銘柄検索"]);
}

#[test]
fn nav_active_matches_root_only_exactly() {
    assert!(is_nav_active("/", "/"));
    assert!(!is_nav_active("/assetbalance", "/"));
}

#[test]
fn nav_active_covers_sub_paths() {
    assert!(is_nav_active("/assetbalance", "/assetbalance"));
    assert!(is_nav_active("/assetbalance/composition", "/assetbalance"));
    assert!(is_nav_active("/receipts?kind=stock", "/receipts"));
    assert!(!is_nav_active("/receipts", "/assetbalance"));
    assert!(!is_nav_active("/receipts-other?tab=dividend", "/receipts"));
    assert!(!is_nav_active("/search?next=/receipts", "/receipts"));
}

#[test]
fn initials_prefers_first_and_last_name_part() {
    assert_eq!(get_initials(Some("Taro Yamada"), None), "TY");
    assert_eq!(get_initials(Some("zaichu"), None), "ZA");
    assert_eq!(get_initials(None, Some("user@example.com")), "US");
    assert_eq!(get_initials(None, None), "U");
}
