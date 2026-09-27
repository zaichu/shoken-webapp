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
