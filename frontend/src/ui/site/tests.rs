use super::*;

#[test]
fn nav_items_cover_main_pages_with_labels() {
    let labels: Vec<&str> = NAV_ITEMS.iter().map(|(_, label)| *label).collect();
    assert_eq!(labels, ["ホーム", "資産管理", "取引明細", "銘柄検索"]);
}

#[test]
fn nav_items_link_to_main_pages() {
    let hrefs: Vec<&str> = NAV_ITEMS.iter().map(|(to, _)| *to).collect();
    assert_eq!(hrefs, ["/", "/assetbalance", "/receipts", "/search"]);
}

#[test]
fn nav_icons_cover_all_items() {
    for (to, _) in NAV_ITEMS {
        assert!(!nav_icon_inner(to).is_empty());
    }
    assert_ne!(nav_icon_inner("/"), nav_icon_inner("/search"));
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
    assert!(is_nav_active("/search", "/search"));
    assert!(!is_nav_active("/receipts", "/assetbalance"));
    assert!(!is_nav_active("/receipts-other?tab=dividend", "/receipts"));
    assert!(!is_nav_active("/search?next=/receipts", "/receipts"));
}

#[test]
fn initials_prefers_first_and_last_name_part() {
    use crate::ui::avatar::avatar_initials;
    assert_eq!(avatar_initials(Some("Taro Yamada"), None), "TY");
    assert_eq!(avatar_initials(Some("zaichu"), None), "ZA");
    assert_eq!(avatar_initials(None, Some("user@example.com")), "US");
    assert_eq!(avatar_initials(None, None), "U");
}
