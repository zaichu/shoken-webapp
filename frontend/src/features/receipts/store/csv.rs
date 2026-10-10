use super::ReceiptsStore;
use crate::api::ApiError;
use crate::api::dto::{CsvPreviewResponse, CsvUploadResponse};
use crate::features::receipts::csv::{CSV_CHUNKING, CsvPreviewRow, fetch_preview};
use crate::features::receipts::{ReceiptTabData, ReceiptsTab, TabState};
use crate::session::Generation;
use crate::support::csv_flow::{CsvTabMeta, CsvTabState};
use leptos::prelude::*;

impl ReceiptsStore {
    pub fn csv_state(&self, tab: ReceiptsTab) -> CsvTabState<CsvPreviewRow> {
        let generation = self.session.generation.get();
        self.tabs.with(|map| {
            map.get(&(generation, tab))
                .map(|entry| entry.csv.clone())
                .unwrap_or_default()
        })
    }

    // 共通 CSV UI が読むのはメタ情報だけ。プレビュー行の複製を避ける
    pub fn csv_meta(&self, tab: ReceiptsTab) -> CsvTabMeta {
        let generation = self.session.generation.get();
        self.tabs.with(|map| {
            map.get(&(generation, tab))
                .map(|entry| entry.csv.meta())
                .unwrap_or_default()
        })
    }

    pub fn csv_busy(&self, tab: ReceiptsTab) -> bool {
        let generation = self.session.generation.get();
        self.tabs.with(|map| {
            map.get(&(generation, tab))
                .is_some_and(|entry| entry.csv.busy())
        })
    }

    // ファイル入力を押せない間は空状態 CTA 経由の選択も効かないので、両者は同じ条件にする
    pub fn csv_input_disabled(&self, tab: ReceiptsTab) -> bool {
        !self.is_authenticated()
            || self.auth_loading()
            || self.csv_busy(tab)
            || self.any_tab_fetching()
    }

    pub fn has_csv_preview(&self, tab: ReceiptsTab) -> bool {
        let generation = self.session.generation.get();
        self.tabs.with(|map| {
            map.get(&(generation, tab))
                .is_some_and(|entry| entry.csv.has_preview_rows())
        })
    }

    fn update_csv_state(
        &self,
        generation: Generation,
        tab: ReceiptsTab,
        update: impl FnOnce(&mut CsvTabState<CsvPreviewRow>),
    ) {
        self.tabs
            .update(|map| update(&mut map.entry((generation, tab)).or_default().csv));
    }

    pub fn select_file(&self, tab: ReceiptsTab, file: web_sys::File) {
        let Some(generation) = self.current_generation() else {
            return;
        };
        let mut started = false;
        self.update_csv_state(generation, tab, |state| {
            started = state.begin_preview(file.name());
        });
        if !started {
            return;
        }
        self.tabs.update(|map| {
            map.entry((generation, tab)).or_default().csv_file = Some(file.clone());
        });
        let store = *self;
        leptos::task::spawn_local(async move {
            let result = fetch_preview(tab, &file).await;
            store.apply_preview_result(generation, tab, result);
        });
    }

    pub fn save_csv(&self, tab: ReceiptsTab) {
        let Some((generation, file)) = self.try_begin_save(tab) else {
            return;
        };
        let store = *self;
        leptos::task::spawn_local(async move {
            let result =
                crate::support::csv_flow::upload_csv(tab.import_path(), &file, CSV_CHUNKING).await;
            if store.apply_upload_result(generation, tab, result) {
                store.fetch.dispatch((generation, tab));
            }
        });
    }

    pub(crate) fn try_begin_save(&self, tab: ReceiptsTab) -> Option<(Generation, web_sys::File)> {
        let generation = self.current_generation()?;
        let file = self
            .tabs
            .with_untracked(|map| map.get(&(generation, tab)).and_then(|e| e.csv_file.clone()))?;
        let mut started = false;
        self.update_csv_state(generation, tab, |state| {
            started = state.begin_save();
        });
        started.then_some((generation, file))
    }

    pub fn open_delete_confirm(&self, tab: ReceiptsTab) {
        if let Some(generation) = self.current_generation() {
            self.update_csv_state(generation, tab, |state| state.open_delete_confirm());
        }
    }

    pub fn close_delete_confirm(&self, tab: ReceiptsTab) {
        if let Some(generation) = self.current_generation() {
            self.update_csv_state(generation, tab, |state| state.close_delete_confirm());
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

    pub(crate) fn apply_preview_result(
        &self,
        generation: Generation,
        tab: ReceiptsTab,
        result: Result<CsvPreviewResponse<CsvPreviewRow>, ApiError>,
    ) {
        if !self.session.is_current(generation) {
            return;
        }
        self.update_csv_state(generation, tab, |state| {
            state.finish_preview(result.ok());
        });
    }

    // 一覧の再取得が必要になったら true を返す。fetch の起動(dispatch)は呼び出し側が行う
    pub(crate) fn apply_upload_result(
        &self,
        generation: Generation,
        tab: ReceiptsTab,
        result: Result<CsvUploadResponse, ApiError>,
    ) -> bool {
        if !self.session.is_current(generation) {
            return false;
        }
        match result {
            Ok(response) => {
                self.tabs.update(|map| {
                    let entry = map.entry((generation, tab)).or_default();
                    entry.csv.finish_save(Ok(response));
                    entry.csv_file = None;
                });
                self.refresh_tab_list(generation, tab)
            }
            Err(error) => {
                let message = error.message();
                self.update_csv_state(generation, tab, |state| {
                    state.finish_save(Err(message));
                });
                false
            }
        }
    }

    pub(crate) fn apply_delete_result(
        &self,
        generation: Generation,
        tab: ReceiptsTab,
        result: Result<(), ApiError>,
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
                            summary: None,
                            truncated: false,
                        }));
                    }
                });
            }
            Err(error) => {
                let message = error.message();
                self.update_csv_state(generation, tab, |state| {
                    state.finish_delete(Err(message));
                });
            }
        }
    }
}
