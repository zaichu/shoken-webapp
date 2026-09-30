use super::*;

#[test]
fn empty_state_icon_sizes() {
    assert_eq!(EmptyStateIcon::Warning.svg_class(), "h-16 w-16");
    assert_eq!(EmptyStateIcon::Search.svg_class(), "h-10 w-10");
    assert_eq!(EmptyStateIcon::Tray.svg_class(), "h-10 w-10");
}

#[test]
fn empty_state_icon_paths() {
    assert!(EmptyStateIcon::Search.path().starts_with("M21 21l-6-6"));
    assert!(EmptyStateIcon::Warning.path().starts_with("M12 9v2"));
    assert!(EmptyStateIcon::Tray.path().starts_with("M20 13V6"));
}
