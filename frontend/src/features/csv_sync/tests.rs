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
    assert!(state.finished);
    // 取り込みが残る間は実行中を維持し、連打による重複送信を防ぐ
    assert!(state.running);
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
fn result_without_csvs_stops_running() {
    let mut state = SyncState::initial();
    state.apply(Event::Result {
        csvs: vec![],
        failures: vec![],
    });
    assert!(state.finished);
    assert!(!state.running);
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
fn result_csvs_are_marked_done_and_handled_to_import_task() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = leptos::prelude::Owner::new();
    owner.with(|| {
        let store = CsvSyncStore::new();
        store.state.set(SyncState::initial());
        store.apply(Event::Result {
            csvs: vec![ReceivedCsv {
                kind: CsvKind::Dividend,
                content: "入金日,商品,口座,銘柄\n2026/10/01,国内株式,特定,秘密銘柄\n".to_string(),
                fetched_at: "2026-10-23T00:00:00Z".to_string(),
            }],
            failures: vec![],
        });
        let state = store.state();
        assert!(state.finished);
        // 取り込みタスクが残る間は実行中のまま(ボタン連打で二重に流さない)
        assert!(state.running);
        // 本文は表示状態に載せず、取り込みタスクへ渡す(タスクは executor 待ちのままでよい)
        assert_eq!(
            state.kinds[CsvKind::Dividend.index()].1,
            KindStatus::Done { lines: 2 }
        );
    });
}

#[test]
fn finish_import_releases_running() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = leptos::prelude::Owner::new();
    owner.with(|| {
        let store = CsvSyncStore::new();
        store.state.set(SyncState::initial());
        store.finish_import();
        // ページ未訪問なら再取得は何もせず、実行中だけ解除する
        assert!(!store.state().running);
        // ファイル取り込みの完了でも統計を取り直せるよう完了を立てる
        assert!(store.state().finished);
    });
}

#[test]
fn stats_refresh_settles_only_imported_statuses() {
    let owner = leptos::prelude::Owner::new();
    owner.with(|| {
        let store = CsvSyncStore::new();
        let mut state = SyncState::initial();
        state.set_kind(
            CsvKind::AssetBalance,
            KindStatus::Imported {
                result: crate::api::dto::CsvUploadResponse {
                    inserted: 1,
                    skipped: 0,
                    errors: vec![],
                },
                fetched_at: "2026-10-26T01:02:00Z".to_string(),
            },
        );
        state.set_kind(
            CsvKind::Dividend,
            KindStatus::Failed {
                reason: "取得失敗".to_string(),
            },
        );
        store.state.set(state);

        store.settle_imported_with_stats();

        let state = store.state();
        assert_eq!(
            state.kinds[CsvKind::AssetBalance.index()].1,
            KindStatus::Waiting
        );
        assert_eq!(
            state.kinds[CsvKind::Dividend.index()].1,
            KindStatus::Failed {
                reason: "取得失敗".to_string()
            }
        );
    });
}

#[test]
fn confirm_answer_no_skips_kind_and_clears_confirm() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = leptos::prelude::Owner::new();
    owner.with(|| {
        let store = CsvSyncStore::new();
        let csv = ReceivedCsv {
            kind: CsvKind::AssetBalance,
            content: "銘柄コード,銘柄名\n9901,架空\n".to_string(),
            fetched_at: "2026-10-23T00:00:00Z".to_string(),
        };
        store.state.set(SyncState::initial());
        store.hold_confirm(import::PendingImport {
            csv: csv.clone(),
            new_rows: 10,
            saved_rows: 40,
            rest: vec![csv],
            epoch: store.epoch.get_untracked(),
            generation: store.session.generation.get_untracked(),
        });
        let state = store.state();
        assert_eq!(
            state.kinds[CsvKind::AssetBalance.index()].1,
            KindStatus::Confirming
        );
        assert_eq!(
            state.import.confirm,
            Some(PlungeConfirm {
                kind: CsvKind::AssetBalance,
                new_rows: 10,
                saved_rows: 40,
            })
        );

        store.answer_confirm(false);
        let state = store.state();
        assert_eq!(state.import.confirm, None);
        assert_eq!(
            state.kinds[CsvKind::AssetBalance.index()].1,
            KindStatus::Skipped {
                reason: SkipReason::Declined
            }
        );
    });
}

#[test]
fn stale_epoch_confirm_is_ignored() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = leptos::prelude::Owner::new();
    owner.with(|| {
        let store = CsvSyncStore::new();
        store.state.set(SyncState::initial());
        // 前の実行(別世代)から遅れて届いた確認は表示しない
        store.hold_confirm(import::PendingImport {
            csv: ReceivedCsv {
                kind: CsvKind::FundBalance,
                content: String::new(),
                fetched_at: String::new(),
            },
            new_rows: 1,
            saved_rows: 10,
            rest: vec![],
            epoch: store.epoch.get_untracked() + 1,
            generation: store.session.generation.get_untracked(),
        });
        let state = store.state();
        assert_eq!(state.import.confirm, None);
        assert_eq!(
            state.kinds[CsvKind::FundBalance.index()].1,
            KindStatus::Waiting
        );
        // 回答しても pending が無いので何も起きない
        store.answer_confirm(false);
        assert_eq!(
            store.state().kinds[CsvKind::FundBalance.index()].1,
            KindStatus::Waiting
        );
    });
}

#[test]
fn stale_generation_confirm_is_ignored() {
    let _ = any_spawner::Executor::init_futures_executor();
    let owner = leptos::prelude::Owner::new();
    owner.with(|| {
        let store = CsvSyncStore::new();
        store.state.set(SyncState::initial());
        // ユーザー切替(セッション世代が進んだ)前の実行から届いた確認は出さない
        store.hold_confirm(import::PendingImport {
            csv: ReceivedCsv {
                kind: CsvKind::AssetBalance,
                content: String::new(),
                fetched_at: String::new(),
            },
            new_rows: 1,
            saved_rows: 10,
            rest: vec![],
            epoch: store.epoch.get_untracked(),
            generation: store.session.generation.get_untracked().next(),
        });
        assert_eq!(store.state().import.confirm, None);
        assert_eq!(
            store.state().kinds[CsvKind::AssetBalance.index()].1,
            KindStatus::Waiting
        );
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

#[test]
fn prepare_file_import_initializes_kinds_and_blocks_concurrent_runs() {
    // 最新取得を一度も実行していない状態でも行の状態を書ける
    let mut state = SyncState::default();
    assert!(state.kinds.is_empty());
    state.prepare_file_import();
    assert!(state.running);
    assert_eq!(state.kinds.len(), CsvKind::ALL.len());
    state.set_kind(CsvKind::Dividend, KindStatus::Importing);
    assert_eq!(
        state.kinds[CsvKind::Dividend.index()].1,
        KindStatus::Importing
    );

    // 進行中の表示は残し、実行中だけ立て直す
    state.prepare_file_import();
    assert!(state.running);
    assert_eq!(
        state.kinds[CsvKind::Dividend.index()].1,
        KindStatus::Importing
    );
}
