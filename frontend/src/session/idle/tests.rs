use super::*;

#[test]
fn idle_timeout_is_thirty_minutes() {
    assert_eq!(IDLE_TIMEOUT_MS, 1_800_000);
}

#[test]
fn idle_remaining_hits_zero_at_deadline() {
    let timeout = IDLE_TIMEOUT_MS as u64;
    assert_eq!(idle_remaining_ms(0, 0), timeout as i64);
    assert_eq!(idle_remaining_ms(0, timeout - 1), 1);
    assert_eq!(idle_remaining_ms(0, timeout), 0);
    assert_eq!(idle_remaining_ms(0, timeout + 1_000), 0);
    assert_eq!(idle_remaining_ms(1_000, 1_000), timeout as i64);
}

#[test]
fn idle_remaining_is_capped_for_future_timestamps() {
    let timeout = IDLE_TIMEOUT_MS as u64;
    assert_eq!(idle_remaining_ms(timeout + 60_000, 0), timeout as i64);
    assert_eq!(idle_remaining_ms(u64::MAX - 1, 0), timeout as i64);
}
