use super::ReceiptsStore;
use crate::features::receipts::{ReceiptTabData, ReceiptsTab, TabState};
use crate::session::Generation;
use crate::support::csv_flow::{CsvTabMeta, CsvTabState};
use leptos::prelude::*;

impl ReceiptsStore {
    // 共通 CSV UI が読むのはメタ情報だけ
    pub fn csv_meta(&self, tab: ReceiptsTab) -> CsvTabMeta {
        let generation = self.session.generation.get();
        self.tabs.with(|map| {
            map.get(&(generation, tab))
                .map(|entry| entry.csv.meta())
                .unwrap_or_default()
        })
    }

    fn update_csv_state(
        &self,
        generation: Generation,
        tab: ReceiptsTab,
        update: impl FnOnce(&mut CsvTabState),
    ) {
        self.tabs
            .update(|map| update(&mut map.entry((generation, tab)).or_default().csv));
    }

    pub fn open_delete_confirm(&self, tab: ReceiptsTab) {
        if let Some(generation) = self.current_generation() {
            self.update_csv_state(generation, tab, |state| state.open_delete_confirm());
        }
    }

    pub fn confirm_delete_all(&self, tab: ReceiptsTab) {
        let Some(generation) = self.try_begin_delete(tab) else {
            return;
        };
        let store = *self;
        leptos::task::spawn_local(async move {
            let result = crate::support::csv_flow::delete_all(tab.list_path()).await;
            store.apply_delete_result(generation, tab, result);
        });
    }

    pub(crate) fn try_begin_delete(&self, tab: ReceiptsTab) -> Option<Generation> {
        let generation = self.current_generation()?;
        let mut started = false;
        self.update_csv_state(generation, tab, |state| {
            started = state.begin_delete();
        });
        started.then_some(generation)
    }

    pub(crate) fn apply_delete_result(
        &self,
        generation: Generation,
        tab: ReceiptsTab,
        result: Result<(), crate::api::ApiError>,
    ) {
        if !self.session.is_current(generation) {
            return;
        }
        match result {
            Ok(()) => {
                self.tabs.update(|map| {
                    let entry = map.entry((generation, tab)).or_default();
                    entry.csv.finish_delete(Ok(()));
                    // 削除前に出た裏再取得の遅れ応答が消した行を復活させないよう取得を失効させる
                    entry.fetch_rev += 1;
                    // 空になった一覧の上に直前の再取得エラーが残るのを防ぐ
                    entry.refresh_error = None;
                    // 未取得タブに空の Ready を作ると以後の再取得が抑止されるため、キャッシュ済みの時だけ上書き
                    if entry.list.is_some() {
                        entry.list = Some(TabState::Ready(ReceiptTabData {
                            rows: Vec::new(),
                            total: Some(0),
                            summary: None,
                            truncated: false,
                        }));
                    }
                });
            }
            Err(error) => {
                self.update_csv_state(generation, tab, |state| {
                    state.set_delete_result(Err(error));
                });
            }
        }
    }
}
