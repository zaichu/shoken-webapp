pub(super) mod csv;

use super::holdings::{HoldingView, holding_view};
use crate::api::dto::{AssetBalance, AssetBalanceSummary, FundBalance, FundBalanceSummary};
use crate::api::{ApiClient, ApiError};
use crate::features::asset_balance::csv::{AssetBalanceCsvRow, AssetBalanceRow};
use crate::features::asset_balance::lookup::{AssetBalanceLookupStore, fetch_single_asset_balance};
use crate::features::asset_balance::model::format_number_value;
use crate::features::asset_balance::search::filter_asset_balances;
use crate::features::dividend_per_share::{
    DIVIDEND_NETWORK_MAX_RETRIES, DIVIDEND_RETRY_DELAY_MS, DividendMaps, dividend_maps_from_batch,
    dividend_pending_max_retries, fetch_dividend_batch, post_dividend_batch, unique_sorted_codes,
};
use crate::session::{Generation, SessionStore};
use crate::support::csv_flow::CsvTabState;
use crate::support::pagination::{LIST_MAX_PAGES, LIST_PER_PAGE, ListEndpoint, fetch_all_pages};
use crate::support::row::Row;
use leptos::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub(crate) struct LoadedRows<T, S> {
    pub(crate) rows: Vec<T>,
    // API の total。実際に取得できた行数(rows.len)とは一致しないことがある
    pub(crate) total: usize,
    // 取得上限超過で行が欠けるときの合計フォールバック用。行が揃う通常時は使わない
    pub(crate) summary: Option<S>,
    pub(crate) truncated: bool,
}

pub(crate) type LoadedAssetBalances = LoadedRows<AssetBalance, AssetBalanceSummary>;
pub(crate) type LoadedFundBalances = LoadedRows<FundBalance, FundBalanceSummary>;

/// 資産残高一覧のエンドポイント。
impl ListEndpoint for AssetBalance {
    type Row = AssetBalance;
    type Summary = AssetBalanceSummary;

    const PATH: &'static str = super::csv::LIST_PATH;
}

/// 投資信託残高一覧のエンドポイント。
impl ListEndpoint for FundBalance {
    type Row = FundBalance;
    type Summary = FundBalanceSummary;

    const PATH: &'static str = super::fund::FUND_LIST_PATH;
}

async fn fetch_loaded<E: ListEndpoint>() -> Result<LoadedRows<E::Row, E::Summary>, ApiError> {
    let page = fetch_all_pages::<E>().await?;
    Ok(LoadedRows {
        total: page.total.unwrap_or(page.rows.len()),
        rows: page.rows,
        summary: page.summary,
        truncated: page.truncated,
    })
}

pub(crate) async fn fetch_fund_balances() -> Result<LoadedFundBalances, ApiError> {
    fetch_loaded::<FundBalance>().await
}

pub(crate) type FundSlot = Option<(Generation, Result<LoadedFundBalances, String>)>;

// ファンドは株価・配当の外部取得がなく、一覧取得だけで表示まで完結する
pub(crate) fn load_fund_balances(
    session: SessionStore,
    generation: Generation,
    funds: RwSignal<FundSlot>,
    data_ops: RwSignal<DataOps>,
) {
    data_ops.update(DataOps::begin_fund_fetch);
    let rev = data_ops.with_untracked(|ops| ops.fund_list_rev);
    leptos::task::spawn_local(async move {
        match fetch_fund_balances().await {
            Err(error) => {
                data_ops.update(|ops| ops.end_fund_fetch(rev));
                if session.is_current(generation)
                    && data_ops.with_untracked(|ops| ops.is_current_fund_list(rev))
                {
                    let has_cached = funds.with_untracked(
                        |slot| matches!(slot, Some((cached, Ok(_))) if *cached == generation),
                    );
                    if has_cached {
                        data_ops.update(|ops| ops.refresh_error = Some(error.message()));
                    } else {
                        funds.set(Some((generation, Err(error.message()))));
                    }
                }
            }
            Ok(loaded) => {
                data_ops.update(|ops| ops.end_fund_fetch(rev));
                if session.is_current(generation)
                    && data_ops.with_untracked(|ops| ops.is_current_fund_list(rev))
                {
                    funds.set(Some((generation, Ok(loaded))));
                }
            }
        }
    });
}

pub(crate) async fn fetch_asset_balances() -> Result<LoadedAssetBalances, ApiError> {
    fetch_loaded::<AssetBalance>().await
}

pub(crate) fn truncated_list_warning() -> String {
    format!(
        "一覧は最大{}件まで表示しています。未表示の銘柄がある可能性があります。",
        format_number_value((LIST_PER_PAGE * LIST_MAX_PAGES) as f64)
    )
}

pub(crate) type BalanceSlot = Option<(Generation, Result<LoadedAssetBalances, String>)>;

// 配当は balances とは別の signal に書く。balances を更新すると
// ページ側の動的 view が作り直されてカードの開閉状態が失われるため、
// ここでは slot の世代確認だけを balances から非追跡で読み、
// 配当 signal だけを更新して子側の表示だけを差し替える。
pub(crate) fn apply_dividend_maps(
    balances: &RwSignal<BalanceSlot>,
    dividends: &RwSignal<DividendMaps>,
    generation: Generation,
    maps: DividendMaps,
) {
    let current = balances
        .with_untracked(|slot| matches!(slot, Some((cached, Ok(_))) if *cached == generation));
    if current {
        dividends.set(maps);
    }
}

pub(crate) async fn poll_dividend_maps(
    session: SessionStore,
    generation: Generation,
    codes: Vec<String>,
    balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
    data_ops: RwSignal<DataOps>,
    poll_rev: u64,
) {
    let is_current = move || {
        session.is_current(generation)
            && data_ops.with_untracked(|ops| ops.is_current_poll(poll_rev))
    };
    let max_pending = dividend_pending_max_retries(codes.len());
    let mut pending_used = 0u32;
    let mut network_used = 0u32;
    loop {
        if !is_current() {
            break;
        }
        match fetch_dividend_batch(&codes, post_dividend_batch).await {
            Ok(batch) => {
                let (maps, has_pending) = dividend_maps_from_batch(&batch);
                if is_current() {
                    apply_dividend_maps(&balances, &dividends, generation, maps);
                }
                if !has_pending || pending_used >= max_pending {
                    break;
                }
                pending_used += 1;
            }
            Err(_) => {
                if network_used >= DIVIDEND_NETWORK_MAX_RETRIES {
                    break;
                }
                network_used += 1;
            }
        }
        gloo_timers::future::TimeoutFuture::new(DIVIDEND_RETRY_DELAY_MS).await;
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FilteredPortfolio {
    /// 絞り込み後の行。カード一覧・構成比・件数表示に使う
    pub(crate) views: Vec<HoldingView>,
    /// 絞り込み前の全行。合計はここから求め、検索中も全保有分を維持する
    pub(crate) all_views: Vec<HoldingView>,
}

pub(crate) fn filtered_portfolio(
    rows: &[AssetBalanceRow],
    search: &crate::features::asset_balance::fund::AssetSearch,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: Generation,
) -> FilteredPortfolio {
    // プレビュー行は CSV の値をそのまま見せるため lookup で DB 行に置き換えない
    let resolve_view = |row: &AssetBalanceRow| {
        let mut view = match row {
            Row::Preview(_) => holding_view(row),
            Row::Saved(saved) => {
                let resolved = lookup
                    .with(|store| store.get(generation, saved).cloned())
                    .unwrap_or_else(|| saved.clone());
                holding_view(&resolved)
            }
        };
        // 詳細は保存済み行のレコード ID で開く。lookup の代替行の id ではなく外側の行の id を使う
        view.id = row.saved_id();
        view
    };
    FilteredPortfolio {
        views: filter_asset_balances(rows, search)
            .into_iter()
            .map(resolve_view)
            .collect(),
        all_views: rows.iter().map(resolve_view).collect(),
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DataOps {
    pub(crate) list_rev: u64,
    pub(crate) fund_list_rev: u64,
    pub(crate) poll_rev: u64,
    // 完了時に世代をまたいで減算されないよう、取得中の rev をそのまま持つ
    pub(crate) inflight: HashSet<u64>,
    pub(crate) refresh_error: Option<String>,
}

impl DataOps {
    pub(crate) fn begin_list_fetch(&mut self) {
        self.list_rev += 1;
        self.inflight.insert(self.list_rev);
        self.refresh_error = None;
    }

    pub(crate) fn end_list_fetch(&mut self, rev: u64) {
        self.inflight.remove(&rev);
    }

    pub(crate) fn is_current_list(&self, rev: u64) -> bool {
        self.list_rev == rev
    }

    pub(crate) fn begin_fund_fetch(&mut self) {
        self.fund_list_rev += 1;
        self.inflight.insert(self.fund_list_rev);
        self.refresh_error = None;
    }

    pub(crate) fn end_fund_fetch(&mut self, rev: u64) {
        self.inflight.remove(&rev);
    }

    pub(crate) fn is_current_fund_list(&self, rev: u64) -> bool {
        self.fund_list_rev == rev
    }

    pub(crate) fn next_poll_rev(&mut self) {
        self.poll_rev += 1;
    }

    pub(crate) fn is_current_poll(&self, rev: u64) -> bool {
        self.poll_rev == rev
    }

    pub(crate) fn list_fetch_in_flight(&self) -> bool {
        !self.inflight.is_empty()
    }

    pub(crate) fn invalidate(&mut self) {
        self.list_rev += 1;
        self.fund_list_rev += 1;
        self.poll_rev += 1;
    }

    pub(crate) fn reset(&mut self) {
        self.invalidate();
        self.inflight.clear();
        self.refresh_error = None;
    }
}

pub(crate) fn has_current_balances(slot: &BalanceSlot, generation: Generation) -> bool {
    matches!(slot, Some((cached, _)) if *cached == generation)
}

// 取得成功時は lookup の seed と配当取得の開始までここでまとめて行う
pub(crate) fn load_asset_balances(
    session: SessionStore,
    generation: Generation,
    balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    csv: RwSignal<AssetCsvSlot>,
    data_ops: RwSignal<DataOps>,
) {
    data_ops.update(DataOps::begin_list_fetch);
    let rev = data_ops.with_untracked(|ops| ops.list_rev);
    leptos::task::spawn_local(async move {
        match fetch_asset_balances().await {
            Err(error) => {
                data_ops.update(|ops| ops.end_list_fetch(rev));
                if session.is_current(generation) {
                    apply_list_error(generation, rev, error.message(), balances, data_ops);
                }
            }
            Ok(loaded) => {
                data_ops.update(|ops| ops.end_list_fetch(rev));
                if !session.is_current(generation)
                    || !data_ops.with_untracked(|ops| ops.is_current_list(rev))
                {
                    return;
                }
                let preview_active = csv.with_untracked(|slot| {
                    matches!(slot, Some((cached, state))
                        if *cached == generation && state.file_name.is_some())
                });
                let (codes, missing) = apply_loaded_asset_balances(
                    generation,
                    loaded,
                    balances,
                    dividends,
                    lookup,
                    preview_active,
                );
                if !missing.is_empty() {
                    leptos::task::spawn_local(async move {
                        for code in missing {
                            if !session.is_current(generation)
                                || !data_ops.with_untracked(|ops| ops.is_current_list(rev))
                            {
                                break;
                            }
                            if let Ok(rows) =
                                fetch_single_asset_balance(&ApiClient::read_client(), &code).await
                            {
                                if !session.is_current(generation)
                                    || !data_ops.with_untracked(|ops| ops.is_current_list(rev))
                                {
                                    break;
                                }
                                lookup.update(|store| {
                                    store.store_single(generation, &code, rows);
                                });
                            }
                        }
                    });
                }
                // ポーリング結果はマップ全体を置き換えるため、プレビュー中は
                // プレビュー銘柄も混ぜて取り直し、片方の配当が欠けないようにする
                let poll_codes = if preview_active {
                    let preview_codes = csv.with_untracked(|slot| match slot {
                        Some((cached, state)) if *cached == generation => state
                            .preview
                            .as_ref()
                            .map(|preview| {
                                preview
                                    .rows
                                    .iter()
                                    .map(|row| row.security_code.to_string())
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default(),
                        _ => Vec::new(),
                    });
                    unique_sorted_codes(
                        &codes
                            .iter()
                            .cloned()
                            .chain(preview_codes)
                            .collect::<Vec<_>>(),
                    )
                } else {
                    codes
                };
                if !poll_codes.is_empty() {
                    data_ops.update(DataOps::next_poll_rev);
                    let poll_rev = data_ops.with_untracked(|ops| ops.poll_rev);
                    leptos::task::spawn_local(poll_dividend_maps(
                        session, generation, poll_codes, balances, dividends, data_ops, poll_rev,
                    ));
                }
            }
        }
    });
}

pub(crate) fn apply_list_error(
    generation: Generation,
    rev: u64,
    message: String,
    balances: RwSignal<BalanceSlot>,
    data_ops: RwSignal<DataOps>,
) {
    if !data_ops.with_untracked(|ops| ops.is_current_list(rev)) {
        return;
    }
    // キャッシュ済みの行は残し、エラーだけを出す(キャッシュ済みデータとエラーは別々に扱う)
    let has_cached = balances
        .with_untracked(|slot| matches!(slot, Some((cached, Ok(_))) if *cached == generation));
    if has_cached {
        data_ops.update(|ops| ops.refresh_error = Some(message));
    } else {
        balances.set(Some((generation, Err(message))));
    }
}

pub(crate) fn apply_loaded_asset_balances(
    generation: Generation,
    loaded: LoadedAssetBalances,
    balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    preview_active: bool,
) -> (Vec<String>, Vec<String>) {
    // 同一世代の置換では or_insert では古い値が残るため seed 前に必ず消す
    lookup.update(|store| {
        store.clear();
        store.seed(generation, &loaded.rows);
    });
    let codes = unique_sorted_codes(
        &loaded
            .rows
            .iter()
            .map(|row| row.security_code.to_string())
            .collect::<Vec<_>>(),
    );
    if !preview_active {
        dividends.set(DividendMaps::default());
    }
    balances.set(Some((generation, Ok(loaded))));
    let missing: Vec<String> = codes
        .iter()
        .filter(|code| lookup.with_untracked(|store| store.needs_fetch(generation, code, true)))
        .cloned()
        .collect();
    (codes, missing)
}

pub(crate) type AssetCsvSlot = Option<(Generation, CsvTabState<AssetBalanceCsvRow>)>;
pub(crate) type AssetCsvFileSlot = Option<(Generation, web_sys::File)>;

#[cfg(test)]
mod tests;
