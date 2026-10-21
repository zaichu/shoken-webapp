use super::{
    AssetCsvFileSlot, AssetCsvSlot, BalanceSlot, DataOps, FundSlot, LoadedAssetBalances,
    LoadedFundBalances, LoadedRows, load_asset_balances, load_fund_balances, poll_dividend_maps,
    truncated_list_warning,
};
use crate::api::ApiError;
use crate::api::dto::{AssetBalanceSummary, CsvPreviewResponse, CsvUploadResponse};
use crate::features::asset_balance::csv::{self, AssetBalanceCsvRow, AssetBalanceRow};
use crate::features::asset_balance::format::format_price_date;
use crate::features::asset_balance::fund::{
    AssetKind, FundBalanceCsvRow, FundBalanceRow, detect_asset_csv_kind,
};
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
    pub(crate) funds: RwSignal<FundSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    pub(crate) csv: RwSignal<AssetCsvSlot>,
    pub(crate) csv_fund: RwSignal<FundCsvSlot>,
    // 直前に選んだ CSV の判別結果。保存先とプレビュー描画を切り替える
    pub(crate) csv_kind: RwSignal<AssetKind>,
    pub(crate) csv_file: RwSignal<AssetCsvFileSlot>,
    pub(crate) data_ops: RwSignal<DataOps>,
}

pub(crate) type FundCsvSlot = Option<(Generation, CsvTabState<FundBalanceCsvRow>)>;

type LoadedSlotValue<T, S> = Option<(Generation, Result<LoadedRows<T, S>, String>)>;

// 世代つきスロットから現世代の値だけを取り出す。古い世代は None として扱う
fn slot_value<T>(slot: &Option<(Generation, T)>, generation: Generation) -> Option<&T> {
    slot.as_ref()
        .filter(|(cached, _)| *cached == generation)
        .map(|(_, value)| value)
}

fn min_max_dates(dates: impl Iterator<Item = String>) -> Option<(String, String)> {
    let dates: Vec<String> = dates.collect();
    let min = dates.iter().min()?;
    let max = dates.iter().max()?;
    Some((format_price_date(min), format_price_date(max)))
}

impl AssetBalanceCsvStore {
    fn with_slot<T: Send + Sync + 'static, R>(
        &self,
        slot: RwSignal<Option<(Generation, T)>>,
        f: impl FnOnce(Option<&T>) -> R,
    ) -> R {
        let generation = self.session.generation.get();
        slot.with(|slot| f(slot_value(slot, generation)))
    }

    fn update_slot<T: Send + Sync + 'static>(
        &self,
        slot: RwSignal<Option<(Generation, CsvTabState<T>)>>,
        generation: Generation,
        update: impl FnOnce(&mut CsvTabState<T>),
    ) {
        slot.update(|slot| {
            if slot_value(slot, generation).is_none() {
                *slot = Some((generation, CsvTabState::default()));
            }
            if let Some((_, state)) = slot {
                update(state);
            }
        });
    }

    fn csv_tab_state<T: Clone + Default + Send + Sync + 'static>(
        &self,
        slot: RwSignal<Option<(Generation, CsvTabState<T>)>>,
    ) -> CsvTabState<T> {
        self.with_slot(slot, |slot| slot.cloned().unwrap_or_default())
    }

    fn csv_tab_meta<T: Send + Sync + 'static>(
        &self,
        slot: RwSignal<Option<(Generation, CsvTabState<T>)>>,
    ) -> CsvTabMeta {
        self.with_slot(slot, |slot| slot.map(CsvTabState::meta).unwrap_or_default())
    }

    fn loaded_total<T: Send + Sync + 'static, S: Send + Sync + 'static>(
        &self,
        slot: RwSignal<LoadedSlotValue<T, S>>,
    ) -> usize {
        self.with_slot(slot, |slot| {
            slot.and_then(|result| result.as_ref().ok())
                .map_or(0, |loaded| loaded.total)
        })
    }

    // 削除ページで保存済み全件数を確かめてから実行するため、件数が確定しているかを返す。
    // 一覧の取り直し中・取り直し失敗時は古い件数を未確定として扱う
    fn loaded_ready<T: Send + Sync + 'static, S: Send + Sync + 'static>(
        &self,
        slot: RwSignal<LoadedSlotValue<T, S>>,
    ) -> bool {
        let fetch_unresolved = self
            .data_ops
            .with(|ops| ops.list_fetch_in_flight() || ops.refresh_error.is_some());
        !fetch_unresolved && self.with_slot(slot, |slot| slot.is_some_and(|result| result.is_ok()))
    }

    // 件数が確定できない理由(一覧取得・取り直しの失敗)を削除ページに出す
    fn loaded_error<T: Send + Sync + 'static, S: Send + Sync + 'static>(
        &self,
        slot: RwSignal<LoadedSlotValue<T, S>>,
    ) -> Option<String> {
        self.with_slot(slot, |slot| match slot {
            Some(Err(message)) => Some(message.clone()),
            _ => self.data_ops.with(|ops| ops.refresh_error.clone()),
        })
    }

    pub(crate) fn new(
        session: SessionStore,
        balances: RwSignal<BalanceSlot>,
        funds: RwSignal<FundSlot>,
        dividends: RwSignal<DividendMaps>,
        lookup: RwSignal<AssetBalanceLookupStore>,
        data_ops: RwSignal<DataOps>,
    ) -> Self {
        Self {
            session,
            balances,
            funds,
            dividends,
            lookup,
            csv: RwSignal::new(None),
            csv_fund: RwSignal::new(None),
            csv_kind: RwSignal::new(AssetKind::Stock),
            csv_file: RwSignal::new(None),
            data_ops,
        }
    }

    pub(crate) fn csv_state(&self) -> CsvTabState<AssetBalanceCsvRow> {
        self.csv_tab_state(self.csv)
    }

    pub(crate) fn fund_csv_state(&self) -> CsvTabState<FundBalanceCsvRow> {
        self.csv_tab_state(self.csv_fund)
    }

    // 共通 CSV UI が読むのはメタ情報だけ。プレビュー行の複製を避ける。
    // 判別済みのファイルがある間はその種別の状態を返す
    pub(crate) fn csv_meta(&self) -> CsvTabMeta {
        if self.csv_kind.get() == AssetKind::Fund {
            self.csv_tab_meta(self.csv_fund)
        } else {
            self.csv_tab_meta(self.csv)
        }
    }

    // どちらかの取り込みが動いている間は新しい選択を受け付けない
    pub(crate) fn csv_busy(&self) -> bool {
        self.csv_state().busy() || self.fund_csv_state().busy()
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
        self.loaded_total(self.balances)
    }

    pub(crate) fn saved_count_ready(&self) -> bool {
        self.loaded_ready(self.balances)
    }

    pub(crate) fn saved_count_error(&self) -> Option<String> {
        self.loaded_error(self.balances)
    }

    pub(crate) fn list_loading(&self) -> bool {
        self.session.user.get().is_some()
            && (self.data_ops.with(|ops| !ops.inflight.is_empty())
                || self.with_slot(self.balances, |slot| slot.is_none()))
    }

    // ---- 投資信託の保存済み件数(削除カード用) ----

    pub(crate) fn fund_db_count(&self) -> usize {
        self.loaded_total(self.funds)
    }

    pub(crate) fn fund_saved_count_ready(&self) -> bool {
        self.loaded_ready(self.funds)
    }

    pub(crate) fn fund_saved_count_error(&self) -> Option<String> {
        self.loaded_error(self.funds)
    }

    /// 投資信託の基準日範囲(price_as_of)
    pub(crate) fn fund_date_range(&self) -> Option<(String, String)> {
        self.with_slot(self.funds, |slot| {
            slot.and_then(|result| result.as_ref().ok())
                .and_then(|loaded| {
                    min_max_dates(loaded.rows.iter().map(|row| row.price_as_of.to_string()))
                })
        })
    }

    /// 削除件数の再取得(投資信託側)
    pub(crate) fn refresh_fund_for_delete(&self) {
        let Some(generation) = self.current_generation() else {
            return;
        };
        self.data_ops.update(|ops| ops.fund_list_rev += 1);
        load_fund_balances(self.session, generation, self.funds, self.data_ops);
    }

    /// 保存済みデータの基準日範囲(最小・最大)。取得済み行から算出し、未取得なら None
    pub(crate) fn date_range(&self) -> Option<(String, String)> {
        self.with_slot(self.balances, |slot| {
            slot.and_then(|result| result.as_ref().ok())
                .and_then(|loaded| {
                    min_max_dates(
                        loaded
                            .rows
                            .iter()
                            .filter_map(|row| row.price_as_of.as_deref().map(str::to_string)),
                    )
                })
        })
    }

    /// 削除件数の再取得を要求する(件数取得失敗時の再読み込み用)
    pub(crate) fn refresh_for_delete(&self) {
        let Some(generation) = self.current_generation() else {
            return;
        };
        self.data_ops.update(|ops| ops.list_rev += 1);
        crate::features::asset_balance::store::load_asset_balances(
            self.session,
            generation,
            self.balances,
            self.dividends,
            self.lookup,
            self.csv,
            self.data_ops,
        );
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
        self.update_slot(self.csv, generation, update);
    }

    pub(crate) fn update_csv_fund(
        &self,
        generation: Generation,
        update: impl FnOnce(&mut CsvTabState<FundBalanceCsvRow>),
    ) {
        self.update_slot(self.csv_fund, generation, update);
    }

    // 見出し行で国内株式/投資信託を判別し、対応する backend の検証・保存へ流す
    pub(crate) fn select_file(&self, file: web_sys::File) {
        let Some(generation) = self.current_generation() else {
            return;
        };
        let store = *self;
        leptos::task::spawn_local(async move {
            let head = crate::support::csv_flow::read_file_head(&file, 8192)
                .await
                .unwrap_or_default();
            if !store.session.is_current(generation) {
                return;
            }
            match detect_asset_csv_kind(&head) {
                AssetKind::Fund => {
                    store.csv_kind.set(AssetKind::Fund);
                    let mut started = false;
                    store.update_csv_fund(generation, |state| {
                        started = state.begin_preview(file.name());
                    });
                    if !started {
                        return;
                    }
                    store.csv_file.set(Some((generation, file.clone())));
                    let result = crate::support::csv_flow::preview_csv(
                        crate::features::asset_balance::fund::FUND_PREVIEW_PATH,
                        &file,
                        crate::features::asset_balance::fund::FUND_CSV_CHUNKING,
                    )
                    .await;
                    store.apply_fund_preview_result(generation, result);
                }
                _ => {
                    store.csv_kind.set(AssetKind::Stock);
                    if !store.begin_file_preview(generation, file.name()) {
                        return;
                    }
                    store.csv_file.set(Some((generation, file.clone())));
                    let result = crate::support::csv_flow::preview_csv(
                        csv::PREVIEW_PATH,
                        &file,
                        csv::CSV_CHUNKING,
                    )
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
                }
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
        if self.csv_kind.get_untracked() == AssetKind::Fund {
            let Some((generation, file)) = self.try_begin_fund_save() else {
                return;
            };
            let store = *self;
            leptos::task::spawn_local(async move {
                let result = crate::support::csv_flow::upload_csv(
                    crate::features::asset_balance::fund::FUND_IMPORT_PATH,
                    &file,
                    crate::features::asset_balance::fund::FUND_CSV_CHUNKING,
                )
                .await;
                if store.apply_fund_upload_result(generation, result) {
                    load_fund_balances(store.session, generation, store.funds, store.data_ops);
                }
            });
            return;
        }
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

    fn try_begin_fund_save(&self) -> Option<(Generation, web_sys::File)> {
        self.try_begin_save_in(self.csv_fund)
    }

    fn try_begin_save(&self) -> Option<(Generation, web_sys::File)> {
        self.try_begin_save_in(self.csv)
    }

    fn try_begin_save_in<T: Send + Sync + 'static>(
        &self,
        slot: RwSignal<Option<(Generation, CsvTabState<T>)>>,
    ) -> Option<(Generation, web_sys::File)> {
        let generation = self.current_generation()?;
        let file = self.csv_file.with_untracked(|slot| match slot {
            Some((cached, file)) if *cached == generation => Some(file.clone()),
            _ => None,
        })?;
        let mut started = false;
        self.update_slot(slot, generation, |state| {
            started = state.has_preview_rows() && state.begin_save();
        });
        started.then_some((generation, file))
    }

    fn apply_preview_in<T: Send + Sync + 'static, R>(
        &self,
        slot: RwSignal<Option<(Generation, CsvTabState<T>)>>,
        generation: Generation,
        result: Result<CsvPreviewResponse<T>, ApiError>,
        map: impl FnOnce(&CsvPreviewResponse<T>) -> R,
    ) -> Option<R> {
        if !self.session.is_current(generation) {
            return None;
        }
        match result {
            Ok(preview) => {
                let value = map(&preview);
                self.update_slot(slot, generation, |state| {
                    state.finish_preview(preview);
                });
                Some(value)
            }
            Err(error) => {
                let message = error.message();
                self.update_slot(slot, generation, |state| state.fail_preview(message));
                None
            }
        }
    }

    fn apply_upload_in<T: Send + Sync + 'static>(
        &self,
        slot: RwSignal<Option<(Generation, CsvTabState<T>)>>,
        generation: Generation,
        result: Result<CsvUploadResponse, ApiError>,
    ) -> bool {
        if !self.session.is_current(generation) {
            return false;
        }
        match result {
            Ok(response) => {
                self.update_slot(slot, generation, |state| {
                    state.finish_save(Ok(response));
                });
                self.csv_file.set(None);
                true
            }
            Err(error) => {
                let message = error.message();
                self.update_slot(slot, generation, |state| {
                    state.finish_save(Err(message));
                });
                false
            }
        }
    }

    pub(crate) fn apply_fund_preview_result(
        &self,
        generation: Generation,
        result: Result<CsvPreviewResponse<FundBalanceCsvRow>, ApiError>,
    ) {
        self.apply_preview_in(self.csv_fund, generation, result, |_| ());
    }

    // 失敗は取引明細と違って画面に出す
    pub(crate) fn apply_preview_result(
        &self,
        generation: Generation,
        result: Result<CsvPreviewResponse<AssetBalanceCsvRow>, ApiError>,
    ) -> Option<Vec<String>> {
        self.apply_preview_in(self.csv, generation, result, |preview| {
            unique_sorted_codes(
                &preview
                    .rows
                    .iter()
                    .map(|row| row.security_code.clone())
                    .collect::<Vec<_>>(),
            )
        })
    }

    pub(crate) fn apply_fund_upload_result(
        &self,
        generation: Generation,
        result: Result<CsvUploadResponse, ApiError>,
    ) -> bool {
        self.apply_upload_in(self.csv_fund, generation, result)
    }

    pub(crate) fn apply_upload_result(
        &self,
        generation: Generation,
        result: Result<CsvUploadResponse, ApiError>,
    ) -> bool {
        self.apply_upload_in(self.csv, generation, result)
    }

    // ---- 投資信託の全件削除(保存済みカードから呼ぶ) ----

    pub(crate) fn fund_open_delete_confirm(&self) {
        if let Some(generation) = self.current_generation() {
            self.update_csv_fund(generation, |state| state.open_delete_confirm());
        }
    }

    pub(crate) fn fund_confirm_delete_all(&self) {
        let Some(generation) = self.try_begin_fund_delete() else {
            return;
        };
        let store = *self;
        leptos::task::spawn_local(async move {
            let result = crate::support::csv_flow::delete_all(
                crate::features::asset_balance::fund::FUND_LIST_PATH,
            )
            .await;
            store.apply_fund_delete_result(generation, result);
        });
    }

    fn try_begin_fund_delete(&self) -> Option<Generation> {
        self.try_begin_delete_in(self.csv_fund, |ops| ops.fund_list_rev += 1)
    }

    fn try_begin_delete_in<T: Send + Sync + 'static>(
        &self,
        slot: RwSignal<Option<(Generation, CsvTabState<T>)>>,
        bump_rev: impl FnOnce(&mut DataOps),
    ) -> Option<Generation> {
        let generation = self.current_generation()?;
        let mut started = false;
        self.update_slot(slot, generation, |state| {
            started = state.begin_delete();
        });
        if started {
            // 先に走った一覧取得の遅れ結果が削除後の一覧を復活させないよう無効化する
            self.data_ops.update(bump_rev);
        }
        started.then_some(generation)
    }

    fn apply_delete_in<T: Send + Sync + 'static>(
        &self,
        slot: RwSignal<Option<(Generation, CsvTabState<T>)>>,
        generation: Generation,
        result: Result<(), ApiError>,
        on_deleted: impl FnOnce(&Self),
    ) {
        if !self.session.is_current(generation) {
            return;
        }
        match result {
            Ok(()) => {
                self.update_slot(slot, generation, |state| state.finish_delete(Ok(())));
                // 削除確定後に届く一覧取得・配当ポーリングの遅れ結果を捨てる
                self.data_ops.update(DataOps::invalidate);
                on_deleted(self);
            }
            Err(error) => {
                let message = error.message();
                self.update_slot(slot, generation, |state| {
                    state.finish_delete(Err(message));
                });
            }
        }
    }

    pub(crate) fn apply_fund_delete_result(
        &self,
        generation: Generation,
        result: Result<(), ApiError>,
    ) {
        self.apply_delete_in(self.csv_fund, generation, result, |store| {
            store.funds.set(Some((
                generation,
                Ok(LoadedFundBalances {
                    rows: Vec::new(),
                    total: 0,
                    summary: None,
                    truncated: false,
                }),
            )));
        });
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
        self.try_begin_delete_in(self.csv, |ops| ops.list_rev += 1)
    }

    pub(crate) fn apply_delete_result(&self, generation: Generation, result: Result<(), ApiError>) {
        self.apply_delete_in(self.csv, generation, result, |store| {
            // DELETE 成功後は DB が空なので、未取得でも空を確定して読み込み表示を残さない
            store.balances.set(Some((
                generation,
                Ok(LoadedAssetBalances {
                    rows: Vec::new(),
                    total: 0,
                    summary: None,
                    truncated: false,
                }),
            )));
            store.dividends.set(DividendMaps::default());
            store.lookup.update(|store| store.clear());
        });
    }
}

pub(crate) fn csv_preview_rows<S, T: Clone>(state: &CsvTabState<T>) -> Vec<Row<S, T>> {
    state
        .preview
        .as_ref()
        .map(|preview| preview.rows.iter().cloned().map(Row::Preview).collect())
        .unwrap_or_default()
}

#[derive(Clone)]
pub(crate) struct ResolvedRows<R, S> {
    pub(crate) rows: Vec<R>,
    /// API が返した総件数。行が欠ける(truncated)とき rows.len とは一致しない
    pub(crate) total: usize,
    pub(crate) warning: Option<String>,
    /// 取得上限超過で行が欠けるときの API 集計。行が揃う通常時は None
    pub(crate) fallback_totals: Option<S>,
    /// API 集計(常に取れる)。行に依らない情報(自動更新の状況など)に使う
    pub(crate) summary: Option<S>,
}

pub(crate) type ResolvedFundBalance =
    ResolvedRows<FundBalanceRow, crate::api::dto::FundBalanceSummary>;
pub(crate) type ResolvedAssetBalance = ResolvedRows<AssetBalanceRow, AssetBalanceSummary>;

fn resolve_balance<T: Clone, P, S: Clone>(
    generation: Generation,
    slot: &LoadedSlotValue<T, S>,
) -> Option<ResolvedRows<Row<T, P>, S>> {
    let result = slot_value(slot, generation)?;
    Some(match result {
        Err(_) => ResolvedRows {
            rows: Vec::new(),
            total: 0,
            warning: None,
            fallback_totals: None,
            summary: None,
        },
        Ok(loaded) => ResolvedRows {
            rows: loaded.rows.iter().cloned().map(Row::Saved).collect(),
            total: loaded.total,
            warning: loaded.truncated.then(truncated_list_warning),
            fallback_totals: if loaded.truncated {
                loaded.summary.clone()
            } else {
                None
            },
            summary: loaded.summary.clone(),
        },
    })
}

pub(crate) fn resolve_fund_balance(
    generation: Generation,
    slot: &FundSlot,
) -> Option<ResolvedFundBalance> {
    resolve_balance(generation, slot)
}

pub(crate) fn resolve_asset_balance(
    generation: Generation,
    slot: &BalanceSlot,
) -> Option<ResolvedAssetBalance> {
    resolve_balance(generation, slot)
}

#[cfg(test)]
mod tests;
