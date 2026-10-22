use super::port::{Event, ProgressState};
use super::*;

#[test]
fn kind_round_trip() {
    for kind in CsvKind::ALL {
        assert_eq!(
            CsvKind::from_request_value(kind.request_value()),
            Some(kind)
        );
    }
    assert_eq!(CsvKind::from_request_value("unknown"), None);
}

fn progress(state: ProgressState, kind: CsvKind) -> Event {
    Event::Progress {
        state,
        kind: Some(kind),
        reason: None,
    }
}

#[test]
fn progress_updates_per_kind_status() {
    let mut state = SyncState::initial();
    state.apply(Event::Progress {
        state: ProgressState::LoginWait,
        kind: None,
        reason: None,
    });
    assert!(state.login_wait);

    state.apply(progress(ProgressState::Fetching, CsvKind::Dividend));
    // 取得が始まったらログイン待ちの案内は消える
    assert!(!state.login_wait);
    assert_eq!(
        state.kinds[CsvKind::Dividend.index()].1,
        KindStatus::Fetching
    );

    state.apply(Event::Progress {
        state: ProgressState::Failed,
        kind: Some(CsvKind::Dividend),
        reason: Some("失敗した".to_string()),
    });
    assert_eq!(
        state.kinds[CsvKind::Dividend.index()].1,
        KindStatus::Failed {
            reason: "失敗した".to_string()
        }
    );
}

#[test]
fn result_marks_done_with_line_count_and_failures() {
    let mut state = SyncState::initial();
    state.apply(Event::Result {
        csvs: vec![ReceivedCsv {
            kind: CsvKind::AssetBalance,
            content: "表題\nヘッダ\n行1\n行2\n".to_string(),
            fetched_at: "2026-10-23T00:00:00Z".to_string(),
        }],
        failures: vec![FetchFailure {
            kind: CsvKind::Dividend,
            reason: "許可されていません".to_string(),
            code: Some("file-access-disabled".to_string()),
        }],
    });
    assert!(!state.running && state.finished);
    assert_eq!(
        state.kinds[CsvKind::AssetBalance.index()].1,
        KindStatus::Done { lines: 4 }
    );
    assert_eq!(
        state.kinds[CsvKind::Dividend.index()].1,
        KindStatus::Failed {
            reason: "許可されていません".to_string()
        }
    );
    assert!(state.needs_file_access_guide());
}

#[test]
fn quoted_newlines_count_as_one_record() {
    let mut state = SyncState::initial();
    state.apply(Event::Result {
        csvs: vec![ReceivedCsv {
            kind: CsvKind::AssetBalance,
            content: "ヘッダ\n\"改行を\n含む\",1\n行3\n".to_string(),
            fetched_at: "2026-10-23T00:00:00Z".to_string(),
        }],
        failures: vec![],
    });
    assert_eq!(
        state.kinds[CsvKind::AssetBalance.index()].1,
        KindStatus::Done { lines: 3 }
    );
}

#[test]
fn store_keeps_received_csvs_for_import() {
    let owner = leptos::prelude::Owner::new();
    owner.with(|| {
        let store = CsvSyncStore::new();
        store.apply(Event::Result {
            csvs: vec![ReceivedCsv {
                kind: CsvKind::Dividend,
                content: "ヘッダ\n行1\n".to_string(),
                fetched_at: "2026-10-23T00:00:00Z".to_string(),
            }],
            failures: vec![],
        });
        let received = store.received();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].kind, CsvKind::Dividend);
        assert!(!store.state().running);
    });
}

#[test]
fn terminal_events_stop_running() {
    for (event, missing, error_is_some) in [
        (Event::ErrorMessage("拒否".to_string()), false, true),
        (Event::Disconnected, false, true),
        (Event::Unavailable, true, false),
    ] {
        let mut state = SyncState::initial();
        state.apply(event);
        assert!(!state.running);
        assert_eq!(state.missing, missing);
        assert_eq!(state.error.is_some(), error_is_some);
    }
}
