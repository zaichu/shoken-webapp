use super::*;

#[test]
fn begin_delete_requires_confirm_and_is_busy_gated() {
    let mut state = CsvTabState::default();
    assert!(!state.begin_delete());
    state.open_delete_confirm();
    assert!(state.begin_delete());
    assert!(!state.begin_delete());
    state.finish_delete(Ok(()));
    assert!(!state.deleting);
}

#[test]
fn finish_delete_keeps_error_on_failure() {
    let mut state = CsvTabState::default();
    state.open_delete_confirm();
    assert!(state.begin_delete());
    state.finish_delete(Err("x".to_string()));
    assert!(!state.deleting);
    assert_eq!(state.error.as_deref(), Some("x"));
}
