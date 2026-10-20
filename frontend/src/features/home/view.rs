//! ホームダッシュボードの表示。集計は model、取得は store に寄せ、
//! ここではスロットの値を表示用の部品に展開する。

use super::model::{self, CompositionSlice};
use super::store::{self, HomeOverviewData};
use crate::api::dto::{AssetBalance, AssetBalanceSummary, FundBalance};
use crate::features::asset_balance::{
    LoadedAssetBalances, LoadedFundBalances, dec_to_f64, format_currency, format_fixed_percent,
    format_signed_row_valuation_amount, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
    is_positive_valuation, summary_valuation,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::use_session;
use crate::ui::amount::Amount;
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, Spinner, SpinnerSize};
use leptos::prelude::*;

// 内内訳チャートの塗り色。CSS 変数で渡し、テーマの明暗はトークン側の切替に任せる
const CHART_COLORS: &[&str] = &[
    "var(--color-brand-text)",
    "var(--color-accent)",
    "var(--color-gold)",
    "var(--color-fill-strong)",
    "var(--color-brand-muted)",
];
const CURRENT_BAR_COLOR: &str = "var(--color-brand-text)";
const PREVIOUS_BAR_COLOR: &str = "var(--color-fill-strong)";

#[component]
pub fn HomePage() -> impl IntoView {
    let session = use_session();
    let data = store::use_home_overview(session);
    // 再訪では表示済みの値を消さず、取得済みの slot だけ裏で取り直す(未取得は Effect が担う)
    if session.user.get_untracked().is_some() {
        store::refresh_loaded(session, session.generation.get_untracked(), data);
    }
    let import_label = Signal::derive(move || {
        let generation = session.generation.get();
        let mut stamps: Vec<String> = Vec::new();
        if let Some(loaded) = store::current_summary(data.assets, generation).flatten() {
            stamps.extend(loaded.rows.iter().map(|row| row.updated_at.clone()));
        }
        if let Some(loaded) = store::current_summary(data.funds, generation).flatten() {
            stamps.extend(loaded.rows.iter().map(|row| row.updated_at.clone()));
        }
        model::latest_import_label(stamps.iter().map(String::as_str))
    });
    let links: Children = Box::new(move || {
        view! {
            {move || {
                import_label.get().map(|label| {
                    view! {
                        <span class="hidden text-xs font-medium text-text-muted lg:inline">
                            "データ: " {label} " 取り込み"
                        </span>
                    }
                })
            }}
            <a class="page-secondary-link-sm" href="/assetbalance/csv">
                "CSV管理"
            </a>
        }
        .into_any()
    });
    view! {
        <PageFrame title="ホーム" full_bleed=true links=links>
            <div class="page-gutter flex flex-1 flex-col pb-4">
                <HomeDashboard data=data />
            </div>
        </PageFrame>
    }
}

fn stocks_of(assets: &Option<Option<LoadedAssetBalances>>) -> Option<&[AssetBalance]> {
    assets
        .as_ref()
        .and_then(|slot| slot.as_ref())
        .map(|loaded| loaded.rows.as_slice())
}

fn stock_market(assets: &Option<Option<LoadedAssetBalances>>) -> f64 {
    match assets {
        Some(Some(loaded)) => loaded.summary.as_ref().map_or_else(
            || dec_to_f64(&model::stock_summary_from_rows(&loaded.rows).total_market_value),
            |summary| dec_to_f64(&summary.total_market_value),
        ),
        _ => 0.0,
    }
}

fn funds_of(funds: &Option<Option<LoadedFundBalances>>) -> Option<&[FundBalance]> {
    funds
        .as_ref()
        .and_then(|slot| slot.as_ref())
        .map(|loaded| loaded.rows.as_slice())
}

fn fund_market(funds: &Option<Option<LoadedFundBalances>>) -> f64 {
    match funds {
        Some(Some(loaded)) => loaded.summary.as_ref().map_or_else(
            || dec_to_f64(&model::fund_summary_from_rows(&loaded.rows).total_market_value),
            |summary| dec_to_f64(&summary.total_market_value),
        ),
        _ => 0.0,
    }
}

#[component]
fn HomeDashboard(data: HomeOverviewData) -> impl IntoView {
    let session = use_session();
    let snapshot = move || {
        let generation = session.generation.get();
        (
            store::current_summary(data.assets, generation),
            store::current_summary(data.funds, generation),
            store::current_summary(data.dividends_current, generation),
            store::current_summary(data.dividends_previous, generation),
            store::current_summary(data.recent, generation),
            store::current_summary(data.estimates, generation),
            generation,
        )
    };
    view! {
        <Show when=move || session.user.get().is_some()>
            <p
                class="text-xs font-medium text-text-muted"
                role="status"
                aria-live="polite"
            >
                {move || {
                    let (_, _, _, _, _, _, generation) = snapshot();
                    let any_failed = data.fetch_state.with(|map| {
                        store::refresh_failed(map, generation)
                    });
                    any_failed.then(|| {
                        view! {
                            <span class="mt-1 block">"一部の集計を取得できませんでした。"</span>
                        }
                    })
                }}
            </p>
            <SummaryBand data=data />
            <div class="home-card-row home-card-row-dividend">
                <DividendCard data=data />
                <CompositionCard data=data />
            </div>
            <div class="home-card-row">
                <RecentDividendsCard data=data />
                <PnlRankingCard data=data />
            </div>
        </Show>
    }
}

#[component]
fn SummaryBand(data: HomeOverviewData) -> impl IntoView {
    let session = use_session();
    let state = move || {
        let generation = session.generation.get();
        let assets = store::current_summary(data.assets, generation);
        let funds = store::current_summary(data.funds, generation);
        let estimates = store::current_summary(data.estimates, generation);
        (assets, funds, estimates)
    };
    view! {
        <section
            class="asset-summary asset-summary-3 home-summary"
            role="region"
            aria-label="資産の概要"
            aria-busy=move || {
                let (assets, funds, estimates) = state();
                if assets.is_none() || funds.is_none() || estimates.is_none() {
                    "true"
                } else {
                    "false"
                }
            }
            data-testid="home-overview"
        >
            {move || {
                let (assets, funds, estimates) = state();
                let loading = assets.is_none() || funds.is_none();
                let merged = if loading {
                    None
                } else {
                    let stock_summary = assets.as_ref().and_then(|slot| {
                        slot.as_ref().map(|loaded| {
                            loaded.summary.clone().unwrap_or_else(|| {
                                model::stock_summary_from_rows(&loaded.rows)
                            })
                        })
                    });
                    let fund_summary = funds.as_ref().and_then(|slot| {
                        slot.as_ref().map(|loaded| {
                            loaded.summary.clone().unwrap_or_else(|| {
                                model::fund_summary_from_rows(&loaded.rows)
                            })
                        })
                    });
                    model::merged_asset_summary(stock_summary.as_ref(), fund_summary.as_ref())
                };
                view! {
                    <BandTiles
                        merged=merged
                        estimates=estimates
                        stocks=stocks_of(&assets).unwrap_or(&[]).to_vec()
                        loading=loading
                    />
                }
            }}
        </section>
    }
}

#[component]
fn BandTiles(
    merged: Option<AssetBalanceSummary>,
    estimates: Option<Option<DividendMaps>>,
    stocks: Vec<AssetBalance>,
    loading: bool,
) -> impl IntoView {
    let market_text = if loading {
        "—".to_string()
    } else {
        format_valuation_amount(
            merged
                .as_ref()
                .map(|summary| dec_to_f64(&summary.total_market_value)),
        )
    };
    let valuation = merged.as_ref().map(summary_valuation);
    let pnl_amount = valuation
        .as_ref()
        .and_then(|valuation| valuation.profit_loss.amount);
    let pnl_tone = if is_negative_valuation(pnl_amount) {
        "asset-summary-loss"
    } else if is_positive_valuation(pnl_amount) {
        "asset-summary-gain"
    } else {
        ""
    };
    let profit_text = if loading {
        "—".to_string()
    } else {
        valuation
            .as_ref()
            .map(|valuation| format_signed_valuation_amount(valuation.profit_loss.amount))
            .unwrap_or_else(|| "—".to_string())
    };
    let profit_rate = valuation
        .as_ref()
        .and_then(|valuation| valuation.profit_loss.rate)
        .map(|rate| format_signed_valuation_rate(Some(rate), 1));
    let estimates_pending = estimates.is_none();
    let maps = estimates.as_ref().and_then(|slot| slot.as_ref());
    let kpi = maps.map(|maps| model::annual_dividend_forecast(&stocks, maps));
    let dividend_text = if estimates_pending || maps.is_none() {
        "—".to_string()
    } else {
        kpi.as_ref()
            .and_then(|kpi| kpi.total_annual_dividends)
            .map_or_else(
                || "—".to_string(),
                |amount| format_valuation_amount(Some(amount)),
            )
    };
    let yield_note = match (
        estimates_pending,
        maps,
        kpi.as_ref().and_then(|k| k.dividend_yield),
    ) {
        (true, _, _) => None,
        (false, None, _) => None,
        (false, Some(_), Some(y)) => Some(format!("取得額に対して {}", format_fixed_percent(y, 1))),
        (false, Some(maps), None) if model::has_pending_estimate(maps) => {
            Some("予想を取得中です".to_string())
        }
        _ => None,
    };
    view! {
        <dl class="asset-summary-grid">
            <div class="asset-summary-item asset-summary-item-primary">
                <dt>"評価額"</dt>
                <dd>
                    <Amount text=market_text class="asset-summary-primary-amount".to_string() />
                </dd>
            </div>
            <div class="asset-summary-item asset-summary-item-pnl">
                <dt class="sr-only lg:not-sr-only">"評価損益"</dt>
                <dd>
                    <Amount
                        text=profit_text
                        class=format!("asset-summary-pnl-amount {pnl_tone}")
                    />
                    {profit_rate.map(|rate| {
                        view! {
                            <span class=format!("asset-summary-pnl-note {pnl_tone}")>
                                {rate}
                            </span>
                        }
                    })}
                </dd>
            </div>
            <div class="asset-summary-item asset-summary-item-sub1">
                <dt>"年間予想配当(税引前)"</dt>
                <dd>
                    <Amount text=dividend_text class="asset-summary-sub-amount".to_string() />
                    {yield_note.map(|note| {
                        view! {
                            <span class="asset-summary-pnl-note text-brand-muted">
                                {note}
                            </span>
                        }
                    })}
                </dd>
            </div>
        </dl>
    }
}

#[component]
fn CardHead(title: &'static str, href: &'static str, link_label: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 px-4 pt-3.5">
            <h2 class="text-sm font-bold text-text">{title}</h2>
            <a class="home-card-link" href=href>
                {link_label}
            </a>
        </div>
    }
}

#[component]
fn CardSkeleton() -> impl IntoView {
    view! {
        <div class="flex items-center justify-center px-4 py-8 text-text-muted">
            <Spinner size=SpinnerSize::Lg class=""/>
        </div>
    }
}

#[component]
fn CardError(#[prop(into)] message: String) -> impl IntoView {
    view! {
        <div class="px-4 py-3">
            <Alert variant=AlertVariant::Warning>{message}</Alert>
        </div>
    }
}

fn short_date(date: &str) -> String {
    match model::jst_date(date) {
        Some((_, m, d)) => format!("{m}/{d}"),
        None => "—".to_string(),
    }
}

#[component]
fn DividendCard(data: HomeOverviewData) -> impl IntoView {
    let session = use_session();
    let card = move || {
        let generation = session.generation.get();
        let current = store::current_summary(data.dividends_current, generation);
        let previous = store::current_summary(data.dividends_previous, generation);
        let assets = store::current_summary(data.assets, generation);
        let estimates = store::current_summary(data.estimates, generation);
        (current, previous, assets, estimates)
    };
    view! {
        <Card variant=CardVariant::Panel class="pb-4" testid="home-dividend-card">
            <CardHead
                title="今年の配当金(税引後)"
                href="/receipts"
                link_label="取引明細で見る →"
            />
            {move || {
                let (current, previous, assets, estimates) = card();
                if current.is_none() || previous.is_none() {
                    return view! { <CardSkeleton /> }.into_any();
                }
                if matches!(current, Some(None)) {
                    return view! { <CardError message="配当の集計を取得できませんでした" /> }
                        .into_any();
                }
                let current_rows = current.flatten().unwrap_or_default();
                // 前年側の失敗はグラフの前年系列を 0 に倒して今年分は見せる。
                // 前年比較の数値は model 側で None になり「—」表示になる
                let previous_rows = previous.flatten();
                let maps = estimates.as_ref().and_then(|slot| slot.as_ref());
                let stocks = stocks_of(&assets).unwrap_or(&[]);
                let forecast = maps.and_then(|maps| {
                    model::annual_dividend_forecast(stocks, maps).total_annual_dividends
                });
                let forecast_pending = maps.is_some_and(model::has_pending_estimate);
                let now = js_sys::Date::new_0();
                let current_month = now.get_month() + 1;
                let year = now.get_full_year();
                let model = model::build_dividend_card(
                    &current_rows,
                    previous_rows.as_deref(),
                    forecast,
                    current_month,
                );
                view! {
                    <DividendCardBody
                        model=model
                        year=year
                        previous_year=year - 1
                        pending=forecast_pending
                    />
                }
                    .into_any()
            }}
        </Card>
    }
}

#[component]
fn DividendCardBody(
    model: model::DividendCardModel,
    year: u32,
    previous_year: u32,
    pending: bool,
) -> impl IntoView {
    let max = model
        .monthly
        .iter()
        .flat_map(|pair| [pair.current, pair.previous])
        .max()
        .unwrap_or_default();
    let received = format_valuation_amount(Some(dec_to_f64(&model.received)));
    let forecast = model
        .forecast_net
        .map(|v| format_valuation_amount(Some(v)))
        .unwrap_or_else(|| "—".to_string());
    let progress_pct = model.progress.map(|p| p * 100.0);
    let progress_text =
        progress_pct.map_or_else(|| "—".to_string(), |p| format!("{}%", p.round() as i64));
    let remaining_text = model::remaining_label(model.remaining);
    let current_month = js_sys::Date::new_0().get_month() + 1;
    let previous_span = match model.previous_remaining {
        Some(remaining) if current_month < 12 => format!(
            "・ 前年の{}〜12月は {} 入金 ",
            current_month + 1,
            format_valuation_amount(Some(dec_to_f64(&remaining)))
        ),
        _ => String::new(),
    };
    let yoy_text = model
        .yoy_diff
        .map(|diff| format_signed_valuation_amount(Some(dec_to_f64(&diff))))
        .unwrap_or_else(|| "—".to_string());
    view! {
        <div class="flex flex-wrap items-baseline gap-x-2 gap-y-1 px-4 pt-3">
            <span class="text-sm">
                "受取 "
                <strong class="text-lg tabular-nums">{received}</strong>
            </span>
            <span class="text-sm text-text-muted">
                {if pending {
                    "/ 予想を取得中".to_string()
                } else {
                    format!("/ 予想 {forecast}")
                }}
            </span>
            <span class="ml-auto text-sm font-bold tabular-nums">{progress_text}</span>
        </div>
        <div class="mx-4 mt-1.5 h-2.5 overflow-hidden rounded-full bg-row-group">
            {progress_pct.map(|pct| {
                view! {
                    <div
                        class="h-full rounded-full"
                        style=format!("width: {}%; background: {}", pct.clamp(0.0, 100.0), CURRENT_BAR_COLOR)
                    />
                }
            })}
        </div>
        <p class="px-4 pt-1.5 text-xs text-text-muted">
            {format!("{remaining_text} {previous_span}・ 前年同期より {yoy_text}")}
        </p>
        <div
            class="mt-3 flex items-end justify-between gap-1 px-4"
            role="group"
            aria-label=format!("月別の税引後配当({year}年と{previous_year}年)")
        >
            {model
                .monthly
                .iter()
                .map(|pair| {
                    let cur = model::bar_height(pair.current, max);
                    let prev = model::bar_height(pair.previous, max);
                    view! {
                        <div
                            class="flex min-w-0 flex-1 flex-col items-center gap-1"
                            aria-label=format!(
                                "{}月: {}年 {} / {}年 {}",
                                pair.month,
                                year,
                                format_currency(dec_to_f64(&pair.current)),
                                previous_year,
                                format_currency(dec_to_f64(&pair.previous))
                            )
                        >
                            <div class="flex h-28 w-full items-end justify-center gap-0.5">
                                <span
                                    class="home-month-bar"
                                    style=format!("height: {prev}%; background: {PREVIOUS_BAR_COLOR}")
                                />
                                <span
                                    class="home-month-bar"
                                    style=format!("height: {cur}%; background: {CURRENT_BAR_COLOR}")
                                />
                            </div>
                            <span class="text-[11px] text-text-muted">{format!("{}月", pair.month)}</span>
                        </div>
                    }
                })
                .collect_view()}
        </div>
        <div class="mt-2 flex justify-center gap-4 px-4 text-xs text-text-muted">
            <span class="flex items-center gap-1">
                <span class="home-swatch" style=format!("background: {CURRENT_BAR_COLOR}")></span>
                {format!("{year}年")}
            </span>
            <span class="flex items-center gap-1">
                <span class="home-swatch" style=format!("background: {PREVIOUS_BAR_COLOR}")></span>
                {format!("{previous_year}年")}
            </span>
        </div>
    }
}

fn slice_style(slices: &[CompositionSlice], total: f64) -> (String, Vec<(String, f64)>) {
    let mut cursor = 0.0;
    let mut stops = Vec::new();
    let mut legend = Vec::new();
    for (index, slice) in slices.iter().enumerate() {
        let color = CHART_COLORS[index % CHART_COLORS.len()];
        let start = cursor;
        let end = cursor + slice.amount / total * 100.0;
        stops.push(format!("{color} {start:.2}% {end:.2}%"));
        legend.push((color.to_string(), slice.amount / total * 100.0));
        cursor = end;
    }
    (format!("conic-gradient({})", stops.join(",")), legend)
}

#[component]
fn CompositionCard(data: HomeOverviewData) -> impl IntoView {
    let session = use_session();
    let card = move || {
        let generation = session.generation.get();
        let assets = store::current_summary(data.assets, generation);
        let funds = store::current_summary(data.funds, generation);
        (assets, funds)
    };
    view! {
        <Card variant=CardVariant::Panel class="pb-4" testid="home-composition-card">
            <CardHead title="資産の内訳" href="/assetbalance" link_label="資産管理 →" />
            {move || {
                let (assets, funds) = card();
                if assets.is_none() && funds.is_none() {
                    return view! { <CardSkeleton /> }.into_any();
                }
                if matches!(assets, Some(None)) && matches!(funds, Some(None)) {
                    return view! { <CardError message="資産の集計を取得できませんでした" /> }
                        .into_any();
                }
                let empty_holdings = stocks_of(&assets).is_none_or(<[_]>::is_empty)
                    && funds_of(&funds).is_none_or(<[_]>::is_empty);
                if matches!((&assets, &funds), (Some(_), Some(_))) && empty_holdings {
                    return view! {
                        <div class="px-4 py-4">
                            <EmptyState
                                title="保有データがありません"
                                description="保有残高をCSVで登録すると、商品別・口座別の内訳が表示されます。"
                            >
                                <a class="home-card-link mt-2" href="/assetbalance/csv">
                                    "CSV管理へ →"
                                </a>
                            </EmptyState>
                        </div>
                    }
                        .into_any();
                }
                let stock_market = stock_market(&assets);
                let fund_market = fund_market(&funds);
                let products = model::product_slices(stock_market, fund_market);
                let accounts =
                    funds_of(&funds).map_or_else(Vec::new, model::account_slices);
                let product_total: f64 = products.iter().map(|s| s.amount).sum();
                view! {
                    {if product_total <= 0.0 {
                        view! {
                            <p class="px-4 py-6 text-center text-sm text-text-muted">
                                "評価額がありません"
                            </p>
                        }
                            .into_any()
                    } else {
                        let (donut, legend) = slice_style(&products, product_total);
                        view! {
                            <div class="flex items-center gap-4 px-4 pt-3.5">
                                <div
                                    class="home-donut"
                                    style=format!("background: {donut}")
                                    role="img"
                                    aria-label="商品別の資産内訳"
                                >
                                    <div class="home-donut-hole">
                                        <span class="text-xs text-text-muted">"商品別"</span>
                                    </div>
                                </div>
                                <ul class="grid gap-2 text-sm">
                                    {products
                                        .iter()
                                        .zip(legend.iter())
                                        .map(|(slice, (color, pct))| {
                                            view! {
                                                <li class="flex items-center gap-1.5">
                                                    <span class="home-swatch" style=format!("background: {color}")></span>
                                                    <span class="truncate">{slice.label.clone()}</span>
                                                    <span class="tabular-nums text-text-muted">
                                                        {format!("{}%", pct.round() as i64)}
                                                    </span>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            </div>
                        }
                            .into_any()
                    }}
                    {if !accounts.is_empty() {
                        let account_total: f64 = accounts.iter().map(|s| s.amount).sum();
                        view! {
                            <div class="px-4 pt-4">
                                <p class="text-xs font-medium text-text-muted">
                                    "口座別(投資信託)"
                                </p>
                                <div class="home-stacked-bar mt-1.5">
                                    {accounts
                                        .iter()
                                        .enumerate()
                                        .map(|(index, slice)| {
                                            let width = slice.amount / account_total * 100.0;
                                            let color = CHART_COLORS[index % CHART_COLORS.len()];
                                            view! {
                                                <span
                                                    class="block h-full"
                                                    style=format!("width: {width:.2}%; background: {color}")
                                                />
                                            }
                                        })
                                        .collect_view()}
                                </div>
                                <ul class="mt-1.5 flex flex-wrap gap-x-3 gap-y-0.5 text-xs">
                                    {accounts
                                        .iter()
                                        .enumerate()
                                        .map(|(index, slice)| {
                                            let color = CHART_COLORS[index % CHART_COLORS.len()];
                                            let pct = slice.amount / account_total * 100.0;
                                            view! {
                                                <li class="flex items-center gap-1">
                                                    <span class="home-swatch" style=format!("background: {color}")></span>
                                                    <span>{slice.label.clone()}</span>
                                                    <span class="tabular-nums text-text-muted">
                                                        {format!("{}%", pct.round() as i64)}
                                                    </span>
                                                </li>
                                            }
                                        })
                                        .collect_view()}
                                </ul>
                            </div>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }}
                }
                    .into_any()
            }}
        </Card>
    }
}

#[component]
fn RecentDividendsCard(data: HomeOverviewData) -> impl IntoView {
    let session = use_session();
    let card = move || {
        let generation = session.generation.get();
        store::current_summary(data.recent, generation)
    };
    view! {
        <Card variant=CardVariant::Panel class="overflow-clip" testid="home-recent-card">
            <CardHead title="最近の配当入金" href="/receipts" link_label="すべて見る →" />
            {move || match card() {
                None => view! { <CardSkeleton /> }.into_any(),
                Some(None) => view! { <CardError message="配当入金を取得できませんでした" /> }.into_any(),
                Some(Some(rows)) if rows.is_empty() => {
                    view! {
                        <p class="px-4 py-6 text-center text-sm text-text-muted">
                            "まだ配当入金がありません"
                        </p>
                    }
                        .into_any()
                }
                Some(Some(rows)) => view! {
                    <ul class="mt-1.5 divide-y divide-border-faint" data-testid="home-recent-list">
                        {rows
                            .iter()
                            .map(|row| {
                                let name = if row.security_code.is_empty() {
                                    row.security_name.clone()
                                } else {
                                    format!("{} {}", row.security_code, row.security_name)
                                };
                                view! {
                                    <li class="flex items-center gap-3 px-4 py-2 text-sm">
                                        <span class="w-10 shrink-0 tabular-nums text-text-muted">
                                            {short_date(&row.settlement_date)}
                                        </span>
                                        <span class="min-w-0 flex-1 truncate">{name}</span>
                                        <span class="shrink-0 text-text-muted">{row.account.to_string()}</span>
                                        <span class="shrink-0 tabular-nums font-bold text-positive">
                                            {format_currency(dec_to_f64(&row.net_amount_received))}
                                        </span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                    .into_any(),
            }}
        </Card>
    }
}

#[component]
fn PnlRankingCard(data: HomeOverviewData) -> impl IntoView {
    let session = use_session();
    let card = move || {
        let generation = session.generation.get();
        let assets = store::current_summary(data.assets, generation);
        let funds = store::current_summary(data.funds, generation);
        (assets, funds)
    };
    view! {
        <Card variant=CardVariant::Panel class="overflow-clip" testid="home-pnl-card">
            <CardHead
                title="評価損益の大きい銘柄"
                href="/assetbalance"
                link_label="資産管理 →"
            />
            {move || {
                let (assets, funds) = card();
                if assets.is_none() && funds.is_none() {
                    return view! { <CardSkeleton /> }.into_any();
                }
                if matches!(assets, Some(None)) && matches!(funds, Some(None)) {
                    return view! { <CardError message="保有の集計を取得できませんでした" /> }
                        .into_any();
                }
                let stocks = stocks_of(&assets).unwrap_or(&[]);
                let fund_rows = funds_of(&funds).unwrap_or(&[]);
                if stocks.is_empty() && fund_rows.is_empty() {
                    return view! {
                        <p class="px-4 py-6 text-center text-sm text-text-muted">
                            "保有している銘柄がありません"
                        </p>
                    }
                        .into_any();
                }
                let holdings = model::holding_pnls(stocks, fund_rows);
                let (gains, losses) = model::pnl_extremes(&holdings, 3, 2);
                view! {
                    <ul class="mt-1.5 divide-y divide-border-faint" data-testid="home-pnl-list">
                        {gains
                            .iter()
                            .map(|holding| view! { <PnlRow holding=holding.clone() /> })
                            .collect_view()}
                    </ul>
                    {if !losses.is_empty() {
                        view! {
                            <>
                                <p class="border-t border-border-faint bg-surface-raised px-4 py-1 text-[11px] font-medium text-text-muted">
                                    "含み損"
                                </p>
                                <ul class="divide-y divide-border-faint">
                                    {losses
                                        .iter()
                                        .map(|holding| view! { <PnlRow holding=holding.clone() /> })
                                        .collect_view()}
                                </ul>
                            </>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }}
                    {if gains.is_empty() && losses.is_empty() {
                        view! {
                            <p class="px-4 py-6 text-center text-sm text-text-muted">
                                "評価損益を出せる銘柄がありません"
                            </p>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }}
                }
                    .into_any()
            }}
        </Card>
    }
}

#[component]
fn PnlRow(holding: model::HoldingPnl) -> impl IntoView {
    let positive = is_positive_valuation(Some(holding.profit_loss));
    let tone = if positive {
        "text-positive"
    } else {
        "text-negative"
    };
    let label = if holding.code.is_empty() {
        holding.name.clone()
    } else {
        format!("{} {}", holding.code, holding.name)
    };
    view! {
        <li>
            <a class="flex items-center gap-2 px-4 py-2 text-sm hover:bg-surface-raised" href=holding.href>
                <span class="min-w-0 flex-1 truncate">{label}</span>
                <span class=format!("shrink-0 tabular-nums font-bold {tone}")>
                    {format_signed_row_valuation_amount(Some(holding.profit_loss))}
                </span>
                <span class=format!("w-16 shrink-0 text-right tabular-nums {tone}")>
                    {holding
                        .rate
                        .map(|rate| format_signed_valuation_rate(Some(rate), 1))
                        .unwrap_or_else(|| "—".to_string())}
                </span>
            </a>
        </li>
    }
}
