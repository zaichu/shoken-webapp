use super::{
    SWIPE_EDGE_GUARD_PX, SWIPE_SETTLE_GUARD_MS, SWIPE_SETTLE_MS, clamp_drag_offset, is_edge_start,
    next_swipe_index, next_tab_index, swipe_can_advance, swipe_commit_index, swipe_lock_allowed,
    swipe_progress, swipe_settle_offset, swipe_should_ignore_input, swipe_should_ignore_touchstart,
    swipe_step, swipe_velocity, tab_ink_offset,
};

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

#[test]
fn half_width_drag_commits_to_neighbor() {
    assert_eq!(swipe_commit_index(3, 0, -195.0, 0.0, 390.0, 0.0), Some(1));
    assert_eq!(swipe_commit_index(3, 1, 200.0, 0.0, 390.0, 0.0), Some(0));
    assert_eq!(swipe_commit_index(3, 1, -194.0, 0.0, 390.0, 0.0), None);
}

#[test]
fn fast_flick_commits_without_half_width() {
    assert_eq!(swipe_commit_index(3, 0, -40.0, 0.0, 390.0, -0.8), Some(1));
    assert_eq!(swipe_commit_index(3, 1, 40.0, 0.0, 390.0, 0.8), Some(0));
    assert_eq!(swipe_commit_index(3, 0, -40.0, 0.0, 390.0, -0.1), None);
    assert_eq!(swipe_commit_index(3, 0, -40.0, 0.0, 390.0, 0.8), None);
}

#[test]
fn commit_ignores_vertical_moves_and_edges() {
    assert_eq!(swipe_commit_index(3, 0, -200.0, -150.0, 390.0, 0.0), None);
    assert_eq!(swipe_commit_index(3, 0, 200.0, 0.0, 390.0, 0.0), None);
    assert_eq!(swipe_commit_index(3, 2, -200.0, 0.0, 390.0, -1.0), None);
    assert_eq!(swipe_commit_index(3, 1, 0.0, 0.0, 390.0, -1.0), None);
    assert_eq!(swipe_commit_index(0, 0, -200.0, 0.0, 390.0, 0.0), None);
}

fn drag_is_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "drag {actual} は {expected} であること"
    );
}

#[test]
fn drag_follows_finger_and_resists_at_edges() {
    drag_is_close(clamp_drag_offset(-100.0, 3, 0, 390.0), -100.0);
    drag_is_close(clamp_drag_offset(-500.0, 3, 0, 390.0), -390.0);
    drag_is_close(clamp_drag_offset(100.0, 3, 0, 390.0), 35.0);
    drag_is_close(clamp_drag_offset(-500.0, 3, 2, 390.0), -96.0);
    drag_is_close(clamp_drag_offset(0.0, 3, 1, 390.0), 0.0);
    drag_is_close(clamp_drag_offset(-100.0, 0, 0, 390.0), 0.0);
}

#[test]
fn only_existing_neighbors_can_advance() {
    assert!(swipe_can_advance(3, 0, -10.0));
    assert!(!swipe_can_advance(3, 0, 10.0));
    assert!(!swipe_can_advance(3, 2, -10.0));
    assert!(swipe_can_advance(3, 2, 10.0));
    assert!(!swipe_can_advance(0, 0, -10.0));
}

#[test]
fn velocity_comes_from_recent_frames() {
    assert_eq!(swipe_velocity(&[]), 0.0);
    assert_eq!(swipe_velocity(&[(0.0, 300.0)]), 0.0);
    assert_eq!(swipe_velocity(&[(0.0, 300.0), (0.0, 250.0)]), 0.0);
    let flick = swipe_velocity(&[(0.0, 300.0), (16.0, 280.0), (32.0, 250.0)]);
    assert!((flick + 1.5625).abs() < 1e-9);
    let stale = swipe_velocity(&[(0.0, 100.0), (500.0, 290.0), (516.0, 270.0)]);
    assert!((stale + 1.25).abs() < 1e-9);
}

#[test]
fn reduced_motion_keeps_distance_only_decision() {
    assert_eq!(next_swipe_index(3, 0, -80.0, 0.0), Some(1));
    assert_eq!(next_swipe_index(3, 0, -40.0, 0.0), None);
}

#[test]
fn lock_ignores_contacts_that_never_switch() {
    assert!(swipe_lock_allowed(false, false, 195.0, Some(390.0)));
    assert!(!swipe_lock_allowed(true, false, 195.0, Some(390.0)));
    assert!(!swipe_lock_allowed(false, true, 195.0, Some(390.0)));
    assert!(!swipe_lock_allowed(false, false, 5.0, Some(390.0)));
    assert!(!swipe_lock_allowed(false, false, 385.0, Some(390.0)));
    assert!(!swipe_lock_allowed(false, false, 195.0, None));
}

#[test]
fn ink_follows_progress_between_tabs() {
    assert_eq!(tab_ink_offset(1, 0.0), 1.0);
    assert_eq!(tab_ink_offset(1, -0.5), 0.5);
    assert_eq!(tab_ink_offset(0, 2.0), 1.0);
}

#[test]
fn commit_settle_continues_outward_without_reversing() {
    let width = 390.0;
    let next_to = swipe_settle_offset(0, 1, width);
    let next_from = clamp_drag_offset(-195.0, 3, 0, width);
    assert!(next_from < 0.0 && next_to < 0.0);
    assert!(next_to < next_from);

    let prev_to = swipe_settle_offset(1, 0, width);
    let prev_from = clamp_drag_offset(195.0, 3, 1, width);
    assert!(prev_from > 0.0 && prev_to > 0.0);
    assert!(prev_to > prev_from);
}

#[test]
fn cancel_settle_returns_to_rest() {
    assert_eq!(swipe_settle_offset(1, 1, 390.0), 0.0);
}

#[test]
fn progress_points_toward_neighbor() {
    assert_eq!(swipe_progress(-195.0, 390.0), 0.5);
    assert_eq!(swipe_progress(195.0, 390.0), -0.5);
    assert_eq!(swipe_progress(-800.0, 390.0), 1.0);
    assert_eq!(swipe_progress(100.0, 0.0), 0.0);
    assert_eq!(tab_ink_offset(0, swipe_progress(-195.0, 390.0)), 0.5);
    assert_eq!(tab_ink_offset(1, swipe_progress(195.0, 390.0)), 0.5);
}

#[test]
fn settling_ignores_taps_and_resumes_after() {
    assert!(swipe_should_ignore_input(true, false));
    assert!(!swipe_should_ignore_input(false, false));
    assert!(!swipe_should_ignore_input(true, true));
    assert!(!swipe_should_ignore_input(false, true));
}

#[test]
fn settling_ignores_new_swipe_and_second_finger() {
    assert!(swipe_should_ignore_touchstart(true, false, 1));
    assert!(swipe_should_ignore_touchstart(false, false, 2));
    assert!(!swipe_should_ignore_touchstart(false, false, 1));
    assert!(!swipe_should_ignore_touchstart(true, true, 1));
    assert!(!swipe_should_ignore_touchstart(false, true, 2));
}

#[test]
#[expect(clippy::assertions_on_constants)]
fn guard_releases_after_settle() {
    assert!(SWIPE_SETTLE_GUARD_MS > SWIPE_SETTLE_MS);
}
