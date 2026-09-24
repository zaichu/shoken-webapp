use crate::api::{ApiClient, ApiError};
use crate::asset_balance_domain::{
    calculate_portfolio_kpi, calculate_valuation, intl_fixed, normalize_security_name,
    summarize_valuation_with_summary, to_fixed, total_purchase_amount, KpiHolding, SummaryOverride,
    ValuationItem,
};
use crate::asset_balance_lookup::{fetch_single_asset_balance, AssetBalanceLookupStore};
use crate::asset_balance_portfolio::{chart_display, chart_plan};
use crate::asset_balance_search::{
    asset_balance_search_options, clear_search_query, filter_asset_balances,
};
use crate::dividend_per_share::{
    dividend_maps_from_batch, dividend_pending_max_retries, fetch_dividend_batch,
    unique_sorted_codes, DividendMaps, DIVIDEND_NETWORK_MAX_RETRIES, DIVIDEND_RETRY_DELAY_MS,
};
use crate::dto::{AssetBalance, AssetBalanceListResponse, AssetBalanceSummary, SearchFacets};
use crate::receipts_pagination::PageCollector;
use crate::receipts_search::SearchOption;
use crate::security_link::SecurityCodeLink;
use crate::session::use_session;
use crate::ui::{Loading, PageHeader};
use leptos::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

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
        let truncated = self.collector.truncated();
        LoadedAssetBalances {
            rows: self.collector.into_rows(),
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
    session: crate::session::SessionStore,
    generation: u64,
    codes: Vec<String>,
    balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
) {
    let max_pending = dividend_pending_max_retries(codes.len());
    let mut pending_used = 0u32;
    let mut network_used = 0u32;
    loop {
        if !session.is_current(generation) {
            break;
        }
        match fetch_dividend_batch(&codes).await {
            Ok(batch) => {
                let (maps, has_pending) = dividend_maps_from_batch(&batch);
                if session.is_current(generation) {
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
) -> Option<AssetBalanceSummary> {
    // 検索中は API の summary が絞り込み前の全体集計のままなので使わない
    if query.is_empty() {
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
    loaded: &LoadedAssetBalances,
    query: &str,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: u64,
) -> FilteredPortfolio {
    let views = filter_asset_balances(&loaded.rows, query)
        .into_iter()
        .map(|row| {
            let resolved = lookup
                .with(|store| store.get(generation, &row.security_code).cloned())
                .unwrap_or_else(|| row.clone());
            holding_view(&resolved)
        })
        .collect();
    FilteredPortfolio {
        views,
        summary: effective_summary(loaded.summary.clone(), query),
    }
}

fn asset_balance_error_message(error: &ApiError) -> String {
    if error.is_unauthorized() {
        error.user_message()
    } else {
        "データ取得に失敗しました".to_string()
    }
}

fn should_apply_asset_balance_result(
    session: &crate::session::SessionStore,
    generation: u64,
) -> bool {
    session.is_current(generation)
}

fn dec_to_f64(value: &Decimal) -> f64 {
    value.to_f64().unwrap_or(0.0)
}

fn group_thousands(digits: &str) -> String {
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, byte) in digits.bytes().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(byte as char);
    }
    grouped
}

// 符号は呼び出し側が丸め前の値で `value < 0` と判定する（React の
// `formatSignedAbsNumber` と同じく `-0.0` は負としない）。
// この関数は絶対値の桁区切りだけを返す。
fn format_abs_number(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let text = value.abs().to_string();
    let (integer, fraction) = match text.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (text.as_str(), None),
    };
    let grouped = group_thousands(integer);
    Some(match fraction {
        Some(fraction) => format!("{grouped}.{fraction}"),
        None => grouped,
    })
}

fn format_currency(value: f64) -> String {
    match format_abs_number(to_fixed(value, 15)) {
        None => "-".to_string(),
        Some(body) if value < 0.0 => format!("¥ -{body}"),
        Some(body) => format!("¥ {body}"),
    }
}

fn format_number_value(value: f64) -> String {
    match format_abs_number(intl_fixed(value, 2)) {
        None => "-".to_string(),
        Some(body) if value < 0.0 => format!("-{body}"),
        Some(body) => body,
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

#[component]
pub fn AssetBalancePage() -> impl IntoView {
    let session = use_session();
    let render_session = session;
    let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
    let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
    let lookup = RwSignal::new(AssetBalanceLookupStore::new());
    let search_query = RwSignal::new(String::new());
    let reset_query = search_query;
    // ページ側に持ち上げた show_all はアンマウントで破棄されないため、グラフを描かない分岐では false に戻す
    let show_all = RwSignal::new(false);
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            lookup.update(|store| store.clear());
            balances.set(None);
            dividends.set(DividendMaps::default());
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
        leptos::task::spawn_local(async move {
            match fetch_asset_balances().await {
                Err(error) => {
                    if should_apply_asset_balance_result(&session, generation) {
                        balances.set(Some((generation, Err(asset_balance_error_message(&error)))));
                    }
                }
                Ok(loaded) => {
                    if !should_apply_asset_balance_result(&session, generation) {
                        return;
                    }
                    lookup.update(|store| store.seed(generation, &loaded.rows));
                    let codes = unique_sorted_codes(
                        &loaded
                            .rows
                            .iter()
                            .map(|row| row.security_code.clone())
                            .collect::<Vec<_>>(),
                    );
                    dividends.set(DividendMaps::default());
                    balances.set(Some((generation, Ok(loaded))));
                    let missing: Vec<String> = codes
                        .iter()
                        .filter(|code| {
                            lookup.with_untracked(|store| store.needs_fetch(generation, code, true))
                        })
                        .cloned()
                        .collect();
                    if !missing.is_empty() {
                        let session = session;
                        leptos::task::spawn_local(async move {
                            for code in missing {
                                if !session.is_current(generation) {
                                    break;
                                }
                                if let Ok(rows) =
                                    fetch_single_asset_balance(&ApiClient::read_client(), &code)
                                        .await
                                {
                                    if !session.is_current(generation) {
                                        break;
                                    }
                                    lookup.update(|store| {
                                        store.store_single(generation, &code, rows);
                                    });
                                }
                            }
                        });
                    }
                    if !codes.is_empty() {
                        leptos::task::spawn_local(poll_dividend_maps(
                            session, generation, codes, balances, dividends,
                        ));
                    }
                }
            }
        });
    });
    view! {
        <div class="page-surface">
            <PageHeader
                title="資産管理"
                eyebrow="Portfolio"
                description="保有している銘柄の一覧と評価額を確認できます。"
            />
            <div data-testid="assetbalance-workspace">
                {move || {
                    let current = render_session.generation.get();
                    match balances
                        .get()
                        .filter(|(cached, _)| *cached == current)
                        .map(|(_, result)| result)
                    {
                        None => {
                            show_all.set(false);
                            view! { <Loading /> }.into_any()
                        }
                        Some(Err(message)) => {
                            show_all.set(false);
                            view! {
                                <div role="alert">
                                    <strong>"エラー:"</strong>
                                    " "
                                    {message}
                                </div>
                            }
                                .into_any()
                        }
                        Some(Ok(loaded)) => {
                            if loaded.rows.is_empty() {
                                show_all.set(false);
                                view! {
                                    <div>
                                        <h3>"資産管理データがありません"</h3>
                                        <p>"CSVファイルをインポートするか、データを登録してください。"</p>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                let query = search_query.get();
                                let options = asset_balance_search_options(
                                    &loaded.rows,
                                    loaded.facets.as_ref(),
                                    false,
                                );
                                let total_count = loaded.rows.len();
                                let truncated = loaded.truncated;
                                let FilteredPortfolio { views, summary } =
                                    filtered_portfolio(&loaded, &query, lookup, current);
                                view! {
                                    {truncated.then(|| {
                                        view! {
                                            <section class="px-5 py-4" role="status" aria-live="polite">
                                                <div class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm font-medium text-amber-900">
                                                    {truncated_list_warning()}
                                                </div>
                                            </section>
                                        }
                                    })}
                                    <AssetBalanceSearchCard query=search_query options=options />
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
                            }
                        }
                    }
                }}
            </div>
        </div>
    }
}

#[derive(Clone, Debug)]
struct ChartItem {
    view: HoldingView,
    percentage: Option<f64>,
}

#[component]
fn AssetBalanceSearchCard(query: RwSignal<String>, options: Vec<SearchOption>) -> impl IntoView {
    view! {
        <div
            class="mb-3 rounded-lg border border-slate-200 bg-white p-4"
            role="search"
            aria-label="資産管理の検索"
            data-testid="search-card"
        >
            <div
                class="mb-3 flex items-center justify-between gap-3"
                data-testid="search-card-header"
            >
                <h2 class="text-sm font-bold text-slate-950">"検索オプション"</h2>
                <button
                    type="button"
                    class="rounded border border-slate-300 px-3 py-1 text-sm"
                    aria-label="検索条件をクリア"
                    data-testid="search-clear-button"
                    disabled=move || query.get().is_empty()
                    on:click=move |_| query.set(clear_search_query())
                >
                    "絞り込み解除"
                </button>
            </div>
            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-4">
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
                            .into_iter()
                            .map(|option| view! {
                                <option value=option.value>{option.label}</option>
                            })
                            .collect_view()}
                    </select>
                </div>
            </div>
        </div>
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
                <div class="mt-4" data-testid="portfolio-valuation-summary">
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
                    class="mt-4 grid grid-cols-2 gap-2 sm:gap-3 lg:grid-cols-4"
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
        assert_eq!(
            asset_balance_error_message(&ApiError::Http { status: 401 }),
            "認証が必要です"
        );
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
        assert_eq!(effective_summary(summary(), ""), summary());
        assert_eq!(effective_summary(summary(), "7203"), None);
        assert_eq!(effective_summary(None, ""), None);
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

            let filtered = filtered_portfolio(&loaded, "7203", lookup, 1);
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

            let unfiltered = filtered_portfolio(&loaded, "", lookup, 1);
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
        assert!(loaded.summary.is_some());
    }
}
