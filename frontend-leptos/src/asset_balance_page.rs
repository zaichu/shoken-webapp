use crate::api::{ApiClient, ApiError};
use crate::asset_balance_csv::{self, AssetBalanceCsvRow};
use crate::asset_balance_domain::{
    calculate_portfolio_kpi, calculate_valuation, format_abs_number, format_number_value,
    normalize_security_name, summarize_valuation_with_summary, to_fixed, total_purchase_amount,
    KpiHolding, SummaryOverride, ValuationItem,
};
use crate::asset_balance_lookup::{fetch_single_asset_balance, AssetBalanceLookupStore};
use crate::asset_balance_portfolio::{chart_display, chart_plan};
use crate::asset_balance_search::{
    asset_balance_search_options, clear_search_query, filter_asset_balances,
};
use crate::asset_review_prompt::generate_asset_review_prompt;
use crate::collapsible_search_card::CollapsibleSearchCard;
use crate::confirm_modal::ConfirmDeleteModal;
use crate::csv_flow::{csv_error_message, CsvTabState};
use crate::csv_rail::CsvActionRail;
use crate::dividend_per_share::{
    dividend_maps_from_batch, dividend_pending_max_retries, fetch_dividend_batch,
    unique_sorted_codes, DividendMaps, DIVIDEND_NETWORK_MAX_RETRIES, DIVIDEND_RETRY_DELAY_MS,
};
use crate::dto::{
    AssetBalance, AssetBalanceListResponse, AssetBalanceSummary, CsvPreviewResponse,
    CsvUploadResponse, SearchFacets,
};
use crate::receipts_pagination::PageCollector;
use crate::receipts_search::SearchOption;
use crate::security_link::{try_copy_to_clipboard, SecurityCodeLink};
use crate::session::{use_session, SessionStore};
use crate::ui::PageHeader;
use leptos::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::collections::HashSet;

const ASSET_BALANCE_LIST_PER_PAGE: usize = 1000;
// API の total が実データより大きい等の不整合でも必ず終了するためのページ数上限
const ASSET_BALANCE_LIST_MAX_PAGES: usize = 100;

const CHART_COLORS: [&str; 10] = [
    "#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#ec4899", "#06b6d4", "#f97316",
    "#84cc16", "#6366f1",
];

#[derive(Clone, Debug)]
struct LoadedAssetBalances {
    rows: Vec<AssetBalance>,
    // API の total。実際に取得できた行数(rows.len)とは一致しないことがある
    total: usize,
    summary: Option<AssetBalanceSummary>,
    facets: Option<SearchFacets>,
    truncated: bool,
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

/// 一覧 API の全ページ結合。取引明細(`receipts.rs`)と同じく
/// `PageCollector` で末尾ページまで逐次取得し、summary は1ページ目のものを採用する。
struct AssetBalancePages {
    collector: PageCollector<AssetBalance>,
    summary: Option<AssetBalanceSummary>,
    facets: Option<SearchFacets>,
}

impl AssetBalancePages {
    fn new() -> Self {
        Self::with_limits(ASSET_BALANCE_LIST_PER_PAGE, ASSET_BALANCE_LIST_MAX_PAGES)
    }

    fn with_limits(per_page: usize, max_pages: usize) -> Self {
        Self {
            collector: PageCollector::new(per_page, max_pages),
            summary: None,
            facets: None,
        }
    }

    fn next_page(&self) -> usize {
        self.collector.next_page()
    }

    fn push(&mut self, page: AssetBalanceListResponse) -> bool {
        if self.collector.next_page() == 1 {
            self.summary = page.summary;
            self.facets = page.facets;
        }
        self.collector.push(page.data, page.total)
    }

    fn finish(self) -> LoadedAssetBalances {
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

fn truncated_list_warning() -> String {
    format!(
        "一覧は最大{}件まで表示しています。未表示の銘柄がある可能性があります。",
        format_number_value((ASSET_BALANCE_LIST_PER_PAGE * ASSET_BALANCE_LIST_MAX_PAGES) as f64)
    )
}

type BalanceSlot = Option<(u64, Result<LoadedAssetBalances, String>)>;

// 配当は balances とは別の signal に書く。balances を更新すると
// ページ側の動的 view が作り直されてカードの開閉状態が失われるため、
// ここでは slot の世代確認だけを balances から非追跡で読み、
// 配当 signal だけを更新して子側の表示だけを差し替える。
fn apply_dividend_maps(
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

async fn poll_dividend_maps(
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
        match fetch_dividend_batch(&codes).await {
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

fn effective_summary(
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
struct FilteredPortfolio {
    views: Vec<HoldingView>,
    summary: Option<AssetBalanceSummary>,
}

fn filtered_portfolio(
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

fn should_apply_asset_balance_result(session: &SessionStore, generation: u64) -> bool {
    session.is_current(generation)
}

#[derive(Clone, Debug, Default)]
struct DataOps {
    list_rev: u64,
    poll_rev: u64,
    // 完了時に世代をまたいで減算されないよう、取得中の rev をそのまま持つ
    inflight: HashSet<u64>,
    refresh_error: Option<String>,
}

impl DataOps {
    fn begin_list_fetch(&mut self) {
        self.list_rev += 1;
        self.inflight.insert(self.list_rev);
        self.refresh_error = None;
    }

    fn end_list_fetch(&mut self, rev: u64) {
        self.inflight.remove(&rev);
    }

    fn is_current_list(&self, rev: u64) -> bool {
        self.list_rev == rev
    }

    fn next_poll_rev(&mut self) {
        self.poll_rev += 1;
    }

    fn is_current_poll(&self, rev: u64) -> bool {
        self.poll_rev == rev
    }

    fn invalidate(&mut self) {
        self.list_rev += 1;
        self.poll_rev += 1;
    }

    fn reset(&mut self) {
        self.invalidate();
        self.inflight.clear();
        self.refresh_error = None;
    }
}

// 取得成功時は lookup の seed と配当取得の開始までここでまとめて行う
fn load_asset_balances(
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

fn apply_list_error(
    generation: u64,
    rev: u64,
    message: String,
    balances: RwSignal<BalanceSlot>,
    data_ops: RwSignal<DataOps>,
) {
    if !data_ops.with_untracked(|ops| ops.is_current_list(rev)) {
        return;
    }
    // キャッシュ済みの行は残し、エラーだけを出す(React は dbQuery.data と error を別々に扱う)
    let has_cached = balances
        .with_untracked(|slot| matches!(slot, Some((cached, Ok(_))) if *cached == generation));
    if has_cached {
        data_ops.update(|ops| ops.refresh_error = Some(message));
    } else {
        balances.set(Some((generation, Err(message))));
    }
}

fn apply_loaded_asset_balances(
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

type AssetCsvSlot = Option<(u64, CsvTabState<AssetBalanceCsvRow>)>;
type AssetCsvFileSlot = Option<(u64, web_sys::File)>;

fn can_save_csv(state: &CsvTabState<AssetBalanceCsvRow>) -> bool {
    state
        .preview
        .as_ref()
        .is_some_and(|preview| !preview.rows.is_empty())
}

// 一覧キャッシュと同じく世代で区切り、ログアウト・ユーザー切替で自動的に無効化する
#[derive(Clone)]
struct AssetBalanceCsvStore {
    session: SessionStore,
    balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    csv: RwSignal<AssetCsvSlot>,
    csv_file: RwSignal<AssetCsvFileSlot>,
    data_ops: RwSignal<DataOps>,
}

impl AssetBalanceCsvStore {
    fn new(
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

    fn csv_state(&self) -> CsvTabState<AssetBalanceCsvRow> {
        let generation = self.session.generation.get();
        self.csv.with(|slot| match slot {
            Some((cached, state)) if *cached == generation => state.clone(),
            _ => CsvTabState::default(),
        })
    }

    fn csv_busy(&self) -> bool {
        self.csv_state().busy()
    }

    fn is_authenticated(&self) -> bool {
        self.session.user.get().is_some()
    }

    // 削除件数はプレビューではなく DB 側の total を使う
    fn db_count(&self) -> usize {
        let generation = self.session.generation.get();
        self.balances.with(|slot| match slot {
            Some((cached, Ok(loaded))) if *cached == generation => loaded.total,
            _ => 0,
        })
    }

    fn list_loading(&self) -> bool {
        let generation = self.session.generation.get();
        self.session.user.get().is_some()
            && (self.data_ops.with(|ops| !ops.inflight.is_empty())
                || self
                    .balances
                    .with(|slot| !matches!(slot, Some((cached, _)) if *cached == generation)))
    }

    fn update_csv(
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

    fn select_file(&self, file: web_sys::File) {
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
            let result = crate::csv_flow::preview_csv(asset_balance_csv::PREVIEW_PATH, &file).await;
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
    fn begin_file_preview(&self, generation: u64, file_name: String) -> bool {
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

    fn save_csv(&self) {
        let Some((generation, file)) = self.try_begin_save() else {
            return;
        };
        let store = self.clone();
        leptos::task::spawn_local(async move {
            let result = crate::csv_flow::upload_csv(asset_balance_csv::IMPORT_PATH, &file).await;
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

    fn open_delete_confirm(&self) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        self.update_csv(generation, |state| state.open_delete_confirm());
    }

    fn close_delete_confirm(&self) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        let generation = self.session.generation.get_untracked();
        self.update_csv(generation, |state| state.close_delete_confirm());
    }

    fn confirm_delete_all(&self) {
        let Some(generation) = self.try_begin_delete() else {
            return;
        };
        let store = self.clone();
        leptos::task::spawn_local(async move {
            let result = crate::csv_flow::delete_all(asset_balance_csv::LIST_PATH).await;
            store.apply_delete_result(generation, result);
        });
    }

    fn try_begin_delete(&self) -> Option<u64> {
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
    fn apply_preview_result(
        &self,
        generation: u64,
        result: Result<CsvPreviewResponse, ApiError>,
    ) -> Option<Vec<String>> {
        if !self.session.is_current(generation) {
            return None;
        }
        match result {
            Ok(response) => {
                let preview = asset_balance_csv::to_preview(response);
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

    fn apply_upload_result(
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

    fn apply_delete_result(&self, generation: u64, result: Result<(), ApiError>) {
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

fn dec_to_f64(value: &Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}

fn format_currency(value: f64) -> String {
    match format_abs_number(to_fixed(value, 15)) {
        None => "-".to_string(),
        Some(body) if value < 0.0 => format!("¥ -{body}"),
        Some(body) => format!("¥ {body}"),
    }
}

fn format_fixed_percent(value: f64, decimals: u32) -> String {
    if value.is_nan() {
        return "-".to_string();
    }
    format!(
        "{:.prec$}%",
        to_fixed(value, decimals),
        prec = decimals as usize
    )
}

fn format_percentage_value(value: f64) -> String {
    format_fixed_percent(value, 2)
}

fn format_valuation_amount(amount: Option<f64>) -> String {
    match amount {
        Some(value) if value.is_nan() => "—".to_string(),
        Some(value) if value >= 0.0 => format!("+{}", format_currency(value)),
        Some(value) => format_currency(value),
        None => "—".to_string(),
    }
}

fn format_valuation_rate(rate: Option<f64>, decimals: u32) -> String {
    match rate {
        Some(value) if value.is_nan() => "—".to_string(),
        Some(value) => {
            let sign = if value >= 0.0 { "+" } else { "" };
            format!("{sign}{}", format_fixed_percent(value, decimals))
        }
        None => "—".to_string(),
    }
}

#[derive(Clone, Debug)]
struct HoldingView {
    code: String,
    name: String,
    shares: f64,
    average_price: f64,
    purchase: f64,
    market: f64,
    current_price: f64,
}

fn holding_view(row: &AssetBalance) -> HoldingView {
    let name_source = if row.security_name.is_empty() {
        row.security_code.as_str()
    } else {
        row.security_name.as_str()
    };
    HoldingView {
        code: row.security_code.clone(),
        name: normalize_security_name(name_source),
        shares: dec_to_f64(&row.shares),
        average_price: dec_to_f64(&row.average_purchase_price),
        purchase: dec_to_f64(&row.total_purchase_amount),
        market: dec_to_f64(&row.market_value),
        current_price: dec_to_f64(&row.current_price),
    }
}

#[derive(Clone, Debug, PartialEq)]
struct HoldingDividend {
    per_share: Option<f64>,
    annual: Option<f64>,
    yield_value: Option<f64>,
    status: Option<String>,
}

fn holding_dividend(
    code: &str,
    shares: f64,
    average_price: f64,
    maps: &DividendMaps,
) -> HoldingDividend {
    let status = maps.status.get(code).cloned();
    match status.as_deref() {
        Some("pending") | Some("error") => HoldingDividend {
            per_share: None,
            annual: None,
            yield_value: None,
            status,
        },
        Some("zero") => HoldingDividend {
            per_share: Some(0.0),
            annual: Some(0.0),
            yield_value: None,
            status,
        },
        _ => match maps.per_share.get(code) {
            None => HoldingDividend {
                per_share: None,
                annual: None,
                yield_value: None,
                status,
            },
            Some(per_share) => HoldingDividend {
                per_share: Some(*per_share),
                annual: Some(*per_share * shares),
                yield_value: if average_price > 0.0 {
                    Some(*per_share / average_price * 100.0)
                } else {
                    None
                },
                status,
            },
        },
    }
}

fn format_dividend_per_share(dividend: &HoldingDividend) -> String {
    match dividend.status.as_deref() {
        Some("pending") => "取得中...".to_string(),
        Some("error") => "取得失敗".to_string(),
        _ => match dividend.per_share {
            Some(value) => format_currency(value),
            None => "---".to_string(),
        },
    }
}

fn format_dividend_annual(dividend: &HoldingDividend) -> String {
    match dividend.status.as_deref() {
        Some("pending") => "取得中...".to_string(),
        Some("error") => "取得失敗".to_string(),
        _ => match dividend.annual {
            Some(value) => format_currency(value),
            None => "---".to_string(),
        },
    }
}

fn format_dividend_yield(dividend: &HoldingDividend) -> String {
    match dividend.status.as_deref() {
        Some("pending") => "取得中...".to_string(),
        Some("error") => "取得失敗".to_string(),
        _ => match dividend.yield_value {
            Some(value) => format_percentage_value(value),
            None => "---".to_string(),
        },
    }
}

fn csv_preview_rows(state: &CsvTabState<AssetBalanceCsvRow>) -> Vec<AssetBalance> {
    state
        .preview
        .as_ref()
        .map(|preview| {
            preview
                .rows
                .iter()
                .map(AssetBalanceCsvRow::to_asset_balance)
                .collect()
        })
        .unwrap_or_default()
}

fn csv_status_text(state: &CsvTabState<AssetBalanceCsvRow>) -> Option<&'static str> {
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

struct ResolvedAssetBalance {
    rows: Vec<AssetBalance>,
    summary: Option<AssetBalanceSummary>,
    facets: Option<SearchFacets>,
    warning: Option<String>,
    has_csv_file: bool,
}

fn resolve_asset_balance(
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

#[component]
fn AssetBalanceRailExtras(
    rows: Vec<AssetBalance>,
    facets: Option<SearchFacets>,
    has_csv_file: bool,
    warning: Option<String>,
    search_query: RwSignal<String>,
) -> impl IntoView {
    let options = asset_balance_search_options(&rows, facets.as_ref(), has_csv_file);
    let has_rows = !rows.is_empty();
    view! {
        {warning.map(|text| {
            view! {
                <div class="px-5 py-4" role="status" aria-live="polite">
                    <div
                        class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm font-medium text-amber-900 shadow-sm"
                        role="alert"
                    >
                        {text}
                    </div>
                </div>
            }
        })}
        {has_rows.then(|| view! { <AssetBalanceSearchCard query=search_query options=options /> })}
        <AssetReviewPromptCard rows=rows />
    }
}

#[derive(Clone, Copy)]
enum ReviewCopyStatus {
    Idle,
    Success,
    Error,
}

#[component]
fn AssetReviewPromptCard(rows: Vec<AssetBalance>) -> impl IntoView {
    let status = RwSignal::new(ReviewCopyStatus::Idle);
    let click_generation = RwSignal::new(0u64);
    let disabled = rows.is_empty();
    let label = move || match status.get() {
        ReviewCopyStatus::Success => "コピーしました！",
        ReviewCopyStatus::Error => "コピーに失敗しました",
        ReviewCopyStatus::Idle => "AI総評プロンプトをコピー",
    };
    let on_click = move |_| {
        let rows = rows.clone();
        let generation = click_generation.get() + 1;
        click_generation.set(generation);
        leptos::task::spawn_local(async move {
            let ok = try_copy_to_clipboard(generate_asset_review_prompt(&rows)).await;
            if click_generation.get() != generation {
                return;
            }
            status.set(if ok {
                ReviewCopyStatus::Success
            } else {
                ReviewCopyStatus::Error
            });
            gloo_timers::future::TimeoutFuture::new(3_000).await;
            if click_generation.get() == generation {
                status.set(ReviewCopyStatus::Idle);
            }
        });
    };
    view! {
        <div class="px-5 py-4" data-testid="asset-review-prompt-card">
            <p class="mb-2 text-xs font-medium text-secondary">"AI総評プロンプト"</p>
            <button
                type="button"
                class="inline-flex w-full items-center justify-center truncate rounded-md border border-slate-300 px-3 py-1.5 text-sm font-bold text-slate-700 transition-[background-color,border-color,color,box-shadow,transform] hover:border-slate-500 hover:bg-slate-50 focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print max-sm:min-h-[44px]"
                aria-label=label
                disabled=disabled
                on:click=on_click
            >
                {label}
            </button>
        </div>
    }
}

#[component]
fn AssetBalanceMainContent(
    state: CsvTabState<AssetBalanceCsvRow>,
    rows: Vec<AssetBalance>,
    summary: Option<AssetBalanceSummary>,
    has_csv_file: bool,
    search_query: RwSignal<String>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: u64,
) -> impl IntoView {
    if state.previewing {
        show_all.set(false);
        return view! { <CsvStatusMessage text="CSVファイルを解析しています..." /> }.into_any();
    }
    let total_count = rows.len();
    let status = csv_status_text(&state);
    view! {
        {status.map(|text| view! { <CsvStatusMessage text=text /> })}
        {move || {
            let query = search_query.get();
            if rows.is_empty() && query.is_empty() {
                show_all.set(false);
                return view! {
                    <div class="mb-3 overflow-hidden rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_14px_38px_-32px_rgba(15,23,42,0.85)]">
                        <div>
                            <div class="flex flex-col items-center justify-center rounded-xl border border-dashed border-slate-300 bg-slate-50/70 px-4 py-8 text-center">
                                <h3 class="text-base font-black text-slate-950">
                                    "資産管理データがありません"
                                </h3>
                                <p class="mt-1.5 max-w-md text-sm font-medium text-slate-600">
                                    "CSVファイルをインポートするか、データを登録してください。"
                                </p>
                            </div>
                        </div>
                    </div>
                }
                    .into_any();
            }
            let FilteredPortfolio { views, summary } = filtered_portfolio(
                &rows,
                summary.clone(),
                &query,
                lookup,
                generation,
                has_csv_file,
            );
            view! {
                <PortfolioSummary
                    views=views
                    total_count=total_count
                    is_filtered=!query.is_empty()
                    on_clear_filter=move || {
                        search_query.set(clear_search_query())
                    }
                    summary=summary
                    dividends=dividends
                    show_all=show_all
                />
            }
                .into_any()
        }}
    }
    .into_any()
}

#[component]
pub fn AssetBalancePage() -> impl IntoView {
    let session = use_session();
    let render_session = session;
    let busy_session = session;
    let rail_session = session;
    let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
    let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
    let lookup = RwSignal::new(AssetBalanceLookupStore::new());
    let search_query = RwSignal::new(String::new());
    let reset_query = search_query;
    // ページ側に持ち上げた show_all はアンマウントで破棄されないため、グラフを描かない分岐では false に戻す
    let show_all = RwSignal::new(false);
    let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
    let csv_store = AssetBalanceCsvStore::new(session, balances, dividends, lookup, data_ops);
    let busy_csv = csv_store.clone();
    let view_csv = csv_store.clone();
    let modal_csv = csv_store.clone();
    let alert_csv = csv_store.clone();
    let rail_csv = csv_store.clone();
    let alert_ops = data_ops;
    let busy_ops = data_ops;
    let alert_session = session;
    let csv_slot = csv_store.csv;
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            lookup.update(|store| store.clear());
            balances.set(None);
            dividends.set(DividendMaps::default());
            data_ops.update(DataOps::reset);
            reset_query.set(clear_search_query());
            show_all.set(false);
            return;
        }
        lookup.update(|store| store.clear_if_stale(generation));
        if balances
            .get_untracked()
            .is_some_and(|(cached, _)| cached == generation)
        {
            return;
        }
        load_asset_balances(
            session, generation, balances, dividends, lookup, csv_slot, data_ops,
        );
    });
    view! {
        <PageHeader
            title="資産管理"
            eyebrow="Portfolio"
            description="保有している銘柄の一覧と評価額を確認できます。"
        />
        <div
            class="mt-2"
            aria-busy=move || {
                !busy_session.loaded.get()
                    || busy_csv.csv_busy()
                    || busy_ops.with(|ops| !ops.inflight.is_empty())
                    || (busy_session.user.get().is_some()
                        && balances
                            .get()
                            .filter(|(cached, _)| {
                                *cached == busy_session.generation.get()
                            })
                            .is_none())
            }
        >
            <div
                class="grid gap-4 lg:grid-cols-[minmax(0,1fr)_19rem] lg:items-start xl:gap-5 xl:grid-cols-[minmax(0,1fr)_20rem]"
                data-testid="assetbalance-workspace"
            >
                // DOM 順は rail 先(キーボード・読み上げ順のため)、sm 以上は order で見た目を main 先に戻す
                <aside class="order-1 sm:order-2" data-testid="assetbalance-utility-rail">
                    <div class="overflow-hidden rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_18px_58px_-42px_rgba(15,23,42,0.9)] backdrop-blur-sm divide-y divide-slate-950/10">
                        <AssetBalanceCsvSection store=view_csv.clone() />
                        {move || {
                            // 一覧取得エラーは CSV エラーより優先して同じ位置に出す
                            let generation = alert_session.generation.get();
                            balances
                                .with(|slot| match slot {
                                    Some((cached, Err(message))) if *cached == generation => {
                                        Some(message.clone())
                                    }
                                    _ => None,
                                })
                                .or_else(|| alert_ops.with(|ops| ops.refresh_error.clone()))
                                .or_else(|| alert_csv.csv_state().error)
                                .map(|message| {
                                    view! {
                                        <div class="px-5 py-4">
                                            <div
                                                class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm font-medium text-red-800"
                                                role="alert"
                                                aria-live="assertive"
                                            >
                                                <strong>"エラー:"</strong>
                                                " "
                                                {message}
                                            </div>
                                        </div>
                                    }
                                })
                        }}
                        {move || {
                            let generation = rail_session.generation.get();
                            let state = rail_csv.csv_state();
                            match balances
                                .with(|slot| resolve_asset_balance(generation, slot, &state))
                            {
                                None => ().into_any(),
                                Some(resolved) => {
                                    view! {
                                        <AssetBalanceRailExtras
                                            rows=resolved.rows
                                            facets=resolved.facets
                                            has_csv_file=resolved.has_csv_file
                                            warning=resolved.warning
                                            search_query=search_query
                                        />
                                    }
                                        .into_any()
                                }
                            }
                        }}
                    </div>
                </aside>
                <div class="min-w-0 order-2 sm:order-1" data-testid="assetbalance-main-stage">
                    {move || {
                        let generation = render_session.generation.get();
                        let state = view_csv.csv_state();
                        match balances
                            .with(|slot| resolve_asset_balance(generation, slot, &state))
                        {
                            None => {
                                show_all.set(false);
                                view! { <CsvStatusMessage text="データを読み込んでいます..." /> }
                                    .into_any()
                            }
                            Some(resolved) => {
                                view! {
                                    <AssetBalanceMainContent
                                        state=state
                                        rows=resolved.rows
                                        summary=resolved.summary
                                        has_csv_file=resolved.has_csv_file
                                        search_query=search_query
                                        dividends=dividends
                                        show_all=show_all
                                        lookup=lookup
                                        generation=generation
                                    />
                                }
                                    .into_any()
                            }
                        }
                    }}
                </div>
            </div>
            {move || {
                if !modal_csv.csv_state().show_delete_confirm {
                    return ().into_any();
                }
                let count = modal_csv.db_count();
                let deleting_csv = modal_csv.clone();
                let deleting = Memo::new(move |_| deleting_csv.csv_state().deleting);
                let confirm = modal_csv.clone();
                let cancel = modal_csv.clone();
                view! {
                    <ConfirmDeleteModal
                        title="資産管理データの全件削除".to_string()
                        description="保存された資産管理データをすべて削除します。".to_string()
                        item_count=count
                        confirm_label="削除する"
                        loading=deleting
                        on_confirm=move || confirm.confirm_delete_all()
                        on_cancel=move || cancel.close_delete_confirm()
                    />
                }
                    .into_any()
            }}
        </div>
    }
}

#[component]
fn AssetBalanceCsvSection(store: AssetBalanceCsvStore) -> impl IntoView {
    let selected = store.clone();
    let selected_file_name = Memo::new(move |_| selected.csv_state().file_name.unwrap_or_default());
    let disabled_store = store.clone();
    let file_input_disabled = Memo::new(move |_| {
        !disabled_store.is_authenticated()
            || disabled_store.csv_busy()
            || disabled_store.list_loading()
    });
    let has_file = store.clone();
    let has_csv_file =
        Memo::new(move |_| has_file.is_authenticated() && has_file.csv_state().file_name.is_some());
    let label_store = store.clone();
    let save_label = Memo::new(move |_| label_store.csv_state().save_label("全件置換で保存"));
    let save_dis = store.clone();
    let save_disabled =
        Memo::new(move |_| save_dis.csv_state().busy() || !can_save_csv(&save_dis.csv_state()));
    let has_db = store.clone();
    let has_db_data = Memo::new(move |_| has_db.is_authenticated() && has_db.db_count() > 0);
    let del_label = store.clone();
    let delete_label = Memo::new(move |_| del_label.csv_state().delete_label(del_label.db_count()));
    let del_dis = store.clone();
    let delete_disabled = Memo::new(move |_| {
        let state = del_dis.csv_state();
        state.saving || state.deleting || del_dis.list_loading()
    });
    let result_store = store.clone();
    let save_result = Memo::new(move |_| result_store.csv_state().import_result);
    let file_select = store.clone();
    let save = store.clone();
    let delete_request = store.clone();
    view! {
        // divide の半透明線は下地色で見え方が変わるため、sm 以上は内側 section 側の線に揃える
        <div class="sm:border-b-0">
            <CsvActionRail
                input_id="csv-file-input-assetbalance"
                toggle_testid="assetbalance-csv-toggle"
                body_id="assetbalance-csv-body"
                section_class="sm:border-b sm:border-slate-950/10"
                on_file_select=move |file| file_select.select_file(file)
                selected_file_name=selected_file_name
                file_input_disabled=file_input_disabled
                has_csv_file=has_csv_file
                save_label=save_label
                on_save=move || save.save_csv()
                save_disabled=save_disabled
                has_db_data=has_db_data
                delete_label=delete_label
                on_delete_request=move || delete_request.open_delete_confirm()
                delete_disabled=delete_disabled
                save_result=save_result
                mode_label="全件置換"
            />
        </div>
    }
}

#[component]
fn CsvStatusMessage(text: &'static str) -> impl IntoView {
    view! {
        <section class="px-5 py-4" role="status" aria-live="polite" aria-atomic="true">
            <div class="flex items-center gap-2 text-slate-600">
                <svg
                    class="animate-spin h-4 w-4"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    aria-hidden="true"
                >
                    <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                    />
                    <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                </svg>
                <p class="text-sm">{text}</p>
            </div>
        </section>
    }
}

#[derive(Clone, Debug)]
struct ChartItem {
    view: HoldingView,
    percentage: Option<f64>,
}

#[component]
fn AssetBalanceSearchCard(query: RwSignal<String>, options: Vec<SearchOption>) -> impl IntoView {
    let options = std::sync::Arc::new(options);
    view! {
        <section role="search" aria-label="資産管理の検索" data-testid="search-card">
            <CollapsibleSearchCard
                has_active_search=Signal::derive(move || !query.get().is_empty())
                is_default_state=Signal::derive(move || query.get().is_empty())
                on_clear=move || query.set(clear_search_query())
            >
                <div class="grid grid-cols-1 gap-3">
                    <div>
                        <label
                            class="mb-1 block text-sm font-bold text-slate-800"
                            for="securities-search"
                        >
                            "銘柄"
                        </label>
                        <select
                            id="securities-search"
                            class="w-full rounded-md border border-slate-300 bg-white px-3 py-2 text-sm"
                            prop:value=move || query.get()
                            on:change=move |event| query.set(event_target_value(&event))
                        >
                            <option value="">"全て表示"</option>
                            {options
                                .iter()
                                .map(|option| {
                                    let option = option.clone();
                                    view! { <option value=option.value>{option.label}</option> }
                                })
                                .collect_view()}
                        </select>
                    </div>
                </div>
            </CollapsibleSearchCard>
        </section>
    }
}

#[component]
fn PortfolioSummary(
    views: Vec<HoldingView>,
    total_count: usize,
    is_filtered: bool,
    on_clear_filter: impl Fn() + 'static,
    summary: Option<AssetBalanceSummary>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
) -> impl IntoView {
    if views.is_empty() {
        show_all.set(false);
        return view! {
            <div>
                <h3>"該当する銘柄がありません"</h3>
                <p>"銘柄の選択を変更するか、絞り込みを解除してください。"</p>
                <div class="mt-3">
                    <button
                        type="button"
                        class="text-sm text-primary hover:underline"
                        on:click=move |_| on_clear_filter()
                    >
                        "絞り込みを解除"
                    </button>
                </div>
            </div>
        }
        .into_any();
    }
    let valuation_items: Vec<ValuationItem> = views
        .iter()
        .map(|view| ValuationItem {
            market_value: serde_json::json!(view.market),
            total_purchase_amount: serde_json::json!(view.purchase),
        })
        .collect();
    let summary_override = summary.as_ref().map(|summary| SummaryOverride {
        total_purchase_amount: serde_json::json!(dec_to_f64(&summary.total_purchase_amount)),
        total_market_value: serde_json::json!(dec_to_f64(&summary.total_market_value)),
    });
    let valuation = summarize_valuation_with_summary(&valuation_items, summary_override.as_ref());
    let kpi_holdings: Vec<KpiHolding> = views
        .iter()
        .map(|view| KpiHolding {
            security_code: view.code.clone(),
            shares: view.shares,
            total_purchase_amount: view.purchase,
        })
        .collect();
    let summary_total = summary
        .as_ref()
        .map(|summary| dec_to_f64(&summary.total_purchase_amount));
    let total_purchase_amount = total_purchase_amount(&kpi_holdings, summary_total);
    let kpi = Memo::new(move |_| {
        calculate_portfolio_kpi(&kpi_holdings, &dividends.get().per_share, summary_total)
    });

    let display_count = views.len();
    let chart_values: Vec<f64> = views.iter().map(|view| view.purchase).collect();
    let chart_markets: Vec<Option<f64>> = views.iter().map(|view| Some(view.market)).collect();
    let plan = chart_plan(&chart_values, &chart_markets);
    let chart_items: Vec<ChartItem> = plan
        .order
        .iter()
        .zip(&plan.percentages)
        .map(|(&index, &percentage)| ChartItem {
            view: views[index].clone(),
            percentage,
        })
        .collect();

    let market_value = valuation.market_value;
    if total_purchase_amount == 0.0 && matches!(market_value, None | Some(0.0)) {
        show_all.set(false);
        return ().into_any();
    }

    view! {
        <div class="mb-3 space-y-4" data-testid="asset-portfolio-summary">
            <section
                class="rounded-xl border border-slate-950/10 bg-white/95 px-5 py-5 shadow-[0_16px_44px_-38px_rgba(15,23,42,0.9)]"
                data-testid="portfolio-kpi-strip"
            >
                <div class="flex flex-col gap-3 border-b border-slate-950/10 pb-4 sm:flex-row sm:items-end sm:justify-between">
                    <div>
                        <h2 class="text-sm font-black text-slate-950">"資産サマリー"</h2>
                    </div>
                    {is_filtered.then(move || {
                        view! {
                            <div class="flex flex-wrap items-center gap-2">
                                <span class="inline-flex items-center rounded-md border border-blue-200 bg-blue-50 px-3 py-1 text-sm font-bold text-blue-700">
                                    {format!("絞り込み中: {display_count}/{total_count}件")}
                                </span>
                                <button
                                    type="button"
                                    class="inline-flex items-center rounded-full border border-slate-200 bg-white px-3 py-1 text-sm font-medium text-slate-600 transition-colors hover:bg-slate-50"
                                    on:click=move |_| on_clear_filter()
                                >
                                    "解除"
                                </button>
                            </div>
                        }
                    })}
                </div>
                <div class="mt-4 sm:hidden" data-testid="portfolio-valuation-summary">
                    <p class="text-sm font-medium text-slate-600">"保有資産の評価額"</p>
                    <p class="mt-1 text-3xl font-black tabular-nums text-slate-950">
                        {market_value.map(format_currency).unwrap_or("—".to_string())}
                    </p>
                    <p class="mt-2 text-sm font-bold tabular-nums text-slate-800">
                        "評価損益 "
                        {match valuation.amount {
                            None => "—".to_string(),
                            Some(amount) => {
                                match valuation.rate {
                                    None => {
                                        format!(
                                            "{}（算出不可）",
                                            format_valuation_amount(Some(amount)),
                                        )
                                    }
                                    Some(rate) => {
                                        format!(
                                            "{}（{}）",
                                            format_valuation_amount(Some(amount)),
                                            format_valuation_rate(Some(rate), 1),
                                        )
                                    }
                                }
                            }
                        }}
                    </p>
                    <p class="mt-1 text-xs text-slate-500">"取込データ時点"</p>
                    {valuation
                        .incomplete
                        .then(|| {
                            view! {
                                <p class="mt-1 text-xs text-amber-700">
                                    "一部の銘柄の評価額が不足しているため、合計を算出できません"
                                </p>
                            }
                        })}
                </div>
                <div
                    class="mt-4 grid grid-cols-2 gap-2 max-sm:hidden sm:gap-3 lg:grid-cols-4"
                    data-testid="portfolio-kpi-grid"
                >
                    <div class="rounded-lg border border-slate-950/10 bg-white px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"合計取得総額"</p>
                        <p
                            class="text-base font-bold sm:text-3xl text-primary tabular-nums"
                            data-negative=move || {
                                if total_purchase_amount < 0.0 { Some("true") } else { None }
                            }
                        >
                            {format_currency(total_purchase_amount)}
                        </p>
                    </div>
                    <div class="rounded-lg border border-teal-200 bg-teal-50 px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"年間配当金額"</p>
                        <p
                            class="text-base font-bold sm:text-3xl text-teal-700 tabular-nums"
                            data-testid="portfolio-annual-dividends"
                        >
                            {move || {
                                kpi.with(|kpi| {
                                    kpi.total_annual_dividends
                                        .map(format_currency)
                                        .unwrap_or("---".to_string())
                                })
                            }}
                        </p>
                    </div>
                    <div class="rounded-lg border border-teal-200 bg-teal-50 px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"配当利回り"</p>
                        <p
                            class="text-base font-bold sm:text-3xl text-teal-700 tabular-nums"
                            data-testid="portfolio-dividend-yield"
                        >
                            {move || {
                                kpi.with(|kpi| {
                                    kpi.dividend_yield
                                        .map(format_percentage_value)
                                        .unwrap_or("---".to_string())
                                })
                            }}
                        </p>
                    </div>
                    <div class="rounded-lg border border-slate-950/10 bg-white px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"保有銘柄数"</p>
                        <p class="text-base font-bold sm:text-3xl text-slate-700 tabular-nums">
                            {if is_filtered {
                                format!("{display_count} / {total_count}")
                            } else {
                                display_count.to_string()
                            }}
                            <span class="ml-1 text-sm font-normal text-slate-500">"銘柄"</span>
                        </p>
                    </div>
                </div>
            </section>

            <div data-testid="portfolio-pie-chart">
                <div class="flex flex-col gap-3 border-b border-slate-950/10 bg-slate-50/80 px-5 py-4 sm:flex-row sm:items-center sm:justify-between">
                    <div>
                        <h3 class="text-sm font-black text-slate-900">
                            "保有内訳"
                            <span class="ml-1 font-medium text-slate-500 sm:hidden">
                                {format!("（保有{display_count}銘柄）")}
                            </span>
                        </h3>
                    </div>
                </div>
                <div class="p-4">
                    <ChartList items=chart_items dividends=dividends show_all=show_all />
                </div>
            </div>
        </div>
    }
        .into_any()
}

#[component]
fn ChartList(
    items: Vec<ChartItem>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
) -> impl IntoView {
    let total = items.len();
    let percentages: Vec<Option<f64>> = items.iter().map(|item| item.percentage).collect();
    let display = Memo::new(move |_| chart_display(total, &percentages, show_all.get()));
    view! {
        <div
            class=move || display.with(|display| display.grid_class)
            data-testid="portfolio-items-grid"
        >
            {move || {
                items
                    .clone()
                    .into_iter()
                    .enumerate()
                    .filter(|(index, _)| *index < display.with(|d| d.visible_count))
                    .map(|(index, item)| {
                        view! {
                            <div>
                                <HoldingCard
                                    item=item.clone()
                                    index=index
                                    dividends=dividends
                                />
                                <HoldingValuationCard item=item dividends=dividends />
                            </div>
                        }
                    })
                    .collect_view()
            }}
            {move || {
                display
                    .with(|d| d.others.clone())
                    .map(|others| {
                        view! {
                            <div class="rounded-lg border border-dashed border-slate-300 bg-slate-50 p-3">
                                <div class="flex items-center justify-between gap-2 mb-2">
                                    <span class="text-sm text-slate-500">
                                        {format!("その他 {}銘柄", others.count)}
                                    </span>
                                    <span class="text-lg font-bold text-slate-500">
                                        {format_fixed_percent(others.percentage, 1)}
                                    </span>
                                </div>
                                <div class="h-2.5 w-full rounded-full bg-slate-200">
                                    <div
                                        class="h-full rounded-full bg-slate-400 transition-all duration-300"
                                        style=format!("width: {}%", others.percentage.min(100.0))
                                    />
                                </div>
                            </div>
                        }
                    })
            }}
        </div>
        {move || {
            display
                .with(|d| d.toggle_label.clone())
                .map(|label| {
                    view! {
                        <button
                            type="button"
                            class="mt-3 w-full rounded-md border border-slate-300 bg-white py-2 text-sm font-medium text-slate-600 hover:bg-slate-50 transition-colors"
                            on:click=move |_| show_all.update(|open| *open = !*open)
                        >
                            {label}
                        </button>
                    }
                })
        }}
    }
}

#[component]
fn HoldingCard(item: ChartItem, index: usize, dividends: RwSignal<DividendMaps>) -> impl IntoView {
    let color = CHART_COLORS[index % CHART_COLORS.len()];
    let code = item.view.code.clone();
    let shares = item.view.shares;
    let average_price = item.view.average_price;
    let dividend =
        Memo::new(move |_| holding_dividend(&code, shares, average_price, &dividends.get()));
    let percentage_text = item
        .percentage
        .map(|percentage| format_fixed_percent(percentage, 1))
        .unwrap_or("-".to_string());
    let bar_width = item
        .percentage
        .map(|percentage| format!("{}%", percentage.min(100.0)))
        .unwrap_or("NaN%".to_string());
    let dividend_class = move |present: bool| {
        if present {
            "mt-0.5 truncate text-[12px] font-semibold text-emerald-600"
        } else {
            "mt-0.5 truncate text-[12px] font-semibold text-slate-500"
        }
    };
    view! {
        <div class="rounded-lg border border-slate-950/10 bg-white px-3.5 py-3 shadow-sm max-sm:hidden">
            <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                    <div class="flex items-center gap-2.5 min-w-0">
                        <span class="h-3 w-3 shrink-0 rounded-sm" style=format!("background-color: {color}") />
                        <div class="flex min-w-0 items-center gap-2" data-testid="portfolio-card-identity">
                            <span
                                class="inline-flex shrink-0 items-center rounded-full bg-blue-50 px-2 py-0.5"
                                data-testid="portfolio-card-code"
                            >
                                <SecurityCodeLink
                                    value=item.view.code.clone()
                                    class="text-[11px] font-semibold tracking-[0.16em] no-underline hover:underline"
                                        .to_string()
                                />
                            </span>
                            <p class="truncate text-[15px] font-semibold text-slate-800" title=item.view.name.clone()>
                                {item.view.name.clone()}
                            </p>
                        </div>
                    </div>
                </div>
                <div class="shrink-0 text-right">
                    <p class="text-xl font-bold text-slate-800">{percentage_text}</p>
                </div>
            </div>

            <div class="mt-2.5 h-2 w-full rounded-full bg-slate-100 max-sm:hidden">
                <div
                    class="h-full rounded-full transition-all duration-300"
                    style=format!("width: {bar_width}; background-color: {color}")
                />
            </div>

            <div
                class="mt-2 grid grid-cols-3 overflow-hidden rounded-md bg-slate-50"
                data-testid="portfolio-card-acquisition-stats"
            >
                <div class="min-w-0 px-2 py-2">
                    <p class="truncate text-[10px] font-medium text-slate-500">"取得総額"</p>
                    <p
                        class="mt-0.5 truncate text-[12px] font-semibold text-slate-800"
                        title=format_currency(item.view.purchase)
                    >
                        {format_currency(item.view.purchase)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-slate-200/80 px-2 py-2">
                    <p class="truncate text-[10px] font-medium text-slate-500">"取得単価"</p>
                    <p
                        class="mt-0.5 truncate text-[12px] font-semibold text-slate-800"
                        title=format_currency(item.view.average_price)
                    >
                        {format_currency(item.view.average_price)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-slate-200/80 px-2 py-2">
                    <p class="truncate text-[10px] font-medium text-slate-500">"数量"</p>
                    <p
                        class="mt-0.5 truncate text-[12px] font-semibold text-slate-800"
                        title=format!("{}株", format_number_value(item.view.shares))
                    >
                        {format!("{}株", format_number_value(item.view.shares))}
                    </p>
                </div>
            </div>

            <div class="mt-2 grid grid-cols-3 overflow-hidden rounded-md bg-emerald-50/55">
                <div class="min-w-0 px-2 py-2 text-xs text-slate-600">
                    <p class="truncate text-[10px] font-medium text-slate-500">"1株配当"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.per_share.is_some()))
                        title=move || dividend.with(format_dividend_per_share)
                    >
                        {move || dividend.with(format_dividend_per_share)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-emerald-100/80 px-2 py-2 text-xs text-slate-600">
                    <p class="truncate text-[10px] font-medium text-slate-500">"年間配当"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.annual.is_some()))
                        title=move || dividend.with(format_dividend_annual)
                    >
                        {move || dividend.with(format_dividend_annual)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-emerald-100/80 px-2 py-2 text-xs text-slate-600">
                    <p class="truncate text-[10px] font-medium text-slate-500">"配当利回り"</p>
                    <p
                        class=move || dividend_class(dividend.with(|d| d.yield_value.is_some()))
                        title=move || dividend.with(format_dividend_yield)
                    >
                        {move || dividend.with(format_dividend_yield)}
                    </p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn HoldingValuationCard(item: ChartItem, dividends: RwSignal<DividendMaps>) -> impl IntoView {
    let open = RwSignal::new(false);
    let detail_id = format!("portfolio-item-detail-{}", item.view.code);
    let valuation = calculate_valuation(
        &serde_json::json!(item.view.market),
        &serde_json::json!(item.view.purchase),
    );
    let market_display = format_currency(item.view.market);
    let current_price_display = format_currency(item.view.current_price);
    let profit_loss = match valuation.amount {
        None => "—".to_string(),
        Some(amount) => match valuation.rate {
            None => format!("{}（算出不可）", format_valuation_amount(Some(amount))),
            Some(rate) => {
                format!(
                    "{}（{}）",
                    format_valuation_amount(Some(amount)),
                    format_valuation_rate(Some(rate), 1),
                )
            }
        },
    };
    let composition = item
        .percentage
        .map(|percentage| format_fixed_percent(percentage, 1))
        .unwrap_or("—".to_string());
    view! {
        <div class="rounded-lg border border-slate-950/10 bg-white shadow-sm sm:hidden" data-testid="portfolio-valuation-card">
            <button
                type="button"
                aria-expanded=move || open.get()
                aria-controls=detail_id.clone()
                on:click=move |_| open.update(|value| *value = !*value)
                class="block min-h-[44px] w-full px-3.5 py-4 text-left"
            >
                <span class="flex min-w-0 items-center gap-2">
                    <span class="min-w-0 flex-1 truncate text-[15px] font-semibold text-slate-800">
                        {item.view.name.clone()}
                    </span>
                    <span
                        class="inline-flex shrink-0 items-center rounded-full bg-blue-50 px-2 py-0.5 text-[11px] font-semibold tracking-[0.16em] text-blue-700"
                        data-testid="portfolio-valuation-card-code"
                    >
                        {item.view.code.clone()}
                    </span>
                </span>
                <span class="mt-2 flex items-baseline justify-between gap-2">
                    <span class="shrink-0 text-xs font-medium text-slate-500">"評価額"</span>
                    <span class="truncate text-base font-bold tabular-nums text-slate-800">
                        {market_display}
                    </span>
                </span>
                <span class="mt-2 flex items-center justify-between gap-2">
                    <span class="shrink-0 text-xs font-medium text-slate-500">"評価損益"</span>
                    <span class="flex min-w-0 items-center gap-1">
                        <span class="truncate text-sm font-bold tabular-nums text-slate-800">
                            {profit_loss}
                        </span>
                        <span aria-hidden="true" class="shrink-0 text-xs text-slate-400">
                            {move || if open.get() { "▴" } else { "▾" }}
                        </span>
                    </span>
                </span>
            </button>
            {move || {
                let dividend = holding_dividend(
                    &item.view.code,
                    item.view.shares,
                    item.view.average_price,
                    &dividends.get(),
                );
                open.get()
                    .then(|| {
                        view! {
                            <div id=detail_id.clone() class="border-t border-slate-950/10 px-3.5 py-3">
                                <dl class="space-y-1.5 text-xs text-slate-600">
                                    <div class="flex items-start justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"銘柄名"</dt>
                                        <dd class="min-w-0 break-words text-right font-semibold text-slate-800">
                                            {item.view.name.clone()}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"取得総額"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {format_currency(item.view.purchase)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"取得単価"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {format_currency(item.view.average_price)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"数量"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {format!("{}株", format_number_value(item.view.shares))}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"現在値"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {current_price_display.clone()}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"取得額構成比"</dt>
                                        <dd class="truncate font-semibold tabular-nums text-slate-800">
                                            {composition.clone()}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"予想年間配当"</dt>
                                        <dd class="truncate font-semibold text-emerald-600">
                                            {format_dividend_annual(&dividend)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"1株配当"</dt>
                                        <dd class="truncate font-semibold text-emerald-600">
                                            {format_dividend_per_share(&dividend)}
                                        </dd>
                                    </div>
                                    <div class="flex items-center justify-between gap-2">
                                        <dt class="shrink-0 font-medium text-slate-500">"取得額基準利回り"</dt>
                                        <dd class="truncate font-semibold text-emerald-600">
                                            {format_dividend_yield(&dividend)}
                                        </dd>
                                    </div>
                                </dl>
                                <p class="mt-2.5 border-t border-slate-100 pt-2.5 text-xs">
                                    <SecurityCodeLink
                                        value=item.view.code.clone()
                                        class="text-xs".to_string()
                                    />
                                    <span class="ml-1 text-slate-500">"の銘柄情報を見る"</span>
                                </p>
                            </div>
                        }
                    })
            }}
        </div>
    }
}

#[cfg(test)]
mod pages_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset_balance_domain::normalize_security_code;
    use crate::dto::SessionUser;
    use crate::security_link::is_searchable_code;
    use crate::session::SessionStore;
    use std::collections::HashMap;

    fn user(id: &str) -> SessionUser {
        SessionUser {
            id: id.to_string(),
            email: format!("{id}@example.com"),
            name: None,
            picture_url: None,
        }
    }

    #[test]
    fn stale_asset_balance_result_is_rejected_after_same_or_different_user_login() {
        let owner = Owner::new();
        owner.with(|| {
            for next_user in [user("alice"), user("bob")] {
                let session = SessionStore::new();
                session.user.set(Some(user("alice")));
                let fetch_generation = session.generation.get_untracked();

                session.mark_unauthenticated();
                session.user.set(Some(next_user));

                assert!(!should_apply_asset_balance_result(
                    &session,
                    fetch_generation
                ));
            }
        });
    }

    #[test]
    fn unauthorized_asset_balance_error_uses_react_message() {
        assert_eq!(ApiError::Http { status: 401 }.message(), "認証が必要です");
    }

    #[test]
    fn currency_formatter_matches_react_cases() {
        assert_eq!(format_currency(850000.0), "¥ 850,000");
        assert_eq!(format_currency(0.0), "¥ 0");
        assert_eq!(format_currency(-250000.0), "¥ -250,000");
        assert_eq!(format_currency(2600.0), "¥ 2,600");
        assert_eq!(format_currency(123.456), "¥ 123.456");
        assert_eq!(format_currency(f64::NAN), "-");
        assert_eq!(format_number_value(100.0), "100");
        assert_eq!(format_number_value(-12345.0), "-12,345");
    }

    #[test]
    fn percentage_formatter_matches_react_cases() {
        assert_eq!(format_percentage_value(2.0), "2.00%");
        assert_eq!(format_percentage_value(1.3770010052107338), "1.38%");
        assert_eq!(format_fixed_percent(5.300076869056115, 1), "5.3%");
        assert_eq!(format_fixed_percent(10.0, 1), "10.0%");
        assert_eq!(format_fixed_percent(8.256880733944955, 1), "8.3%");
        assert_eq!(format_fixed_percent(60.0, 1), "60.0%");
        assert_eq!(format_valuation_amount(Some(60000.0)), "+¥ 60,000");
        assert_eq!(format_valuation_amount(Some(-10000.0)), "¥ -10,000");
        assert_eq!(format_valuation_amount(None), "—");
        assert_eq!(format_valuation_rate(Some(10.0), 1), "+10.0%");
        assert_eq!(format_valuation_rate(Some(-10.0), 1), "-10.0%");
        assert_eq!(format_valuation_rate(None, 1), "—");
    }

    #[test]
    fn holding_dividend_matches_component_cases() {
        let maps = DividendMaps {
            per_share: HashMap::from([("7203".to_string(), 50.0), ("6758".to_string(), 360.0)]),
            status: HashMap::from([
                ("0001".to_string(), "pending".to_string()),
                ("0002".to_string(), "error".to_string()),
                ("0003".to_string(), "zero".to_string()),
            ]),
        };
        let with_data = holding_dividend("7203", 100.0, 2500.0, &maps);
        assert_eq!(with_data.per_share, Some(50.0));
        assert_eq!(with_data.annual, Some(5000.0));
        assert_eq!(with_data.yield_value, Some(2.0));
        assert_eq!(format_dividend_yield(&with_data), "2.00%");

        let missing = holding_dividend("9999", 10.0, 100.0, &maps);
        assert_eq!(missing.per_share, None);
        assert_eq!(format_dividend_per_share(&missing), "---");

        let pending = holding_dividend("0001", 10.0, 100.0, &maps);
        assert_eq!(format_dividend_per_share(&pending), "取得中...");

        let error = holding_dividend("0002", 10.0, 100.0, &maps);
        assert_eq!(format_dividend_annual(&error), "取得失敗");

        let zero = holding_dividend("0003", 10.0, 100.0, &maps);
        assert_eq!(zero.per_share, Some(0.0));
        assert_eq!(zero.annual, Some(0.0));
        assert_eq!(format_dividend_per_share(&zero), "¥ 0");
        assert_eq!(format_dividend_yield(&zero), "---");

        let zero_price = holding_dividend("7203", 10.0, 0.0, &maps);
        assert_eq!(zero_price.yield_value, None);
    }

    #[test]
    fn security_name_normalization_matches_react_cases() {
        assert_eq!(normalize_security_name("ＫＤＤＩ"), "KDDI");
        assert_eq!(normalize_security_name("トヨタ自動車"), "トヨタ自動車");
        assert_eq!(normalize_security_code(" 7203: トヨタ自動車 "), "7203");
        assert!(is_searchable_code("7203"));
        assert!(is_searchable_code("BRK.B"));
        assert!(!is_searchable_code(""));
        assert!(!is_searchable_code("7203: トヨタ"));
    }

    #[test]
    fn number_formatter_matches_intl_cases() {
        assert_eq!(format_number_value(100.0), "100");
        assert_eq!(format_number_value(1.5), "1.5");
        assert_eq!(format_number_value(1.2345), "1.23");
        assert_eq!(format_number_value(-0.001), "-0");
        // React は符号を丸め前の値の `num < 0` で判定するため -0 は `0` と表示する
        assert_eq!(format_number_value(-0.0), "0");
        assert_eq!(format_currency(-0.0), "¥ 0");
        assert_eq!(format_number_value(-12345.678), "-12,345.68");
        // Intl.NumberFormat は toFixed と異なり10進の値で半分以上を切り上げる
        assert_eq!(format_number_value(1.005), "1.01");
        assert_eq!(format_number_value(-1.005), "-1.01");
        assert_eq!(format_number_value(2.675), "2.68");
        assert_eq!(format_number_value(0.995), "1");
        assert_eq!(
            format_currency("0.123456789012345678".parse::<f64>().unwrap()),
            "¥ 0.123456789012346"
        );
        assert_eq!(format_currency(850000.0), "¥ 850,000");
        assert_eq!(format_fixed_percent(f64::NAN, 1), "-");
    }

    #[test]
    fn api_summary_is_ignored_while_searching() {
        let summary = || {
            Some(AssetBalanceSummary {
                total_market_value: Decimal::from(260_000),
                total_purchase_amount: Decimal::from(250_000),
                total_daily_change: Decimal::ZERO,
            })
        };
        assert_eq!(effective_summary(summary(), "", false), summary());
        assert_eq!(effective_summary(summary(), "7203", false), None);
        assert_eq!(effective_summary(summary(), "", true), None);
        assert_eq!(effective_summary(None, "", false), None);
    }

    #[test]
    fn filtered_portfolio_shows_filtered_row_totals_while_searching() {
        let owner = Owner::new();
        owner.with(|| {
            // 行合計と一致しない API summary を使い、画面の金額が表示中の行から来ることを確認する
            let mut toyota = balance_row(7203);
            toyota.security_name = "トヨタ自動車".to_string();
            toyota.total_purchase_amount = rust_decimal_macros::dec!(100000);
            toyota.market_value = rust_decimal_macros::dec!(110000);
            let mut sony = balance_row(6758);
            sony.security_name = "ソニーグループ".to_string();
            sony.total_purchase_amount = rust_decimal_macros::dec!(200000);
            sony.market_value = rust_decimal_macros::dec!(180000);
            let loaded = LoadedAssetBalances {
                total: 2,
                rows: vec![toyota, sony],
                summary: Some(AssetBalanceSummary {
                    total_market_value: rust_decimal_macros::dec!(999999),
                    total_purchase_amount: rust_decimal_macros::dec!(888888),
                    total_daily_change: rust_decimal_macros::dec!(0),
                }),
                facets: None,
                truncated: false,
            };
            let lookup = RwSignal::new(AssetBalanceLookupStore::new());
            lookup.update(|store| store.seed(1, &loaded.rows));

            let filtered = filtered_portfolio(
                &loaded.rows,
                loaded.summary.clone(),
                "7203",
                lookup,
                1,
                false,
            );
            assert_eq!(filtered.views.len(), 1);
            assert_eq!(filtered.views[0].code, "7203");
            assert!(filtered.summary.is_none());

            // PortfolioSummary と同じ手順で画面に出る金額を計算する
            let valuation_items: Vec<ValuationItem> = filtered
                .views
                .iter()
                .map(|view| ValuationItem {
                    market_value: serde_json::json!(view.market),
                    total_purchase_amount: serde_json::json!(view.purchase),
                })
                .collect();
            let summary_override = filtered.summary.as_ref().map(|summary| SummaryOverride {
                total_purchase_amount: serde_json::json!(dec_to_f64(
                    &summary.total_purchase_amount,
                )),
                total_market_value: serde_json::json!(dec_to_f64(&summary.total_market_value)),
            });
            let valuation =
                summarize_valuation_with_summary(&valuation_items, summary_override.as_ref());
            let kpi_holdings: Vec<KpiHolding> = filtered
                .views
                .iter()
                .map(|view| KpiHolding {
                    security_code: view.code.clone(),
                    shares: view.shares,
                    total_purchase_amount: view.purchase,
                })
                .collect();
            let summary_total = filtered
                .summary
                .as_ref()
                .map(|summary| dec_to_f64(&summary.total_purchase_amount));
            let total_purchase = total_purchase_amount(&kpi_holdings, summary_total);
            let dividends = RwSignal::new(DividendMaps {
                per_share: HashMap::from([("7203".to_string(), 50.0)]),
                status: HashMap::new(),
            });
            let kpi = Memo::new(move |_| {
                calculate_portfolio_kpi(&kpi_holdings, &dividends.get().per_share, summary_total)
            });

            assert_eq!(valuation.market_value, Some(110000.0));
            assert_eq!(valuation.amount, Some(10000.0));
            assert_eq!(valuation.rate, Some(10.0));
            assert_eq!(format_currency(total_purchase), "¥ 100,000");
            kpi.with(|kpi| {
                assert_eq!(kpi.total_purchase_amount, 100000.0);
                assert_eq!(kpi.total_annual_dividends, Some(5000.0));
                assert_eq!(kpi.dividend_yield, Some(5.0));
                assert_eq!(kpi.holdings_count, 1);
            });

            let unfiltered =
                filtered_portfolio(&loaded.rows, loaded.summary.clone(), "", lookup, 1, false);
            assert_eq!(unfiltered.views.len(), 2);
            assert_eq!(
                unfiltered
                    .summary
                    .map(|summary| summary.total_purchase_amount),
                Some(rust_decimal_macros::dec!(888888))
            );
        });
    }

    #[test]
    fn show_all_resets_when_summary_stops_drawing_the_chart() {
        let owner = Owner::new();
        owner.with(|| {
            let dividends = RwSignal::new(DividendMaps::default());
            let show_all = RwSignal::new(true);
            let views: Vec<HoldingView> =
                (0..21).map(|id| holding_view(&balance_row(id))).collect();
            let summary_view = |views: Vec<HoldingView>| {
                PortfolioSummary(
                    PortfolioSummaryProps::builder()
                        .views(views)
                        .total_count(21)
                        .is_filtered(true)
                        .on_clear_filter(|| {})
                        .summary(None)
                        .dividends(dividends)
                        .show_all(show_all)
                        .build(),
                )
            };

            let _ = summary_view(views.clone());
            assert!(show_all.get_untracked());

            let _ = summary_view(Vec::new());
            assert!(!show_all.get_untracked());

            show_all.set(true);
            let mut zero = balance_row(9999);
            zero.total_purchase_amount = rust_decimal_macros::dec!(0);
            zero.market_value = rust_decimal_macros::dec!(0);
            let _ = summary_view(vec![holding_view(&zero)]);
            assert!(!show_all.get_untracked());

            let _ = summary_view(views);
            assert!(!show_all.get_untracked());
            let display = chart_display(21, &[Some(100.0 / 21.0); 21], show_all.get_untracked());
            assert_eq!(display.visible_count, 20);
            assert_eq!(
                display.toggle_label.as_deref(),
                Some("残り1銘柄を表示（全21）")
            );
        });
    }

    #[test]
    fn dividend_maps_apply_only_to_current_generation() {
        let owner = Owner::new();
        owner.with(|| {
            let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
            let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
            let fresh = DividendMaps {
                per_share: HashMap::from([("7203".to_string(), 50.0)]),
                status: HashMap::new(),
            };
            apply_dividend_maps(&balances, &dividends, 7, fresh.clone());
            assert!(balances.get_untracked().is_none());
            assert!(dividends.get_untracked().per_share.is_empty());
            balances.set(Some((
                7,
                Ok(LoadedAssetBalances {
                    rows: vec![],
                    total: 0,
                    summary: None,
                    facets: None,
                    truncated: false,
                }),
            )));
            apply_dividend_maps(&balances, &dividends, 8, fresh.clone());
            assert!(dividends.get_untracked().per_share.is_empty());
            apply_dividend_maps(&balances, &dividends, 7, fresh);
            assert_eq!(dividends.get_untracked().per_share.get("7203"), Some(&50.0));
        });
    }

    #[test]
    fn dividend_update_does_not_notify_balance_view() {
        // 配当更新で balances の購読者(ページ側の動的 view)を再実行させないこと。
        // 再実行されると子コンポーネントが作り直され、開閉状態が失われる。
        let owner = Owner::new();
        owner.with(|| {
            let balances: RwSignal<BalanceSlot> = RwSignal::new(Some((
                7,
                Ok(LoadedAssetBalances {
                    rows: vec![],
                    total: 0,
                    summary: None,
                    facets: None,
                    truncated: false,
                }),
            )));
            let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
            let runs = RwSignal::new(0u32);
            let balance_view = Memo::new(move |_| {
                runs.update(|count| *count += 1);
                balances.get().is_some()
            });
            assert!(balance_view.get());

            apply_dividend_maps(
                &balances,
                &dividends,
                7,
                DividendMaps {
                    per_share: HashMap::from([("7203".to_string(), 50.0)]),
                    status: HashMap::new(),
                },
            );

            assert!(balance_view.get());
            assert_eq!(runs.get_untracked(), 1);
            assert_eq!(dividends.get_untracked().per_share.get("7203"), Some(&50.0));
        });
    }

    fn balance_row(id: usize) -> AssetBalance {
        AssetBalance {
            id: format!("id-{id}"),
            security_code: format!("{id:04}"),
            security_name: "銘柄".to_string(),
            shares: rust_decimal_macros::dec!(100),
            executing_shares: rust_decimal_macros::dec!(0),
            average_purchase_price: rust_decimal_macros::dec!(2500),
            total_purchase_amount: rust_decimal_macros::dec!(250000),
            current_price: rust_decimal_macros::dec!(2600),
            daily_change: rust_decimal_macros::dec!(50),
            market_value: rust_decimal_macros::dec!(260000),
            profit_loss_rate: rust_decimal_macros::dec!(4),
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn balance_page(
        range: std::ops::Range<usize>,
        total: i64,
        summary_total: Option<Decimal>,
    ) -> AssetBalanceListResponse {
        AssetBalanceListResponse {
            data: range.map(balance_row).collect(),
            total,
            page: 1,
            per_page: ASSET_BALANCE_LIST_PER_PAGE as i64,
            summary: summary_total.map(|total_purchase_amount| AssetBalanceSummary {
                total_market_value: rust_decimal_macros::dec!(0),
                total_purchase_amount,
                total_daily_change: rust_decimal_macros::dec!(0),
            }),
            facets: None,
        }
    }

    #[test]
    fn asset_balance_pages_join_all_pages_in_order() {
        let mut pages = AssetBalancePages::new();
        assert_eq!(pages.next_page(), 1);
        let mut first = balance_page(0..1000, 2300, Some(rust_decimal_macros::dec!(10)));
        first.facets = Some(SearchFacets {
            securities: Some(vec![crate::dto::FacetOption {
                value: "7203".to_string(),
                label: "トヨタ自動車".to_string(),
                count: None,
            }]),
            ..SearchFacets::default()
        });
        assert!(pages.push(first));
        assert_eq!(pages.next_page(), 2);
        let mut second = balance_page(1000..2000, 2300, Some(rust_decimal_macros::dec!(20)));
        second.facets = Some(SearchFacets::default());
        assert!(pages.push(second));
        assert_eq!(pages.next_page(), 3);
        assert!(!pages.push(balance_page(2000..2300, 2300, None)));

        let loaded = pages.finish();
        assert_eq!(loaded.rows.len(), 2300);
        assert_eq!(loaded.total, 2300);
        assert_eq!(loaded.rows[0].id, "id-0");
        assert_eq!(loaded.rows[2299].id, "id-2299");
        // summary・facets は1ページ目のものだけを採用し、以降のページのものは捨てる
        assert_eq!(
            loaded.summary.map(|summary| summary.total_purchase_amount),
            Some(rust_decimal_macros::dec!(10))
        );
        assert_eq!(
            loaded
                .facets
                .and_then(|facets| facets.securities)
                .map(|securities| securities.len()),
            Some(1)
        );
    }

    #[test]
    fn asset_balance_pages_stop_when_first_page_is_short() {
        let mut pages = AssetBalancePages::new();
        assert!(!pages.push(balance_page(0..3, 3, Some(rust_decimal_macros::dec!(10)))));
        let loaded = pages.finish();
        assert_eq!(loaded.rows.len(), 3);
        assert_eq!(loaded.total, 3);
        assert!(loaded.summary.is_some());
    }

    fn csv_store(
        session: &SessionStore,
        balances: RwSignal<BalanceSlot>,
        dividends: RwSignal<DividendMaps>,
    ) -> AssetBalanceCsvStore {
        AssetBalanceCsvStore::new(
            *session,
            balances,
            dividends,
            RwSignal::new(AssetBalanceLookupStore::new()),
            RwSignal::new(DataOps::default()),
        )
    }

    fn csv_upload_response(inserted: usize) -> CsvUploadResponse {
        CsvUploadResponse {
            inserted,
            skipped: 0,
            errors: vec![],
        }
    }

    fn csv_preview_response(rows: Vec<serde_json::Value>) -> CsvPreviewResponse {
        CsvPreviewResponse {
            total_rows: 1,
            valid_rows: 1,
            errors: vec![],
            rows,
        }
    }

    fn csv_preview_row(code: &str) -> serde_json::Value {
        serde_json::json!({
            "security_code": code,
            "security_name": "銘柄",
            "shares": 100,
            "average_purchase_price": 2500,
            "market_value": 260000
        })
    }

    #[test]
    fn csv_state_is_scoped_to_session_generation() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let store = csv_store(
                &session,
                RwSignal::new(None),
                RwSignal::new(DividendMaps::default()),
            );
            store.update_csv(generation, |state| {
                state.file_name = Some("asset.csv".to_string());
            });
            assert_eq!(store.csv_state().file_name.as_deref(), Some("asset.csv"));

            session.mark_unauthenticated();
            assert_eq!(
                store.csv_state(),
                CsvTabState::default(),
                "ログアウトでCSV状態は見えなくなる"
            );

            session.user.set(Some(user("bob")));
            assert_eq!(
                store.csv_state(),
                CsvTabState::default(),
                "別ユーザーでも見えない"
            );
        });
    }

    #[test]
    fn stale_generation_csv_results_are_discarded() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let store = csv_store(
                &session,
                RwSignal::new(None),
                RwSignal::new(DividendMaps::default()),
            );
            store.update_csv(generation, |state| {
                state.file_name = Some("asset.csv".to_string());
                state.previewing = true;
            });

            session.mark_unauthenticated();
            session.user.set(Some(user("bob")));

            assert!(store
                .apply_preview_result(
                    generation,
                    Ok(csv_preview_response(vec![csv_preview_row("7203")])),
                )
                .is_none());
            assert!(!store.apply_upload_result(generation, Ok(csv_upload_response(2))));
            store.apply_delete_result(generation, Ok(()));

            // 古い世代のスロットは進行中のまま残るだけで、現在世代からは見えない
            let slot = store.csv.get_untracked();
            assert!(matches!(
                slot,
                Some((cached, ref state))
                    if cached == generation && state.previewing && state.preview.is_none()
            ));
            assert_eq!(store.csv_state(), CsvTabState::default());
        });
    }

    #[test]
    fn preview_success_collects_security_codes_and_failure_shows_error() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let store = csv_store(
                &session,
                RwSignal::new(None),
                RwSignal::new(DividendMaps::default()),
            );
            store.update_csv(generation, |state| {
                state.file_name = Some("asset.csv".to_string());
                state.previewing = true;
            });

            let codes = store.apply_preview_result(
                generation,
                Ok(csv_preview_response(vec![
                    csv_preview_row("7203"),
                    csv_preview_row("6758"),
                    csv_preview_row("7203"),
                ])),
            );
            assert_eq!(
                codes,
                Some(vec!["6758".to_string(), "7203".to_string()]),
                "プレビュー行の銘柄コードが配当取得の対象になる"
            );
            let state = store.csv_state();
            assert!(!state.previewing);
            assert_eq!(
                state.preview.as_ref().map(|preview| preview.rows.len()),
                Some(3)
            );

            store.update_csv(generation, |state| {
                state.file_name = Some("asset.csv".to_string());
                state.previewing = true;
            });
            assert!(store
                .apply_preview_result(generation, Err(ApiError::Http { status: 500 }))
                .is_none());
            let state = store.csv_state();
            assert!(!state.previewing);
            assert_eq!(
                state.error.as_deref(),
                Some("サーバーエラーが発生しました"),
                "資産管理はプレビュー失敗を画面に出す"
            );
            assert_eq!(state.file_name.as_deref(), Some("asset.csv"));
        });
    }

    #[test]
    fn save_success_clears_file_and_preview_and_requests_reload() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let store = csv_store(
                &session,
                RwSignal::new(None),
                RwSignal::new(DividendMaps::default()),
            );
            store.update_csv(generation, |state| {
                state.file_name = Some("asset.csv".to_string());
                state.preview = Some(crate::csv_flow::CsvPreview {
                    valid_rows: 1,
                    rows: vec![AssetBalanceCsvRow::default()],
                    ..Default::default()
                });
                state.saving = true;
            });

            assert!(
                store.apply_upload_result(generation, Ok(csv_upload_response(2))),
                "成功時は一覧の再取得を要求する"
            );
            let state = store.csv_state();
            assert!(!state.saving);
            assert!(state.file_name.is_none());
            assert!(state.preview.is_none());
            assert_eq!(state.import_result.map(|result| result.inserted), Some(2));
            assert!(store.csv_file.get_untracked().is_none());
        });
    }

    #[test]
    fn save_error_keeps_preview_and_sets_message() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let store = csv_store(
                &session,
                RwSignal::new(None),
                RwSignal::new(DividendMaps::default()),
            );
            store.update_csv(generation, |state| {
                state.file_name = Some("asset.csv".to_string());
                state.preview = Some(crate::csv_flow::CsvPreview::default());
                state.saving = true;
            });

            assert!(!store.apply_upload_result(generation, Err(ApiError::Http { status: 401 }),));
            let state = store.csv_state();
            assert!(!state.saving);
            assert_eq!(state.file_name.as_deref(), Some("asset.csv"));
            assert!(state.preview.is_some());
            assert_eq!(state.error.as_deref(), Some("認証が必要です"));
        });
    }

    #[test]
    fn save_requires_preview_rows() {
        let state = CsvTabState::<AssetBalanceCsvRow>::default();
        assert!(!can_save_csv(&state));
        let empty = CsvTabState::<AssetBalanceCsvRow> {
            preview: Some(crate::csv_flow::CsvPreview::default()),
            ..Default::default()
        };
        assert!(!can_save_csv(&empty), "有効行0件のプレビューでは保存しない");
        let ready = CsvTabState::<AssetBalanceCsvRow> {
            preview: Some(crate::csv_flow::CsvPreview {
                rows: vec![AssetBalanceCsvRow::default()],
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(can_save_csv(&ready));
    }

    #[test]
    fn delete_requires_open_confirm_and_rejects_double_start() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let store = csv_store(
                &session,
                RwSignal::new(None),
                RwSignal::new(DividendMaps::default()),
            );
            assert!(
                store.try_begin_delete().is_none(),
                "確認を表示していないと削除を開始しない"
            );

            store.open_delete_confirm();
            assert!(store.try_begin_delete().is_some());
            assert!(store.try_begin_delete().is_none(), "削除中は再開始しない");
            let state = store.csv_state();
            assert!(state.deleting);
            assert!(!state.show_delete_confirm);
        });
    }

    #[test]
    fn delete_success_clears_result_and_empties_cached_list() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let balances: RwSignal<BalanceSlot> = RwSignal::new(Some((
                generation,
                Ok(LoadedAssetBalances {
                    rows: vec![balance_row(7203)],
                    total: 1,
                    summary: None,
                    facets: None,
                    truncated: false,
                }),
            )));
            let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps {
                per_share: HashMap::from([("7203".to_string(), 50.0)]),
                status: HashMap::new(),
            });
            let store = csv_store(&session, balances, dividends);
            store.update_csv(generation, |state| {
                state.import_result = Some(csv_upload_response(3));
                state.deleting = true;
            });

            store.apply_delete_result(generation, Ok(()));

            let state = store.csv_state();
            assert!(!state.deleting);
            assert!(state.import_result.is_none(), "削除成功で保存結果を消す");
            let loaded = balances
                .get_untracked()
                .and_then(|(cached, result)| (cached == generation).then_some(result))
                .and_then(|result| result.ok());
            assert_eq!(
                loaded.map(|loaded| loaded.rows.len()),
                Some(0),
                "キャッシュ済みの一覧は空で上書きされる"
            );
            assert!(dividends.get_untracked().per_share.is_empty());
        });
    }

    #[test]
    fn delete_success_without_cached_list_sets_empty() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
            let store = csv_store(&session, balances, RwSignal::new(DividendMaps::default()));
            store.update_csv(generation, |state| state.deleting = true);

            store.apply_delete_result(generation, Ok(()));

            let loaded = balances
                .get_untracked()
                .and_then(|(cached, result)| (cached == generation).then_some(result))
                .and_then(|result| result.ok());
            assert_eq!(
                loaded.map(|loaded| (loaded.rows.len(), loaded.total)),
                Some((0, 0)),
                "一覧未取得でも削除成功は空一覧を確定させ、読み込み表示を残さない"
            );
        });
    }

    #[test]
    fn apply_loaded_asset_balances_replaces_same_generation_list() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let balances: RwSignal<BalanceSlot> = RwSignal::new(Some((
                generation,
                Ok(LoadedAssetBalances {
                    rows: vec![balance_row(6758)],
                    total: 1,
                    summary: None,
                    facets: None,
                    truncated: false,
                }),
            )));
            let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps {
                per_share: HashMap::from([("6758".to_string(), 40.0)]),
                status: HashMap::new(),
            });
            let lookup = RwSignal::new(AssetBalanceLookupStore::new());
            lookup.update(|store| store.seed(generation, &[balance_row(6758)]));

            let new_row = balance_row(7203);
            let (codes, missing) = apply_loaded_asset_balances(
                generation,
                LoadedAssetBalances {
                    rows: vec![new_row],
                    total: 1,
                    summary: None,
                    facets: None,
                    truncated: false,
                },
                balances,
                dividends,
                lookup,
                false,
            );

            let loaded = balances
                .get_untracked()
                .and_then(|(cached, result)| (cached == generation).then_some(result))
                .and_then(|result| result.ok())
                .expect("loaded");
            assert_eq!(loaded.rows.len(), 1);
            assert_eq!(loaded.rows[0].id, "id-7203");
            assert!(dividends.get_untracked().per_share.is_empty());
            assert_eq!(codes, vec!["7203".to_string()]);
            assert!(
                missing.is_empty(),
                "一覧行を seed 済みなので個別再取得の対象はない"
            );
        });
    }

    #[test]
    fn apply_loaded_replaces_same_generation_lookup_values() {
        let owner = Owner::new();
        owner.with(|| {
            let generation = 1;
            let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
            let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
            let lookup = RwSignal::new(AssetBalanceLookupStore::new());
            lookup.update(|store| store.seed(generation, &[balance_row(7203), balance_row(6758)]));

            let mut replaced = balance_row(7203);
            replaced.shares = rust_decimal_macros::dec!(200);
            replaced.market_value = rust_decimal_macros::dec!(520000);
            apply_loaded_asset_balances(
                generation,
                LoadedAssetBalances {
                    rows: vec![replaced],
                    total: 1,
                    summary: None,
                    facets: None,
                    truncated: false,
                },
                balances,
                dividends,
                lookup,
                false,
            );

            let shares =
                lookup.with_untracked(|store| store.get(generation, "7203").map(|row| row.shares));
            assert_eq!(
                shares,
                Some(rust_decimal_macros::dec!(200)),
                "同一世代の置換でも lookup の古い値を残さない"
            );
            assert!(
                lookup.with_untracked(|store| store.get(generation, "6758").is_none()),
                "置換後に消えた銘柄の lookup も消す"
            );
        });
    }

    #[test]
    fn apply_loaded_keeps_dividends_while_preview_active() {
        let owner = Owner::new();
        owner.with(|| {
            let generation = 1;
            let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
            let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps {
                per_share: HashMap::from([("7203".to_string(), 50.0)]),
                status: HashMap::new(),
            });
            let lookup = RwSignal::new(AssetBalanceLookupStore::new());

            apply_loaded_asset_balances(
                generation,
                LoadedAssetBalances {
                    rows: vec![balance_row(7203)],
                    total: 1,
                    summary: None,
                    facets: None,
                    truncated: false,
                },
                balances,
                dividends,
                lookup,
                true,
            );

            assert_eq!(
                dividends.get_untracked().per_share.get("7203"),
                Some(&50.0),
                "プレビュー表示中の一覧適用ではプレビュー行の配当を消さない"
            );
            assert!(matches!(
                balances.get_untracked(),
                Some((cached, Ok(_))) if cached == generation
            ));
        });
    }

    #[test]
    fn list_error_keeps_cached_rows_and_reports_refresh_error() {
        let owner = Owner::new();
        owner.with(|| {
            let generation = 1;
            let balances: RwSignal<BalanceSlot> = RwSignal::new(Some((
                generation,
                Ok(LoadedAssetBalances {
                    rows: vec![balance_row(7203)],
                    total: 1,
                    summary: None,
                    facets: None,
                    truncated: false,
                }),
            )));
            let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
            data_ops.update(DataOps::begin_list_fetch);
            let rev = data_ops.with_untracked(|ops| ops.list_rev);
            data_ops.update(|ops| ops.end_list_fetch(rev));

            apply_list_error(
                generation,
                rev,
                "サーバーエラーが発生しました".to_string(),
                balances,
                data_ops,
            );

            let loaded = balances
                .get_untracked()
                .and_then(|(cached, result)| (cached == generation).then_some(result))
                .and_then(|result| result.ok());
            assert_eq!(
                loaded.map(|loaded| loaded.rows.len()),
                Some(1),
                "再取得失敗でも表示中の一覧を消さない"
            );
            assert_eq!(
                data_ops.with_untracked(|ops| ops.refresh_error.clone()),
                Some("サーバーエラーが発生しました".to_string())
            );
        });
    }

    #[test]
    fn list_error_without_cache_sets_error_slot() {
        let owner = Owner::new();
        owner.with(|| {
            let generation = 1;
            let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
            let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
            data_ops.update(DataOps::begin_list_fetch);
            let rev = data_ops.with_untracked(|ops| ops.list_rev);
            data_ops.update(|ops| ops.end_list_fetch(rev));

            apply_list_error(
                generation,
                rev,
                "認証が必要です".to_string(),
                balances,
                data_ops,
            );

            assert!(matches!(
                balances.get_untracked(),
                Some((cached, Err(_))) if cached == generation
            ));
            assert!(data_ops.with_untracked(|ops| ops.refresh_error.is_none()));
        });
    }

    #[test]
    fn list_error_from_stale_fetch_is_dropped() {
        let owner = Owner::new();
        owner.with(|| {
            let generation = 1;
            let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
            let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
            data_ops.update(DataOps::begin_list_fetch);
            let stale_rev = data_ops.with_untracked(|ops| ops.list_rev);
            data_ops.update(DataOps::begin_list_fetch);

            apply_list_error(
                generation,
                stale_rev,
                "サーバーエラーが発生しました".to_string(),
                balances,
                data_ops,
            );

            assert!(balances.get_untracked().is_none());
            assert!(data_ops.with_untracked(|ops| ops.refresh_error.is_none()));
        });
    }

    #[test]
    fn list_loading_counts_inflight_fetch() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let balances: RwSignal<BalanceSlot> = RwSignal::new(Some((
                generation,
                Ok(LoadedAssetBalances {
                    rows: vec![balance_row(7203)],
                    total: 1,
                    summary: None,
                    facets: None,
                    truncated: false,
                }),
            )));
            let store = csv_store(&session, balances, RwSignal::new(DividendMaps::default()));

            assert!(
                !store.list_loading(),
                "キャッシュ済みで取得中でなければ false"
            );

            store.data_ops.update(DataOps::begin_list_fetch);
            let rev = store.data_ops.with_untracked(|ops| ops.list_rev);
            assert!(
                store.list_loading(),
                "キャッシュがあっても再取得中は true(React の isFetching と同じ)"
            );

            store.data_ops.update(|ops| ops.end_list_fetch(rev));
            assert!(!store.list_loading());
        });
    }

    #[test]
    fn begin_delete_invalidates_inflight_list_result() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let store = csv_store(
                &session,
                RwSignal::new(None),
                RwSignal::new(DividendMaps::default()),
            );
            store.data_ops.update(DataOps::begin_list_fetch);
            let inflight_rev = store.data_ops.with_untracked(|ops| ops.list_rev);

            store.open_delete_confirm();
            assert!(store.try_begin_delete().is_some());
            assert!(
                !store
                    .data_ops
                    .with_untracked(|ops| ops.is_current_list(inflight_rev)),
                "削除開始で進行中の一覧取得結果は適用されなくなる"
            );
        });
    }

    #[test]
    fn delete_success_invalidates_late_list_and_poll_results() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let store = csv_store(
                &session,
                RwSignal::new(None),
                RwSignal::new(DividendMaps::default()),
            );
            store.data_ops.update(DataOps::begin_list_fetch);
            let list_rev = store.data_ops.with_untracked(|ops| ops.list_rev);
            store.data_ops.update(DataOps::next_poll_rev);
            let poll_rev = store.data_ops.with_untracked(|ops| ops.poll_rev);
            store.update_csv(generation, |state| state.deleting = true);

            store.apply_delete_result(generation, Ok(()));

            assert!(!store
                .data_ops
                .with_untracked(|ops| ops.is_current_list(list_rev)));
            assert!(!store
                .data_ops
                .with_untracked(|ops| ops.is_current_poll(poll_rev)));
        });
    }

    #[test]
    fn poll_rev_supersedes_earlier_poll() {
        let owner = Owner::new();
        owner.with(|| {
            let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
            data_ops.update(DataOps::next_poll_rev);
            let first = data_ops.with_untracked(|ops| ops.poll_rev);
            data_ops.update(DataOps::next_poll_rev);
            let second = data_ops.with_untracked(|ops| ops.poll_rev);
            assert!(!data_ops.with_untracked(|ops| ops.is_current_poll(first)));
            assert!(data_ops.with_untracked(|ops| ops.is_current_poll(second)));
        });
    }

    #[test]
    fn stale_list_completion_does_not_clear_inflight_of_current_generation() {
        let owner = Owner::new();
        owner.with(|| {
            let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
            data_ops.update(DataOps::begin_list_fetch);
            let stale_rev = data_ops.with_untracked(|ops| ops.list_rev);
            // ユーザー切替相当のリセット後、旧世代の完了が新世代の取得中を消さない
            data_ops.update(DataOps::reset);
            data_ops.update(DataOps::begin_list_fetch);
            let current_rev = data_ops.with_untracked(|ops| ops.list_rev);

            data_ops.update(|ops| ops.end_list_fetch(stale_rev));
            assert!(data_ops.with_untracked(|ops| !ops.inflight.is_empty()));

            data_ops.update(|ops| ops.end_list_fetch(current_rev));
            assert!(data_ops.with_untracked(|ops| ops.inflight.is_empty()));
        });
    }

    #[test]
    fn begin_file_preview_stops_old_poll_and_clears_dividends() {
        let owner = Owner::new();
        owner.with(|| {
            let session = SessionStore::new();
            session.user.set(Some(user("alice")));
            let generation = session.generation.get_untracked();
            let dividends = RwSignal::new(DividendMaps {
                per_share: HashMap::from([("7203".to_string(), 50.0)]),
                status: HashMap::new(),
            });
            let store = csv_store(&session, RwSignal::new(None), dividends);
            store.data_ops.update(DataOps::next_poll_rev);
            let old_poll_rev = store.data_ops.with_untracked(|ops| ops.poll_rev);

            assert!(store.begin_file_preview(generation, "next.csv".to_string()));

            assert!(!store
                .data_ops
                .with_untracked(|ops| ops.is_current_poll(old_poll_rev)));
            assert!(dividends.get_untracked().per_share.is_empty());
        });
    }
}
