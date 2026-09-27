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
    assert_eq!(Route::Home.title(), "ホーム");
    assert_eq!(Route::Search.title(), "銘柄検索");
    assert_eq!(Route::Receipts.title(), "取引明細");
    assert_eq!(Route::AssetBalance.title(), "資産管理");
    assert_eq!(Route::Login.title(), "ログイン");
    assert_eq!(Route::NotFound.title(), "ページが見つかりません");
    assert!(!Route::Home.protected());
    assert!(!Route::Search.protected());
    assert!(Route::Receipts.protected());
    assert!(Route::AssetBalance.protected());
    assert!(!Route::Login.protected());
    assert!(!Route::NotFound.protected());
}
