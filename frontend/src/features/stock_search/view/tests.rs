use super::*;

#[test]
fn stock_links_contain_code_placeholder() {
    assert!(!STOCK_LINKS.is_empty());
    for (name, template) in STOCK_LINKS {
        assert!(!name.is_empty());
        assert!(template.contains("{code}"));
        assert!(template.starts_with("https://"));
    }
}

#[test]
fn empty_search_description_differs_for_initial_and_no_results() {
    assert_eq!(
        empty_search_description("銘柄を検索"),
        "銘柄コード（例：7203）または銘柄名を入力して検索してください。"
    );
    assert_eq!(
        empty_search_description("該当する銘柄が見つかりませんでした"),
        "銘柄コードまたは銘柄名を確認してください。"
    );
}
