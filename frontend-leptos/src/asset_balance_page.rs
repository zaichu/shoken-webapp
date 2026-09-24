use crate::api::{ApiClient, ApiError};
use crate::asset_balance_domain::{
    calculate_portfolio_kpi, calculate_valuation, chart_percentages, should_include_chart_item,
    summarize_valuation_with_summary, to_fixed, KpiHolding, SummaryOverride, ValuationItem,
};
use crate::dto::{AssetBalance, AssetBalanceListResponse, AssetBalanceSummary};
use crate::session::use_session;
use leptos::prelude::*;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const ASSET_BALANCE_LIST_LIMIT: &str = "1000";
const TOP_ITEMS: usize = 20;

const CHART_COLORS: [&str; 10] = [
    "#3b82f6", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6", "#ec4899", "#06b6d4", "#f97316",
    "#84cc16", "#6366f1",
];

#[derive(Clone, Debug, Default)]
struct DividendMaps {
    per_share: HashMap<String, f64>,
    status: HashMap<String, String>,
}

#[derive(Clone, Debug)]
struct LoadedAssetBalances {
    rows: Vec<AssetBalance>,
    summary: Option<AssetBalanceSummary>,
    dividends: DividendMaps,
}

#[derive(Clone, Debug, Deserialize)]
struct DividendEstimateItem {
    #[serde(default)]
    security_code: String,
    #[serde(default)]
    dividend_per_share: Option<f64>,
    #[serde(default)]
    status: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct DividendBatchResponse {
    #[serde(default)]
    items: Vec<DividendEstimateItem>,
}

#[derive(Clone, Debug, Serialize)]
struct DividendBatchRequest {
    security_codes: Vec<String>,
}

async fn fetch_asset_balances() -> Result<AssetBalanceListResponse, ApiError> {
    ApiClient::read_client()
        .get_json::<AssetBalanceListResponse>(
            "/api/v1/asset-balances",
            &[
                ("page", "1"),
                ("per_page", ASSET_BALANCE_LIST_LIMIT),
                ("include_summary", "true"),
                ("include_facets", "true"),
            ],
        )
        .await
}

async fn fetch_dividend_maps(mut codes: Vec<String>) -> DividendMaps {
    codes.sort();
    codes.dedup();
    if codes.is_empty() {
        return DividendMaps::default();
    }
    let response = ApiClient::default_client()
        .post_json::<DividendBatchRequest, DividendBatchResponse>(
            "/api/v1/dividend-per-share-estimates",
            &DividendBatchRequest {
                security_codes: codes,
            },
        )
        .await;
    let mut maps = DividendMaps::default();
    if let Ok(batch) = response {
        for item in batch.items {
            if let Some(status) = &item.status {
                maps.status
                    .insert(item.security_code.clone(), status.clone());
            }
            if item.status.as_deref() == Some("ok") {
                if let Some(per_share) = item.dividend_per_share {
                    if per_share > 0.0 {
                        maps.per_share.insert(item.security_code, per_share);
                    }
                }
            }
        }
    }
    maps
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

fn format_signed_number(value: f64) -> Option<(bool, String)> {
    if !value.is_finite() {
        return None;
    }
    let text = value.abs().to_string();
    let (integer, fraction) = match text.split_once('.') {
        Some((integer, fraction)) => (integer, Some(fraction)),
        None => (text.as_str(), None),
    };
    let grouped = group_thousands(integer);
    let body = match fraction {
        Some(fraction) => format!("{grouped}.{fraction}"),
        None => grouped,
    };
    Some((value.is_sign_negative(), body))
}

fn format_currency(value: f64) -> String {
    match format_signed_number(value) {
        None => "-".to_string(),
        Some((true, body)) => format!("¥ -{body}"),
        Some((false, body)) => format!("¥ {body}"),
    }
}

fn format_number_value(value: f64) -> String {
    match format_signed_number(value) {
        None => "-".to_string(),
        Some((true, body)) => format!("-{body}"),
        Some((false, body)) => body,
    }
}

fn format_fixed_percent(value: f64, decimals: u32) -> String {
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

fn normalize_security_name(name: &str) -> String {
    name.chars()
        .map(|character| match character {
            'Ａ'..='Ｚ' | 'ａ'..='ｚ' | '０'..='９' => {
                char::from_u32(character as u32 - 0xfee0).unwrap_or(character)
            }
            _ => character,
        })
        .collect()
}

fn is_searchable_code(code: &str) -> bool {
    !code.is_empty()
        && code
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '.')
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

#[derive(Clone, Debug)]
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
    let balances = RwSignal::new(None::<(u64, Result<LoadedAssetBalances, String>)>);
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            return;
        }
        if balances
            .get_untracked()
            .is_some_and(|(cached, _)| cached == generation)
        {
            return;
        }
        let session = session.clone();
        leptos::task::spawn_local(async move {
            let result = match fetch_asset_balances().await {
                Err(error) => Err(asset_balance_error_message(&error)),
                Ok(list) => {
                    let codes: Vec<String> = list
                        .data
                        .iter()
                        .map(|row| row.security_code.clone())
                        .collect();
                    let dividends = fetch_dividend_maps(codes).await;
                    Ok(LoadedAssetBalances {
                        rows: list.data,
                        summary: list.summary,
                        dividends,
                    })
                }
            };
            if should_apply_asset_balance_result(&session, generation) {
                balances.set(Some((generation, result)));
            }
        });
    });
    view! {
        <div class="page-surface">
            <div class="mb-5 max-sm:mb-2">
                <div class="flex flex-col gap-3 border-l-4 border-amber-500 pl-4 sm:flex-row sm:items-end sm:justify-between">
                    <div>
                        <p class="mb-1 text-[11px] font-bold uppercase tracking-[0.22em] text-slate-500 max-sm:hidden">
                            "Portfolio"
                        </p>
                        <h1 class="text-2xl font-black leading-tight tracking-normal text-slate-950 max-sm:text-lg">
                            "資産管理"
                        </h1>
                        <p class="mt-1 text-sm font-medium text-slate-600 max-sm:hidden">
                            "保有している銘柄の一覧と評価額を確認できます。"
                        </p>
                    </div>
                </div>
            </div>
            <div data-testid="assetbalance-workspace">
                {move || match balances.get().map(|(_, result)| result) {
                    None => view! { <p role="status">"読み込み中..."</p> }.into_any(),
                    Some(Err(message)) => {
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
                            view! {
                                <div>
                                    <h3>"資産管理データがありません"</h3>
                                    <p>"CSVファイルをインポートするか、データを登録してください。"</p>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <PortfolioSummary
                                    rows=loaded.rows
                                    summary=loaded.summary
                                    dividends=loaded.dividends
                                />
                            }
                                .into_any()
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
fn PortfolioSummary(
    rows: Vec<AssetBalance>,
    summary: Option<AssetBalanceSummary>,
    dividends: DividendMaps,
) -> impl IntoView {
    let views: Vec<HoldingView> = rows.iter().map(holding_view).collect();
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
    let kpi = calculate_portfolio_kpi(&kpi_holdings, &dividends.per_share, summary_total);

    let mut chart_views: Vec<HoldingView> = views
        .into_iter()
        .filter(|view| should_include_chart_item(Some(view.purchase), Some(view.market)))
        .collect();
    chart_views.sort_by(|a, b| b.purchase.total_cmp(&a.purchase));
    let chart_values: Vec<f64> = chart_views.iter().map(|view| view.purchase).collect();
    let chart_markets: Vec<Option<f64>> =
        chart_views.iter().map(|view| Some(view.market)).collect();
    let chart_percentages = chart_percentages(&chart_values, &chart_markets);
    let chart_items: Vec<ChartItem> = chart_views
        .into_iter()
        .zip(chart_percentages)
        .map(|(view, percentage)| ChartItem { view, percentage })
        .collect();

    let display_count = rows.len();
    let total_purchase_amount = kpi.total_purchase_amount;
    let market_value = valuation.market_value;
    if total_purchase_amount == 0.0 && matches!(market_value, None | Some(0.0)) {
        return ().into_any();
    }

    let show_all = RwSignal::new(false);
    let annual_dividends = kpi.total_annual_dividends;
    let dividend_yield = kpi.dividend_yield;
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
                            {annual_dividends.map(format_currency).unwrap_or("---".to_string())}
                        </p>
                    </div>
                    <div class="rounded-lg border border-teal-200 bg-teal-50 px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"配当利回り"</p>
                        <p
                            class="text-base font-bold sm:text-3xl text-teal-700 tabular-nums"
                            data-testid="portfolio-dividend-yield"
                        >
                            {dividend_yield.map(format_percentage_value).unwrap_or("---".to_string())}
                        </p>
                    </div>
                    <div class="rounded-lg border border-slate-950/10 bg-white px-4 py-4 shadow-sm">
                        <p class="mb-1 text-xs font-medium text-slate-600">"保有銘柄数"</p>
                        <p class="text-base font-bold sm:text-3xl text-slate-700 tabular-nums">
                            {display_count.to_string()}
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
    dividends: DividendMaps,
    show_all: RwSignal<bool>,
) -> impl IntoView {
    let total = items.len();
    let others_value: Option<f64> = if total <= TOP_ITEMS {
        None
    } else {
        let sum: f64 = items
            .iter()
            .skip(TOP_ITEMS)
            .filter_map(|item| item.percentage)
            .sum();
        if sum > 0.0 {
            Some(sum)
        } else {
            None
        }
    };
    let grid_class = if total <= 1 {
        "grid grid-cols-1 gap-3"
    } else if total == 2 {
        "grid grid-cols-1 gap-3 xl:grid-cols-2"
    } else {
        "grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3"
    };
    view! {
        <div class=grid_class data-testid="portfolio-items-grid">
            {move || {
                items
                    .clone()
                    .into_iter()
                    .enumerate()
                    .filter(|(index, _)| {
                        show_all.get() || total <= TOP_ITEMS || *index < TOP_ITEMS
                    })
                    .map(|(index, item)| {
                        let dividends = dividends.clone();
                        view! {
                            <div>
                                <HoldingCard
                                    item=item.clone()
                                    index=index
                                    dividends=dividends.clone()
                                />
                                <HoldingValuationCard item=item dividends=dividends />
                            </div>
                        }
                    })
                    .collect_view()
            }}
            {move || {
                others_value
                    .filter(|_| !show_all.get())
                    .map(|others| {
                        view! {
                            <div class="rounded-lg border border-dashed border-slate-300 bg-slate-50 p-3">
                                <div class="flex items-center justify-between gap-2 mb-2">
                                    <span class="text-sm text-slate-500">
                                        {format!("その他 {}銘柄", total - TOP_ITEMS)}
                                    </span>
                                    <span class="text-lg font-bold text-slate-500">
                                        {format_fixed_percent(others, 1)}
                                    </span>
                                </div>
                                <div class="h-2.5 w-full rounded-full bg-slate-200">
                                    <div
                                        class="h-full rounded-full bg-slate-400 transition-all duration-300"
                                        style=format!("width: {}%", others.min(100.0))
                                    />
                                </div>
                            </div>
                        }
                    })
            }}
        </div>
        {move || {
            if total <= TOP_ITEMS {
                return None;
            }
            let label = if show_all.get() {
                format!("上位{TOP_ITEMS}件のみ表示")
            } else {
                format!("残り{}銘柄を表示（全{total}）", total - TOP_ITEMS)
            };
            Some(view! {
                <button
                    type="button"
                    class="mt-3 w-full rounded-md border border-slate-300 bg-white py-2 text-sm font-medium text-slate-600 hover:bg-slate-50 transition-colors"
                    on:click=move |_| show_all.update(|open| *open = !*open)
                >
                    {label}
                </button>
            })
        }}
    }
}

#[component]
fn SecurityCodeAnchor(code: String, #[prop(optional)] class: Option<String>) -> impl IntoView {
    if !is_searchable_code(&code) {
        return view! { <span>{code}</span> }.into_any();
    }
    let href = format!("/search?code={}", urlencoding::encode(&code));
    let classes = format!(
        "security-code-link text-blue-700 underline-offset-2 hover:text-blue-900 hover:underline focus-visible:rounded-sm focus-visible:outline-2 focus-visible:outline-blue-500 font-bold{}",
        class.map(|extra| format!(" {extra}")).unwrap_or_default(),
    );
    view! { <a href=href class=classes data-search=code>{code.clone()}</a> }.into_any()
}

#[component]
fn HoldingCard(item: ChartItem, index: usize, dividends: DividendMaps) -> impl IntoView {
    let color = CHART_COLORS[index % CHART_COLORS.len()];
    let dividend = holding_dividend(
        &item.view.code,
        item.view.shares,
        item.view.average_price,
        &dividends,
    );
    let percentage_text = item
        .percentage
        .map(|percentage| format_fixed_percent(percentage, 1))
        .unwrap_or("NaN%".to_string());
    let bar_width = item
        .percentage
        .map(|percentage| format!("{}%", percentage.min(100.0)))
        .unwrap_or("NaN%".to_string());
    let per_share_class = match dividend.per_share {
        Some(_) => "mt-0.5 truncate text-[12px] font-semibold text-emerald-600",
        None => "mt-0.5 truncate text-[12px] font-semibold text-slate-500",
    };
    let annual_class = match dividend.annual {
        Some(_) => "mt-0.5 truncate text-[12px] font-semibold text-emerald-600",
        None => "mt-0.5 truncate text-[12px] font-semibold text-slate-500",
    };
    let yield_class = match dividend.yield_value {
        Some(_) => "mt-0.5 truncate text-[12px] font-semibold text-emerald-600",
        None => "mt-0.5 truncate text-[12px] font-semibold text-slate-500",
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
                                <SecurityCodeAnchor
                                    code=item.view.code.clone()
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
                    <p class=per_share_class title=format_dividend_per_share(&dividend)>
                        {format_dividend_per_share(&dividend)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-emerald-100/80 px-2 py-2 text-xs text-slate-600">
                    <p class="truncate text-[10px] font-medium text-slate-500">"年間配当"</p>
                    <p class=annual_class title=format_dividend_annual(&dividend)>
                        {format_dividend_annual(&dividend)}
                    </p>
                </div>
                <div class="min-w-0 border-l border-emerald-100/80 px-2 py-2 text-xs text-slate-600">
                    <p class="truncate text-[10px] font-medium text-slate-500">"配当利回り"</p>
                    <p class=yield_class title=format_dividend_yield(&dividend)>
                        {format_dividend_yield(&dividend)}
                    </p>
                </div>
            </div>
        </div>
    }
}

#[component]
fn HoldingValuationCard(item: ChartItem, dividends: DividendMaps) -> impl IntoView {
    let open = RwSignal::new(false);
    let detail_id = format!("portfolio-item-detail-{}", item.view.code);
    let dividend = holding_dividend(
        &item.view.code,
        item.view.shares,
        item.view.average_price,
        &dividends,
    );
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
                                    <SecurityCodeAnchor
                                        code=item.view.code.clone()
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
mod tests {
    use super::*;
    use crate::dto::SessionUser;
    use crate::session::SessionStore;

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
        assert!(is_searchable_code("7203"));
        assert!(is_searchable_code("BRK.B"));
        assert!(!is_searchable_code(""));
        assert!(!is_searchable_code("7203: トヨタ"));
    }
}
