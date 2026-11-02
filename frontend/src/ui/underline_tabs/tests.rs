use super::{SWIPE_EDGE_GUARD_PX, is_edge_start, next_swipe_index, next_tab_index, swipe_step};

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

#[test]
fn swipe_needs_horizontal_distance_twice_the_vertical() {
    assert!(swipe_step(-80.0, 10.0).is_some());
    assert!(swipe_step(80.0, 10.0).is_some());
    assert_eq!(swipe_step(-59.0, 0.0), None);
    assert_eq!(swipe_step(-80.0, 50.0), None);
    assert_eq!(swipe_step(0.0, -100.0), None);
}

#[test]
fn swipe_stops_at_edge_tabs_without_wrapping() {
    assert_eq!(next_swipe_index(3, 0, -80.0, 0.0), Some(1));
    assert_eq!(next_swipe_index(3, 2, -80.0, 0.0), None);
    assert_eq!(next_swipe_index(3, 2, 80.0, 0.0), Some(1));
    assert_eq!(next_swipe_index(3, 0, 80.0, 0.0), None);
    assert_eq!(next_swipe_index(3, 1, -10.0, 0.0), None);
}

#[test]
fn edge_starts_are_ignored() {
    assert!(is_edge_start(0.0, 390.0));
    assert!(is_edge_start(SWIPE_EDGE_GUARD_PX - 1.0, 390.0));
    assert!(!is_edge_start(SWIPE_EDGE_GUARD_PX, 390.0));
    assert!(is_edge_start(389.0, 390.0));
    assert!(!is_edge_start(195.0, 390.0));
}
