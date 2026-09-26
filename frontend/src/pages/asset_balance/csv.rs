use super::data::{
    load_asset_balances, poll_dividend_maps, truncated_list_warning, AssetCsvFileSlot,
    AssetCsvSlot, BalanceSlot, DataOps, LoadedAssetBalances,
};
use crate::api::ApiError;
use crate::asset_balance::csv::{self, AssetBalanceCsvRow};
use crate::asset_balance::lookup::AssetBalanceLookupStore;
use crate::csv_flow::{csv_error_message, CsvTabState};
use crate::dividend_per_share::{unique_sorted_codes, DividendMaps};
use crate::dto::{
    AssetBalance, AssetBalanceSummary, CsvPreviewResponse, CsvUploadResponse, SearchFacets,
};
use crate::session::SessionStore;
use leptos::prelude::*;

pub(crate) fn can_save_csv(state: &CsvTabState<AssetBalanceCsvRow>) -> bool {
    state
        .preview
        .as_ref()
        .is_some_and(|preview| !preview.rows.is_empty())
}

// 一覧キャッシュと同じく世代で区切り、ログアウト・ユーザー切替で自動的に無効化する
#[derive(Clone)]
pub(crate) struct AssetBalanceCsvStore {
    session: SessionStore,
    pub(crate) balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    pub(crate) csv: RwSignal<AssetCsvSlot>,
    pub(crate) csv_file: RwSignal<AssetCsvFileSlot>,
    pub(crate) data_ops: RwSignal<DataOps>,
}

impl AssetBalanceCsvStore {
    pub(crate) fn new(
        session: SessionStore,
        balances: RwSignal<BalanceSlot>,
        dividends: RwSignal<DividendMaps>,
        lookup: RwSignal<AssetBalanceLookupStore>,
        data_ops: RwSignal<DataOps>,
    ) -> Self {
        Self {
            session,
            balances,
            dividends,
            lookup,
            csv: RwSignal::new(None),
            csv_file: RwSignal::new(None),
            data_ops,
        }
    }

    pub(crate) fn csv_state(&self) -> CsvTabState<AssetBalanceCsvRow> {
        let generation = self.session.generation.get();
        self.csv.with(|slot| match slot {
            Some((cached, state)) if *cached == generation => state.clone(),
            _ => CsvTabState::default(),
        })
    }

    pub(crate) fn csv_busy(&self) -> bool {
        self.csv_state().busy()
    }

    pub(crate) fn is_authenticated(&self) -> bool {
        self.session.user.get().is_some()
    }

    // 削除件数はプレビューではなく DB 側の total を使う
    pub(crate) fn db_count(&self) -> usize {
        let generation = self.session.generation.get();
        self.balances.with(|slot| match slot {
            Some((cached, Ok(loaded))) if *cached == generation => loaded.total,
            _ => 0,
        })
    }

    pub(crate) fn list_loading(&self) -> bool {
        let generation = self.session.generation.get();
        self.session.user.get().is_some()
            && (self.data_ops.with(|ops| !ops.inflight.is_empty())
                || self
                    .balances
                    .with(|slot| !matches!(slot, Some((cached, _)) if *cached == generation)))
    }

    pub(crate) fn update_csv(
        &self,
        generation: u64,
        update: impl FnOnce(&mut CsvTabState<AssetBalanceCsvRow>),
    ) {
        self.csv.update(|slot| {
            if !matches!(slot, Some((cached, _)) if *cached == generation) {
                *slot = Some((generation, CsvTabState::default()));
            }
            if let Some((_, state)) = slot {
                update(state);
            }
        });
    }

    pub(crate) fn select_file(&self, file: web_sys::File) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        if !self.begin_file_preview(generation, file.name()) {
            return;
        }
        self.csv_file.set(Some((generation, file.clone())));
        let store = self.clone();
        leptos::task::spawn_local(async move {
            let result = crate::csv_flow::preview_csv(csv::PREVIEW_PATH, &file).await;
            if let Some(codes) = store.apply_preview_result(generation, result) {
                if !codes.is_empty() {
                    store.data_ops.update(DataOps::next_poll_rev);
                    let poll_rev = store.data_ops.with_untracked(|ops| ops.poll_rev);
                    leptos::task::spawn_local(poll_dividend_maps(
                        store.session,
                        generation,
                        codes,
                        store.balances,
                        store.dividends,
                        store.data_ops,
                        poll_rev,
                    ));
                }
            }
        });
    }

    // 別CSVを選び直した場合、旧銘柄向けのポーリング結果と配当マップが残らないよう無効化する
    pub(crate) fn begin_file_preview(&self, generation: u64, file_name: String) -> bool {
        let mut started = false;
        self.update_csv(generation, |state| {
            started = state.begin_preview(file_name);
        });
        if started {
            self.data_ops.update(DataOps::next_poll_rev);
            self.dividends.set(DividendMaps::default());
        }
        started
    }

    pub(crate) fn save_csv(&self) {
        let Some((generation, file)) = self.try_begin_save() else {
            return;
        };
        let store = self.clone();
        leptos::task::spawn_local(async move {
            let result = crate::csv_flow::upload_csv(csv::IMPORT_PATH, &file).await;
            if store.apply_upload_result(generation, result) {
                load_asset_balances(
                    store.session,
                    generation,
                    store.balances,
                    store.dividends,
                    store.lookup,
                    store.csv,
                    store.data_ops,
                );
            }
        });
    }

    fn try_begin_save(&self) -> Option<(u64, web_sys::File)> {
        self.session.user.get_untracked()?;
        let generation = self.session.generation.get_untracked();
        let file = self.csv_file.with_untracked(|slot| match slot {
            Some((cached, file)) if *cached == generation => Some(file.clone()),
            _ => None,
        })?;
        let mut started = false;
        self.update_csv(generation, |state| {
            started = can_save_csv(state) && state.begin_save();
        });
        started.then_some((generation, file))
    }

    pub(crate) fn open_delete_confirm(&self) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        self.update_csv(generation, |state| state.open_delete_confirm());
    }

    pub(crate) fn close_delete_confirm(&self) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        self.update_csv(generation, |state| state.close_delete_confirm());
    }

    pub(crate) fn confirm_delete_all(&self) {
        let Some(generation) = self.try_begin_delete() else {
            return;
        };
        let store = self.clone();
        leptos::task::spawn_local(async move {
            let result = crate::csv_flow::delete_all(csv::LIST_PATH).await;
            store.apply_delete_result(generation, result);
        });
    }

    pub(crate) fn try_begin_delete(&self) -> Option<u64> {
        self.session.user.get_untracked()?;
        let generation = self.session.generation.get_untracked();
        let mut started = false;
        self.update_csv(generation, |state| {
            started = state.begin_delete();
        });
        if started {
            // 先に走った一覧取得の遅れ結果が削除後の一覧を復活させないよう無効化する
            self.data_ops.update(|ops| ops.list_rev += 1);
        }
        started.then_some(generation)
    }

    // 失敗は取引明細と違って画面に出す
    pub(crate) fn apply_preview_result(
        &self,
        generation: u64,
        result: Result<CsvPreviewResponse, ApiError>,
    ) -> Option<Vec<String>> {
        if !self.session.is_current(generation) {
            return None;
        }
        match result {
            Ok(response) => {
                let preview = csv::to_preview(response);
                let codes = unique_sorted_codes(
                    &preview
                        .rows
                        .iter()
                        .map(|row| row.security_code.clone())
                        .collect::<Vec<_>>(),
                );
                self.update_csv(generation, |state| {
                    state.finish_preview(Some(preview));
                });
                Some(codes)
            }
            Err(error) => {
                let message = csv_error_message(&error);
                self.update_csv(generation, |state| state.fail_preview(message));
                None
            }
        }
    }

    pub(crate) fn apply_upload_result(
        &self,
        generation: u64,
        result: Result<CsvUploadResponse, ApiError>,
    ) -> bool {
        if !self.session.is_current(generation) {
            return false;
        }
        match result {
            Ok(response) => {
                self.update_csv(generation, |state| state.finish_save(Ok(response)));
                self.csv_file.set(None);
                true
            }
            Err(error) => {
                let message = csv_error_message(&error);
                self.update_csv(generation, |state| state.finish_save(Err(message)));
                false
            }
        }
    }

    pub(crate) fn apply_delete_result(&self, generation: u64, result: Result<(), ApiError>) {
        if !self.session.is_current(generation) {
            return;
        }
        match result {
            Ok(()) => {
                self.update_csv(generation, |state| state.finish_delete(Ok(())));
                // 削除確定後に届く一覧取得・配当ポーリングの遅れ結果を捨てる
                self.data_ops.update(DataOps::invalidate);
                // DELETE 成功後は DB が空なので、未取得でも空を確定して読み込み表示を残さない
                self.balances.set(Some((
                    generation,
                    Ok(LoadedAssetBalances {
                        rows: Vec::new(),
                        total: 0,
                        summary: None,
                        facets: None,
                        truncated: false,
                    }),
                )));
                self.dividends.set(DividendMaps::default());
                self.lookup.update(|store| store.clear());
            }
            Err(error) => {
                let message = csv_error_message(&error);
                self.update_csv(generation, |state| state.finish_delete(Err(message)));
            }
        }
    }
}

pub(crate) fn csv_preview_rows(state: &CsvTabState<AssetBalanceCsvRow>) -> Vec<AssetBalance> {
    state
        .preview
        .as_ref()
        .map(|preview| {
            preview
                .rows
                .iter()
                .cloned()
                .map(AssetBalance::from)
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn csv_status_text(state: &CsvTabState<AssetBalanceCsvRow>) -> Option<&'static str> {
    if state.saving {
        Some("データを保存しています...")
    } else if state.deleting {
        Some("データを削除しています...")
    } else if state.previewing {
        Some("CSVファイルを解析しています...")
    } else {
        None
    }
}

pub(crate) struct ResolvedAssetBalance {
    pub(crate) rows: Vec<AssetBalance>,
    pub(crate) summary: Option<AssetBalanceSummary>,
    pub(crate) facets: Option<SearchFacets>,
    pub(crate) warning: Option<String>,
    pub(crate) has_csv_file: bool,
}

pub(crate) fn resolve_asset_balance(
    generation: u64,
    slot: &BalanceSlot,
    state: &CsvTabState<AssetBalanceCsvRow>,
) -> Option<ResolvedAssetBalance> {
    let (_, result) = slot.as_ref().filter(|(cached, _)| *cached == generation)?;
    let has_csv_file = state.file_name.is_some();
    match result {
        Err(_) => Some(ResolvedAssetBalance {
            rows: csv_preview_rows(state),
            summary: None,
            facets: None,
            warning: None,
            has_csv_file,
        }),
        Ok(loaded) => Some(ResolvedAssetBalance {
            rows: if has_csv_file {
                csv_preview_rows(state)
            } else {
                loaded.rows.clone()
            },
            summary: loaded.summary.clone(),
            facets: loaded.facets.clone(),
            warning: (!has_csv_file && loaded.truncated).then(truncated_list_warning),
            has_csv_file,
        }),
    }
}
