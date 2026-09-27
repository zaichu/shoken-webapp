use crate::api::dto::{
    AssetBalanceListResponse, AssetBalanceSummary, DividendListResponse, DividendSummary,
};
use crate::api::{ApiClient, ApiError};
use crate::features::asset_balance::{
    calculate_valuation_from_decimal, format_valuation_amount, format_valuation_rate,
    valuation_tone,
};
use crate::session::{use_session, Generation, SessionStore};
use leptos::prelude::*;
use shared::format::format_currency as format_currency_decimal;
use std::future::Future;

const STATUS_ITEMS: &[(&str, &str, Option<&str>, &str, &str)] = &[
    (
        "銘柄検索",
        "検索",
        Some("/search"),
        "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
        "h-5 w-5 text-slate-950",
    ),
    (
        "資産管理",
        "一覧確認",
        Some("/assetbalance"),
        "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
        "h-5 w-5 text-slate-950",
    ),
    (
        "取引明細",
        "明細確認",
        Some("/receipts"),
        "M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
        "h-5 w-5 text-slate-950",
    ),
    (
        "CSV取込",
        "CSV反映",
        None,
        "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12",
        "h-5 w-5 text-slate-500",
    ),
];

const FLOW_STEPS: &[(&str, &str)] = &[
    ("01", "CSV取得"),
    ("02", "各ページで取込"),
    ("03", "資産と明細を確認"),
];

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="page-surface space-y-7">
            <div>
                <p class="mb-2 text-[11px] font-black uppercase tracking-[0.28em] text-amber-700">
                    "Portfolio Desk"
                </p>
                <h1 class="text-3xl font-black leading-tight tracking-normal text-slate-950 sm:text-4xl">
                    "証券Web"
                </h1>
                <p class="mt-2 max-w-2xl text-sm font-medium text-slate-600">
                    "資産、配当、取引明細をひとつの作業面で確認します。"
                </p>
            </div>

            <HomeOverview />

            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
                {STATUS_ITEMS
                    .iter()
                    .map(|(label, sub, to, icon_d, icon_class)| {
                        let inner = view! {
                            <div class="flex items-start justify-between gap-3">
                                <span class="inline-flex h-10 w-10 items-center justify-center rounded-md border border-slate-950/10 bg-white shadow-sm">
                                    <svg
                                        class={*icon_class}
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                        aria-hidden="true"
                                        focusable="false"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="1.8"
                                            d={*icon_d}
                                        />
                                    </svg>
                                </span>
                                <span class="text-[11px] font-black uppercase tracking-[0.18em] text-slate-500">
                                    {*sub}
                                </span>
                            </div>
                            <p class="mt-4 text-base font-black text-slate-950">{*label}</p>
                        };
                        match to {
                            Some(to) => {
                                view! {
                                    <a
                                        href={*to}
                                        class="feature-card group"
                                    >
                                        {inner}
                                    </a>
                                }
                                    .into_any()
                            }
                            None => {
                                view! {
                                    <div class="rounded-xl border border-dashed border-slate-300 bg-slate-100/70 px-4 py-4">
                                        {inner}
                                        <p class="mt-2 text-xs font-medium text-slate-500">"各ページから取込可能"</p>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    })
                    .collect_view()}
            </div>

            <section class="border-t border-slate-950/10 pt-5">
                <h2 class="text-sm font-black text-slate-950">"データ確認フロー"</h2>
                <div class="mt-3 grid gap-2 sm:grid-cols-3">
                    {FLOW_STEPS
                        .iter()
                        .map(|(step, text)| {
                            view! {
                                <div class="rounded-lg border border-slate-950/10 bg-white/75 px-3 py-3">
                                    <span class="text-[11px] font-black uppercase tracking-[0.18em] text-amber-700">
                                        {*step}
                                    </span>
                                    <p class="mt-1 text-sm font-bold text-slate-800">{*text}</p>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </section>
        </div>
    }
}

// 外側の None は取得中、内側の None は取得失敗
type SummarySlot<T> = RwSignal<Option<(Generation, Option<T>)>>;

async fn fetch_asset_summary() -> Result<Option<AssetBalanceSummary>, ApiError> {
    ApiClient::read_client()
        .get_json::<AssetBalanceListResponse>(
            "/api/v1/asset-balances",
            &[("per_page", "1"), ("include_summary", "true")],
        )
        .await
        .map(|response| response.summary)
}

async fn fetch_dividend_summary(year: u32) -> Result<Option<DividendSummary>, ApiError> {
    let year = year.to_string();
    ApiClient::read_client()
        .get_json::<DividendListResponse>(
            "/api/v1/dividends",
            &[
                ("per_page", "1"),
                ("year", year.as_str()),
                ("include_summary", "true"),
            ],
        )
        .await
        .map(|response| response.summary)
}

fn load_summary<T: Send + Sync + 'static>(
    session: SessionStore,
    generation: Generation,
    slot: SummarySlot<T>,
    fetch: impl Future<Output = Result<Option<T>, ApiError>> + 'static,
) {
    leptos::task::spawn_local(async move {
        let result = fetch.await;
        if !session.is_current(generation) {
            return;
        }
        if result.as_ref().is_err_and(ApiError::is_unauthorized) {
            session.mark_unauthenticated();
            return;
        }
        slot.set(Some((generation, result.ok().flatten())));
    });
}

fn current_summary<T: Clone + Send + Sync + 'static>(
    slot: SummarySlot<T>,
    generation: Generation,
) -> Option<Option<T>> {
    slot.get()
        .filter(|(cached, _)| *cached == generation)
        .map(|(_, summary)| summary)
}

#[component]
fn HomeOverview() -> impl IntoView {
    let session = use_session();
    let asset: SummarySlot<AssetBalanceSummary> = RwSignal::new(None);
    let dividend: SummarySlot<DividendSummary> = RwSignal::new(None);
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            asset.set(None);
            dividend.set(None);
            return;
        }
        load_summary(session, generation, asset, fetch_asset_summary());
        load_summary(
            session,
            generation,
            dividend,
            fetch_dividend_summary(js_sys::Date::new_0().get_full_year()),
        );
    });
    let snapshot = move || {
        let generation = session.generation.get();
        (
            current_summary(asset, generation),
            current_summary(dividend, generation),
        )
    };
    view! {
        <Show when=move || session.user.get().is_some()>
            <section
                aria-label="資産の概要"
                aria-busy=move || {
                    let (asset, dividend) = snapshot();
                    if asset.is_none() || dividend.is_none() { "true" } else { "false" }
                }
                data-testid="home-overview"
            >
                {move || {
                    let (asset, dividend) = snapshot();
                    view! { <OverviewTiles asset=asset dividend=dividend /> }
                }}
                <p class="text-xs font-medium text-slate-600" role="status" aria-live="polite">
                    {move || {
                        let (asset, dividend) = snapshot();
                        matches!((asset, dividend), (Some(None), _) | (_, Some(None)))
                            .then(|| {
                                view! {
                                    <span class="mt-2 block">"一部の集計を取得できませんでした。"</span>
                                }
                            })
                    }}
                </p>
            </section>
        </Show>
    }
}

#[component]
fn OverviewTiles(
    asset: Option<Option<AssetBalanceSummary>>,
    dividend: Option<Option<DividendSummary>>,
) -> impl IntoView {
    let asset_busy = asset.is_none();
    let dividend_busy = dividend.is_none();
    let asset = asset.flatten();
    let market = asset
        .as_ref()
        .map(|summary| format_currency_decimal(summary.total_market_value));
    let valuation = asset
        .as_ref()
        .filter(|summary| {
            !(summary.total_market_value.is_zero() && summary.total_purchase_amount.is_zero())
        })
        .map(|summary| {
            calculate_valuation_from_decimal(
                summary.total_market_value,
                summary.total_purchase_amount,
            )
        });
    let (profit_class, profit_negative) =
        valuation_tone(valuation.as_ref().and_then(|valuation| valuation.amount));
    let profit_rate = valuation.as_ref().map(|valuation| match valuation.rate {
        Some(rate) => format_valuation_rate(Some(rate), 1),
        None => "算出不可".to_string(),
    });
    let profit = valuation.map(|valuation| format_valuation_amount(valuation.amount));
    let dividend = dividend
        .flatten()
        .map(|summary| format_currency_decimal(summary.total_net_amount_received));
    view! {
        <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
            <OverviewTile label="評価額" value=market busy=asset_busy />
            <OverviewTile
                label="評価損益"
                value=profit
                busy=asset_busy
                value_class=profit_class
                negative=profit_negative
                note=profit_rate
            />
            <OverviewTile label="今年の配当金(税引)" value=dividend busy=dividend_busy />
        </div>
    }
}

#[component]
fn OverviewTile(
    label: &'static str,
    value: Option<String>,
    busy: bool,
    #[prop(default = "text-slate-950")] value_class: &'static str,
    #[prop(default = None)] negative: Option<&'static str>,
    #[prop(default = None)] note: Option<String>,
) -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-950/10 bg-white px-4 py-4 shadow-sm">
            <p class="text-sm font-medium text-slate-600">{label}</p>
            {if busy {
                view! {
                    <div
                        class="mt-2 h-7 w-32 animate-pulse rounded bg-slate-200"
                        aria-hidden="true"
                    ></div>
                }
                    .into_any()
            } else {
                view! {
                    <p
                        class=format!(
                            "mt-1 flex flex-wrap items-baseline gap-x-2 text-2xl font-black tabular-nums {value_class}",
                        )
                        data-negative=negative
                    >
                        <span class="break-all">{value.unwrap_or_else(|| "—".to_string())}</span>
                        {note
                            .map(|note| {
                                view! {
                                    <span class="whitespace-nowrap text-sm font-bold">{format!("（{note}）")}</span>
                                }
                            })}
                    </p>
                }
                    .into_any()
            }}
        </div>
    }
}
