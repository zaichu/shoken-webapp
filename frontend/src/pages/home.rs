use crate::api::{ApiClient, ApiError};
use crate::asset_balance_domain::calculate_valuation_from_decimal;
use crate::dto::{
    AssetBalanceListResponse, AssetBalanceSummary, DividendListResponse, DividendSummary,
};
use crate::pages::asset_balance::format::{
    dec_to_f64, format_currency, format_valuation_amount, format_valuation_rate, valuation_tone,
};
use crate::session::use_session;
use leptos::prelude::*;

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

#[derive(Clone, Default)]
struct Overview {
    asset: Option<AssetBalanceSummary>,
    dividend: Option<DividendSummary>,
    unauthorized: bool,
}

async fn fetch_overview(year: u32) -> Overview {
    let client = ApiClient::read_client();
    let year = year.to_string();
    let asset = client
        .get_json::<AssetBalanceListResponse>(
            "/api/v1/asset-balances",
            &[("per_page", "1"), ("include_summary", "true")],
        )
        .await;
    let dividend = client
        .get_json::<DividendListResponse>(
            "/api/v1/dividends",
            &[
                ("per_page", "1"),
                ("year", year.as_str()),
                ("include_summary", "true"),
            ],
        )
        .await;
    let unauthorized = [asset.as_ref().err(), dividend.as_ref().err()]
        .into_iter()
        .flatten()
        .any(ApiError::is_unauthorized);
    Overview {
        asset: asset.ok().and_then(|response| response.summary),
        dividend: dividend.ok().and_then(|response| response.summary),
        unauthorized,
    }
}

#[component]
fn HomeOverview() -> impl IntoView {
    let session = use_session();
    let overview: RwSignal<Option<(u64, Overview)>> = RwSignal::new(None);
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            overview.set(None);
            return;
        }
        leptos::task::spawn_local(async move {
            let loaded = fetch_overview(js_sys::Date::new_0().get_full_year()).await;
            if !session.is_current(generation) {
                return;
            }
            if loaded.unauthorized {
                session.mark_unauthenticated();
                return;
            }
            overview.set(Some((generation, loaded)));
        });
    });
    move || {
        session.user.get()?;
        let generation = session.generation.get();
        let loaded = overview
            .get()
            .filter(|(cached, _)| *cached == generation)
            .map(|(_, loaded)| loaded);
        Some(view! { <OverviewTiles loaded=loaded /> })
    }
}

#[component]
fn OverviewTiles(loaded: Option<Overview>) -> impl IntoView {
    let busy = loaded.is_none();
    let Overview {
        asset, dividend, ..
    } = loaded.unwrap_or_default();
    let failed = !busy && (asset.is_none() || dividend.is_none());
    let market = asset
        .as_ref()
        .map(|summary| format_currency(dec_to_f64(&summary.total_market_value)));
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
        .as_ref()
        .map(|summary| format_currency(dec_to_f64(&summary.total_net_amount_received)));
    view! {
        <section
            aria-label="資産の概要"
            aria-busy=if busy { "true" } else { "false" }
            data-testid="home-overview"
        >
            <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
                <OverviewTile label="評価額" value=market busy=busy />
                <OverviewTile
                    label="評価損益"
                    value=profit
                    busy=busy
                    value_class=profit_class
                    negative=profit_negative
                    note=profit_rate
                />
                <OverviewTile label="今年の配当金(税引)" value=dividend busy=busy />
            </div>
            {failed
                .then(|| {
                    view! {
                        <p class="mt-2 text-xs font-medium text-slate-600" role="status">
                            "一部の集計を取得できませんでした。"
                        </p>
                    }
                })}
        </section>
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
                            "mt-1 whitespace-nowrap text-2xl font-black tabular-nums {value_class}",
                        )
                        data-negative=negative
                    >
                        {value.unwrap_or_else(|| "—".to_string())}
                        {note
                            .map(|note| {
                                view! {
                                    <span class="ml-2 text-sm font-bold">{format!("（{note}）")}</span>
                                }
                            })}
                    </p>
                }
                    .into_any()
            }}
        </div>
    }
}
