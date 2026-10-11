use super::{
    AssetCsvFileSlot, AssetCsvSlot, BalanceSlot, DataOps, LoadedAssetBalances, load_asset_balances,
    poll_dividend_maps, truncated_list_warning,
};
use crate::api::ApiError;
use crate::api::dto::{AssetBalanceSummary, CsvPreviewResponse, CsvUploadResponse};
use crate::features::asset_balance::csv::{self, AssetBalanceCsvRow, AssetBalanceRow};
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::dividend_per_share::{DividendMaps, unique_sorted_codes};
use crate::session::{Generation, SessionStore};
use crate::support::csv_flow::{CsvTabMeta, CsvTabState};
use crate::support::row::Row;
use leptos::prelude::*;

// 一覧キャッシュと同じく世代で区切り、ログアウト・ユーザー切替で自動的に無効化する
#[derive(Clone, Copy)]
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

    // 共通 CSV UI が読むのはメタ情報だけ。プレビュー行の複製を避ける
    pub(crate) fn csv_meta(&self) -> CsvTabMeta {
        let generation = self.session.generation.get();
        self.csv.with(|slot| match slot {
            Some((cached, state)) if *cached == generation => state.meta(),
            _ => CsvTabMeta::default(),
        })
    }

    pub(crate) fn csv_busy(&self) -> bool {
        let generation = self.session.generation.get();
        self.csv.with(|slot| match slot {
            Some((cached, state)) if *cached == generation => state.busy(),
            _ => false,
        })
    }

    // ファイル入力を押せない間は空状態 CTA 経由の選択も効かないので、両者は同じ条件にする
    pub(crate) fn csv_input_disabled(&self) -> bool {
        !self.is_authenticated() || self.csv_busy() || self.list_loading()
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

    // 削除ページで保存済み全件数を確かめてから実行するため、件数が確定しているかを返す。
    // 一覧の取り直し中・取り直し失敗時は古い件数を未確定として扱う
    pub(crate) fn saved_count_ready(&self) -> bool {
        let generation = self.session.generation.get();
        let fetch_unresolved = self
            .data_ops
            .with(|ops| ops.list_fetch_in_flight() || ops.refresh_error.is_some());
        !fetch_unresolved
            && self
                .balances
                .with(|slot| matches!(slot, Some((cached, Ok(_))) if *cached == generation))
    }

    // 件数が確定できない理由(一覧取得・取り直しの失敗)を削除ページに出す
    pub(crate) fn saved_count_error(&self) -> Option<String> {
        let generation = self.session.generation.get();
        self.balances.with(|slot| match slot {
            Some((cached, Err(message))) if *cached == generation => Some(message.clone()),
            _ => self.data_ops.with(|ops| ops.refresh_error.clone()),
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

    fn current_generation(&self) -> Option<Generation> {
        self.session.user.get_untracked()?;
        Some(self.session.generation.get_untracked())
    }

    pub(crate) fn update_csv(
        &self,
        generation: Generation,
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
        let Some(generation) = self.current_generation() else {
            return;
        };
        if !self.begin_file_preview(generation, file.name()) {
            return;
        }
        self.csv_file.set(Some((generation, file.clone())));
        let store = *self;
        leptos::task::spawn_local(async move {
            let result =
                crate::support::csv_flow::preview_csv(csv::PREVIEW_PATH, &file, csv::CSV_CHUNKING)
                    .await;
            if let Some(codes) = store.apply_preview_result(generation, result)
                && !codes.is_empty()
            {
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
        });
    }

    // 別CSVを選び直した場合、旧銘柄向けのポーリング結果と配当マップが残らないよう無効化する
    pub(crate) fn begin_file_preview(&self, generation: Generation, file_name: String) -> bool {
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
        let store = *self;
        leptos::task::spawn_local(async move {
            let result =
                crate::support::csv_flow::upload_csv(csv::IMPORT_PATH, &file, csv::CSV_CHUNKING)
                    .await;
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

    fn try_begin_save(&self) -> Option<(Generation, web_sys::File)> {
        let generation = self.current_generation()?;
        let file = self.csv_file.with_untracked(|slot| match slot {
            Some((cached, file)) if *cached == generation => Some(file.clone()),
            _ => None,
        })?;
        let mut started = false;
        self.update_csv(generation, |state| {
            started = state.has_preview_rows() && state.begin_save();
        });
        started.then_some((generation, file))
    }

    pub(crate) fn discard_csv(&self) {
        let Some(generation) = self.current_generation() else {
            return;
        };
        let mut repoll_dividends = false;
        self.update_csv(generation, |state| {
            // 配当マップを消したのはファイル選択時なので、選択があった時だけ取り直す
            repoll_dividends = state.file_name.is_some() && state.discard();
        });
        if !repoll_dividends {
            return;
        }
        self.csv_file.set(None);
        // プレビュー用に消して埋め直した配当マップを、破棄後の保存済み一覧向けに取り直す
        self.data_ops.update(DataOps::next_poll_rev);
        let codes = self.balances.with_untracked(|slot| match slot {
            Some((cached, Ok(loaded))) if *cached == generation => unique_sorted_codes(
                &loaded
                    .rows
                    .iter()
                    .map(|row| row.security_code.to_string())
                    .collect::<Vec<_>>(),
            ),
            _ => Vec::new(),
        });
        if codes.is_empty() {
            self.dividends.set(DividendMaps::default());
            return;
        }
        let poll_rev = self.data_ops.with_untracked(|ops| ops.poll_rev);
        let store = *self;
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

    pub(crate) fn open_delete_confirm(&self) {
        if let Some(generation) = self.current_generation() {
            self.update_csv(generation, |state| state.open_delete_confirm());
        }
    }

    pub(crate) fn confirm_delete_all(&self) {
        let Some(generation) = self.try_begin_delete() else {
            return;
        };
        let store = *self;
        leptos::task::spawn_local(async move {
            let result = crate::support::csv_flow::delete_all(csv::LIST_PATH).await;
            store.apply_delete_result(generation, result);
        });
    }

    pub(crate) fn try_begin_delete(&self) -> Option<Generation> {
        let generation = self.current_generation()?;
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
        generation: Generation,
        result: Result<CsvPreviewResponse<AssetBalanceCsvRow>, ApiError>,
    ) -> Option<Vec<String>> {
        if !self.session.is_current(generation) {
            return None;
        }
        match result {
            Ok(preview) => {
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
                let message = error.message();
                self.update_csv(generation, |state| state.fail_preview(message));
                None
            }
        }
    }

    pub(crate) fn apply_upload_result(
        &self,
        generation: Generation,
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
                let message = error.message();
                self.update_csv(generation, |state| state.finish_save(Err(message)));
                false
            }
        }
    }

    pub(crate) fn apply_delete_result(&self, generation: Generation, result: Result<(), ApiError>) {
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
                        truncated: false,
                    }),
                )));
                self.dividends.set(DividendMaps::default());
                self.lookup.update(|store| store.clear());
            }
            Err(error) => {
                let message = error.message();
                self.update_csv(generation, |state| state.finish_delete(Err(message)));
            }
        }
    }
}

pub(crate) fn csv_preview_rows(state: &CsvTabState<AssetBalanceCsvRow>) -> Vec<AssetBalanceRow> {
    state
        .preview
        .as_ref()
        .map(|preview| preview.rows.iter().cloned().map(Row::Preview).collect())
        .unwrap_or_default()
}

pub(crate) struct ResolvedAssetBalance {
    pub(crate) rows: Vec<AssetBalanceRow>,
    pub(crate) warning: Option<String>,
    /// 取得上限超過で行が欠けるときの API 集計。行が揃う通常時は None
    pub(crate) fallback_totals: Option<AssetBalanceSummary>,
}

pub(crate) fn resolve_asset_balance(
    generation: Generation,
    slot: &BalanceSlot,
) -> Option<ResolvedAssetBalance> {
    let (_, result) = slot.as_ref().filter(|(cached, _)| *cached == generation)?;
    match result {
        Err(_) => Some(ResolvedAssetBalance {
            rows: Vec::new(),
            warning: None,
            fallback_totals: None,
        }),
        Ok(loaded) => Some(ResolvedAssetBalance {
            rows: loaded.rows.iter().cloned().map(Row::Saved).collect(),
            warning: loaded.truncated.then(truncated_list_warning),
            fallback_totals: if loaded.truncated {
                loaded.summary.clone()
            } else {
                None
            },
        }),
    }
}

#[cfg(test)]
mod tests;
