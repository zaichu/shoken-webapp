use super::should_close_panel;

#[test]
fn close_only_when_compact_and_open() {
    assert!(should_close_panel(true, true));
    assert!(!should_close_panel(true, false));
    assert!(!should_close_panel(false, true));
    assert!(!should_close_panel(false, false));
}
