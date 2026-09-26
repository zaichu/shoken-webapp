use super::holdings::{holding_view, HoldingView};
use crate::api::{ApiClient, ApiError};
use crate::asset_balance_csv::AssetBalanceCsvRow;
use crate::asset_balance_domain::format_number_value;
use crate::asset_balance_lookup::{fetch_single_asset_balance, AssetBalanceLookupStore};
use crate::asset_balance_search::filter_asset_balances;
use crate::csv_flow::CsvTabState;
use crate::dividend_per_share::{
    dividend_maps_from_batch, dividend_pending_max_retries, fetch_dividend_batch,
    post_dividend_batch, unique_sorted_codes, DividendMaps, DIVIDEND_NETWORK_MAX_RETRIES,
    DIVIDEND_RETRY_DELAY_MS,
};
use crate::dto::{AssetBalance, AssetBalanceListResponse, AssetBalanceSummary, SearchFacets};
use crate::receipts_pagination::PageCollector;
use crate::session::SessionStore;
use leptos::prelude::*;
use std::collections::HashSet;

pub(crate) const ASSET_BALANCE_LIST_PER_PAGE: usize = 1000;
// API の total が実データより大きい等の不整合でも必ず終了するためのページ数上限
pub(crate) const ASSET_BALANCE_LIST_MAX_PAGES: usize = 100;

#[derive(Clone, Debug)]
pub(crate) struct LoadedAssetBalances {
    pub(crate) rows: Vec<AssetBalance>,
    // API の total。実際に取得できた行数(rows.len)とは一致しないことがある
    pub(crate) total: usize,
    pub(crate) summary: Option<AssetBalanceSummary>,
    pub(crate) facets: Option<SearchFacets>,
    pub(crate) truncated: bool,
}

async fn fetch_asset_balance_page(page_no: usize) -> Result<AssetBalanceListResponse, ApiError> {
    let page = page_no.to_string();
    let per_page = ASSET_BALANCE_LIST_PER_PAGE.to_string();
    // summary・facets はどのページも同じ全体集計を返すため、取引明細と同じく1ページ目だけ取る
    let include_aggregates = if page_no == 1 { "true" } else { "false" };
    ApiClient::read_client()
        .get_json::<AssetBalanceListResponse>(
            "/api/v1/asset-balances",
            &[
                ("page", page.as_str()),
                ("per_page", per_page.as_str()),
                ("include_summary", include_aggregates),
                ("include_facets", include_aggregates),
            ],
        )
        .await
}

/// 一覧 API の全ページ結合。取引明細(`receipts/store.rs`)と同じく
/// `PageCollector` で末尾ページまで逐次取得し、summary は1ページ目のものを採用する。
pub(crate) struct AssetBalancePages {
    pub(crate) collector: PageCollector<AssetBalance>,
    pub(crate) summary: Option<AssetBalanceSummary>,
    pub(crate) facets: Option<SearchFacets>,
}

impl AssetBalancePages {
    pub(crate) fn new() -> Self {
        Self::with_limits(ASSET_BALANCE_LIST_PER_PAGE, ASSET_BALANCE_LIST_MAX_PAGES)
    }

    pub(crate) fn with_limits(per_page: usize, max_pages: usize) -> Self {
        Self {
            collector: PageCollector::new(per_page, max_pages),
            summary: None,
            facets: None,
        }
    }

    pub(crate) fn next_page(&self) -> usize {
        self.collector.next_page()
    }

    pub(crate) fn push(&mut self, page: AssetBalanceListResponse) -> bool {
        if self.collector.next_page() == 1 {
            self.summary = page.summary;
            self.facets = page.facets;
        }
        self.collector.push(page.data, page.total)
    }

    pub(crate) fn finish(self) -> LoadedAssetBalances {
        let total = self.collector.total();
        let truncated = self.collector.truncated();
        let rows = self.collector.into_rows();
        LoadedAssetBalances {
            total: total.unwrap_or(rows.len()),
            rows,
            summary: self.summary,
            facets: self.facets,
            truncated,
        }
    }
}

async fn fetch_asset_balances() -> Result<LoadedAssetBalances, ApiError> {
    let mut pages = AssetBalancePages::new();
    loop {
        let response = fetch_asset_balance_page(pages.next_page()).await?;
        if !pages.push(response) {
            break;
        }
    }
    Ok(pages.finish())
}

pub(crate) fn truncated_list_warning() -> String {
    format!(
        "一覧は最大{}件まで表示しています。未表示の銘柄がある可能性があります。",
        format_number_value((ASSET_BALANCE_LIST_PER_PAGE * ASSET_BALANCE_LIST_MAX_PAGES) as f64)
    )
}

pub(crate) type BalanceSlot = Option<(u64, Result<LoadedAssetBalances, String>)>;

// 配当は balances とは別の signal に書く。balances を更新すると
// ページ側の動的 view が作り直されてカードの開閉状態が失われるため、
// ここでは slot の世代確認だけを balances から非追跡で読み、
// 配当 signal だけを更新して子側の表示だけを差し替える。
pub(crate) fn apply_dividend_maps(
    balances: &RwSignal<BalanceSlot>,
    dividends: &RwSignal<DividendMaps>,
    generation: u64,
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
    generation: u64,
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

pub(crate) fn effective_summary(
    summary: Option<AssetBalanceSummary>,
    query: &str,
    has_csv_file: bool,
) -> Option<AssetBalanceSummary> {
    // 検索中は絞り込み前、CSV プレビュー中は取込前の全体集計のままなので API の summary は使わない
    if query.is_empty() && !has_csv_file {
        summary
    } else {
        None
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FilteredPortfolio {
    pub(crate) views: Vec<HoldingView>,
    pub(crate) summary: Option<AssetBalanceSummary>,
}

pub(crate) fn filtered_portfolio(
    rows: &[AssetBalance],
    summary: Option<AssetBalanceSummary>,
    query: &str,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: u64,
    has_csv_file: bool,
) -> FilteredPortfolio {
    let views = filter_asset_balances(rows, query)
        .into_iter()
        .map(|row| {
            // プレビュー行は CSV の値をそのまま見せるため lookup で DB 行に置き換えない
            let resolved = if has_csv_file {
                row.clone()
            } else {
                lookup
                    .with(|store| store.get(generation, &row.security_code).cloned())
                    .unwrap_or_else(|| row.clone())
            };
            holding_view(&resolved)
        })
        .collect();
    FilteredPortfolio {
        views,
        summary: effective_summary(summary, query, has_csv_file),
    }
}

pub(crate) fn should_apply_asset_balance_result(session: &SessionStore, generation: u64) -> bool {
    session.is_current(generation)
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DataOps {
    pub(crate) list_rev: u64,
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

    pub(crate) fn next_poll_rev(&mut self) {
        self.poll_rev += 1;
    }

    pub(crate) fn is_current_poll(&self, rev: u64) -> bool {
        self.poll_rev == rev
    }

    pub(crate) fn invalidate(&mut self) {
        self.list_rev += 1;
        self.poll_rev += 1;
    }

    pub(crate) fn reset(&mut self) {
        self.invalidate();
        self.inflight.clear();
        self.refresh_error = None;
    }
}

// 取得成功時は lookup の seed と配当取得の開始までここでまとめて行う
pub(crate) fn load_asset_balances(
    session: SessionStore,
    generation: u64,
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
                if should_apply_asset_balance_result(&session, generation) {
                    apply_list_error(generation, rev, error.message(), balances, data_ops);
                }
            }
            Ok(loaded) => {
                data_ops.update(|ops| ops.end_list_fetch(rev));
                if !should_apply_asset_balance_result(&session, generation)
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
                if !codes.is_empty() && !preview_active {
                    data_ops.update(DataOps::next_poll_rev);
                    let poll_rev = data_ops.with_untracked(|ops| ops.poll_rev);
                    leptos::task::spawn_local(poll_dividend_maps(
                        session, generation, codes, balances, dividends, data_ops, poll_rev,
                    ));
                }
            }
        }
    });
}

pub(crate) fn apply_list_error(
    generation: u64,
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
    generation: u64,
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
            .map(|row| row.security_code.clone())
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

pub(crate) type AssetCsvSlot = Option<(u64, CsvTabState<AssetBalanceCsvRow>)>;
pub(crate) type AssetCsvFileSlot = Option<(u64, web_sys::File)>;
