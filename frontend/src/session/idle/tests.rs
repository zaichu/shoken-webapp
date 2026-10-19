use super::*;

const TIMEOUT: u64 = 30 * MS_PER_MINUTE as u64;

#[test]
fn configured_minutes_convert_to_ms() {
    assert_eq!(timeout_ms(RwSignal::new(15)), 900_000);
    assert_eq!(timeout_ms(RwSignal::new(30)), 1_800_000);
    assert_eq!(timeout_ms(RwSignal::new(60)), 3_600_000);
}

#[test]
fn idle_remaining_hits_zero_at_deadline() {
    assert_eq!(idle_remaining_ms(0, 0, TIMEOUT), TIMEOUT as i64);
    assert_eq!(idle_remaining_ms(0, TIMEOUT - 1, TIMEOUT), 1);
    assert_eq!(idle_remaining_ms(0, TIMEOUT, TIMEOUT), 0);
    assert_eq!(idle_remaining_ms(0, TIMEOUT + 1_000, TIMEOUT), 0);
    assert_eq!(idle_remaining_ms(1_000, 1_000, TIMEOUT), TIMEOUT as i64);
}

#[test]
fn idle_remaining_is_capped_for_future_timestamps() {
    assert_eq!(
        idle_remaining_ms(TIMEOUT + 60_000, 0, TIMEOUT),
        TIMEOUT as i64
    );
    assert_eq!(idle_remaining_ms(u64::MAX - 1, 0, TIMEOUT), TIMEOUT as i64);
}

#[test]
fn idle_remaining_tracks_shorter_timeouts() {
    let fifteen = 15 * MS_PER_MINUTE as u64;
    assert_eq!(idle_remaining_ms(0, 0, fifteen), fifteen as i64);
    assert_eq!(idle_remaining_ms(0, TIMEOUT - 1, fifteen), 0);
}
