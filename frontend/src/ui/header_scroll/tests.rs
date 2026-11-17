use super::{HeaderScroll, ScrollMetrics};

const MAX: f64 = 5000.0;

fn at(top: f64) -> ScrollMetrics {
    ScrollMetrics {
        top,
        max: MAX,
        focus_within: false,
    }
}

fn scrolled_to(top: f64) -> HeaderScroll {
    HeaderScroll::default().update(at(top))
}

#[test]
fn scrolling_down_hides_and_up_shows() {
    let hidden = scrolled_to(super::HIDE_AFTER + 200.0);
    let hidden = hidden.update(at(super::HIDE_AFTER + 200.0 + super::MOVE_THRESHOLD * 3.0));
    assert!(hidden.hidden);
    let shown = hidden.update(at(super::HIDE_AFTER + 200.0));
    assert!(!shown.hidden);
}

#[test]
fn tiny_movement_keeps_state() {
    let hidden = scrolled_to(500.0).update(at(500.0 + super::MOVE_THRESHOLD * 2.0));
    assert!(hidden.hidden);
    let same = hidden.update(at(
        500.0 + super::MOVE_THRESHOLD * 2.0 - super::MOVE_THRESHOLD / 2.0
    ));
    assert!(same.hidden);
}

#[test]
fn slow_scroll_accumulates_past_threshold() {
    let mut state = scrolled_to(500.0);
    let step = super::MOVE_THRESHOLD / 4.0;
    for index in 1..=8 {
        state = state.update(at(500.0 + step * f64::from(index)));
    }
    assert!(state.hidden);
}

#[test]
fn stays_shown_near_top() {
    let state = scrolled_to(0.0).update(at(super::HIDE_AFTER));
    assert!(!state.hidden);
}

#[test]
fn top_bounce_shows() {
    let hidden = scrolled_to(500.0).update(at(500.0 + super::MOVE_THRESHOLD * 2.0));
    assert!(hidden.hidden);
    assert!(!hidden.update(at(-30.0)).hidden);
}

#[test]
fn bottom_bounce_does_not_toggle() {
    let hidden = scrolled_to(MAX - 100.0).update(at(MAX));
    assert!(hidden.hidden);
    let bounced = hidden.update(at(MAX + 40.0));
    assert!(bounced.hidden);
    let settled = bounced.update(at(MAX));
    assert!(settled.hidden);
}

#[test]
fn focus_inside_keeps_shown() {
    let hidden = scrolled_to(500.0).update(at(500.0 + super::MOVE_THRESHOLD * 2.0));
    assert!(hidden.hidden);
    let focused = hidden.update(ScrollMetrics {
        focus_within: true,
        ..at(500.0 + super::MOVE_THRESHOLD * 4.0)
    });
    assert!(!focused.hidden);
}

#[test]
fn no_scrollable_range_stays_shown() {
    let state = HeaderScroll::default().update(ScrollMetrics {
        top: 300.0,
        max: 0.0,
        focus_within: false,
    });
    assert!(!state.hidden);
}
