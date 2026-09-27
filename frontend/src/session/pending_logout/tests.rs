use super::*;
use std::cell::RefCell;

#[derive(Default)]
struct MemoryStore {
    value: RefCell<Option<String>>,
}

impl FlagStore for MemoryStore {
    fn is_set(&self) -> bool {
        self.value.borrow().is_some()
    }
    fn set(&self) {
        *self.value.borrow_mut() = Some("1".to_string());
    }
    fn clear(&self) {
        *self.value.borrow_mut() = None;
    }
}

struct BrokenStore;

impl FlagStore for BrokenStore {
    fn is_set(&self) -> bool {
        false
    }
    fn set(&self) {}
    fn clear(&self) {}
}

#[test]
fn pending_flag_roundtrip() {
    let store = MemoryStore::default();
    assert!(!is_pending_in(&store));
    mark_in(&store);
    assert!(is_pending_in(&store));
    clear_in(&store);
    assert!(!is_pending_in(&store));
}

#[test]
fn unavailable_storage_never_pending_and_never_panics() {
    let store = BrokenStore;
    mark_in(&store);
    assert!(!is_pending_in(&store));
    clear_in(&store);
    assert!(!is_pending_in(&store));
}

#[test]
fn retry_backoff_stretches_then_caps() {
    assert_eq!(retry_delay_ms(0), 10_000);
    assert_eq!(retry_delay_ms(1), 30_000);
    assert_eq!(retry_delay_ms(2), 60_000);
    assert_eq!(retry_delay_ms(3), 60_000);
    assert_eq!(retry_delay_ms(100), 60_000);
}

#[test]
fn finished_means_ok_or_missing_session() {
    assert!(is_finished(&Ok(())));
    assert!(is_finished(&Err(ApiError::http(401))));
    assert!(!is_finished(&Err(ApiError::http(403))));
    assert!(!is_finished(&Err(ApiError::http(500))));
    assert!(!is_finished(&Err(ApiError::Network)));
    assert!(!is_finished(&Err(ApiError::Timeout)));
    assert!(!is_finished(&Err(ApiError::Parse)));
}
