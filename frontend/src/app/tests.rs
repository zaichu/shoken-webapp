use super::*;

#[test]
fn route_for_known_and_unknown_paths() {
    assert_eq!(route_for_path("/"), Route::Home);
    assert_eq!(route_for_path("/search"), Route::Search);
    assert_eq!(route_for_path("/receipts"), Route::Receipts);
    assert_eq!(route_for_path("/assetbalance"), Route::AssetBalance);
    assert_eq!(route_for_path("/login"), Route::Login);
    assert_eq!(route_for_path("/404"), Route::NotFound);
    assert_eq!(route_for_path("/unknown"), Route::NotFound);
    assert_eq!(route_for_path(""), Route::NotFound);
}

#[test]
fn route_titles_and_protection() {
    for route in [
        Route::Home,
        Route::Search,
        Route::Receipts,
        Route::AssetBalance,
        Route::Login,
        Route::NotFound,
    ] {
        assert!(!route.title().is_empty());
    }
    assert!(!Route::Home.protected());
    assert!(!Route::Search.protected());
    assert!(Route::Receipts.protected());
    assert!(Route::AssetBalance.protected());
    assert!(!Route::Login.protected());
    assert!(!Route::NotFound.protected());
}

#[test]
fn spa_path_only_accepts_root_relative() {
    assert_eq!(spa_path("/receipts").as_deref(), Some("/receipts"));
    assert_eq!(
        spa_path("/search?code=7203").as_deref(),
        Some("/search?code=7203")
    );
    assert_eq!(spa_path("https://example.com/"), None);
    assert_eq!(spa_path("//example.com/"), None);
    assert_eq!(spa_path("mailto:a@example.com"), None);
    assert_eq!(spa_path("#main-content"), None);
    assert_eq!(spa_path("receipts"), None);
    assert_eq!(spa_path(""), None);
}

#[test]
fn pathname_of_strips_query_and_hash() {
    assert_eq!(pathname_of("/receipts?tab=domesticstock"), "/receipts");
    assert_eq!(pathname_of("/search?code=7203#x"), "/search");
    assert_eq!(pathname_of("/assetbalance"), "/assetbalance");
    assert_eq!(pathname_of("/?code=7203#summary"), "/");
    assert_eq!(pathname_of("/login#session"), "/login");
    assert_eq!(pathname_of(""), "");
    assert_eq!(
        route_for_path(pathname_of("/search?code=7203")),
        Route::Search
    );
}

#[test]
fn hash_only_change_detects_fragment_append() {
    assert!(hash_only_change("/receipts", "/receipts#list"));
    assert!(hash_only_change("/receipts?t=1", "/receipts?t=1#list"));
    assert!(!hash_only_change("/receipts", "/receipts"));
    assert!(!hash_only_change("/receipts", "/receipts/other#x"));
    assert!(!hash_only_change("/receipts", "/search#x"));
}

#[test]
fn strip_hash_keeps_query() {
    assert_eq!(strip_hash("/receipts#list"), "/receipts");
    assert_eq!(strip_hash("/search?code=7203#x"), "/search?code=7203");
    assert_eq!(strip_hash("/receipts"), "/receipts");
}

#[test]
fn spa_href_parts_rejects_download_target_and_external() {
    // アプリ内遷移にするのは href がルート相対で download も target=_self 以外の target もないリンクだけ
    assert_eq!(
        spa_href_parts(Some("/receipts"), false, None).as_deref(),
        Some("/receipts")
    );
    assert_eq!(
        spa_href_parts(Some("/receipts"), false, Some("_self")).as_deref(),
        Some("/receipts")
    );
    assert_eq!(spa_href_parts(Some("/receipts"), true, None), None);
    assert_eq!(
        spa_href_parts(Some("/receipts"), false, Some("_blank")),
        None
    );
    assert_eq!(
        spa_href_parts(Some("https://example.com/"), false, None),
        None
    );
    assert_eq!(spa_href_parts(Some("#main-content"), false, None), None);
    assert_eq!(spa_href_parts(None, false, None), None);
}
