use super::next_tab_index;

#[test]
fn arrow_keys_cycle_and_home_end_jump() {
    assert_eq!(next_tab_index(3, 0, "ArrowRight"), Some(1));
    assert_eq!(next_tab_index(3, 2, "ArrowRight"), Some(0));
    assert_eq!(next_tab_index(3, 0, "ArrowLeft"), Some(2));
    assert_eq!(next_tab_index(3, 1, "Home"), Some(0));
    assert_eq!(next_tab_index(3, 1, "End"), Some(2));
    assert_eq!(next_tab_index(3, 1, "Enter"), None);
    assert_eq!(next_tab_index(0, 0, "ArrowRight"), None);
}
