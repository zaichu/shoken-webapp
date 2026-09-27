use super::*;

#[test]
fn activity_write_is_throttled() {
    assert!(should_record(u64::MAX, 0));
    assert!(should_record(ACTIVITY_THROTTLE_MS, 0));
    assert!(!should_record(ACTIVITY_THROTTLE_MS - 1, 0));
    assert!(should_record(30_000, 10_000));
    assert!(!should_record(19_999, 10_000));
}
