//! ホームダッシュボードの表示。集計は model、取得は store に寄せ、
//! ここではスロットの値を表示用の部品に展開する。

use super::model::{self, CompositionSlice};
use super::store::{self, HomeOverviewData};
use crate::api::dto::{AssetBalance, AssetBalanceSummary, FundBalance};
use crate::features::asset_balance::{
    LoadedAssetBalances, LoadedFundBalances, dec_to_f64, format_signed_row_valuation_amount,
    format_signed_valuation_amount, format_signed_valuation_rate, format_valuation_amount,
    is_negative_valuation, is_positive_valuation, summary_valuation,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::use_session;
use crate::ui::badge::{AccountBadge, account_display_name};
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::format::{
    format_currency, format_integer_percent, format_latest_import_date, format_short_date,
};
use crate::ui::labels::{ANNUAL_EXPECTED_DIVIDEND, MARKET_VALUE, MISSING, VALUATION_PROFIT_LOSS};
use crate::ui::login_prompt::LoginPrompt;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, Spinner, SpinnerSize};
use crate::ui::summary::{SummaryItem, SummarySize, SummaryTone};
use leptos::prelude::*;

// 内訳チャートの塗り色。ドーナツのグラデーションは CSS 変数、棒・凡例は同じ色の
// クラスで渡す。本番 CSP(style-src 'self')は style 属性を止めるので、静的な色は
// クラス・動的な大きさやグラデーションは style:<prop> で指定する
const CHART_COLORS: &[(&str, &str)] = &[
    ("var(--color-chart-1)", "bg-chart-1"),
    ("var(--color-chart-2)", "bg-chart-2"),
    ("var(--color-chart-3)", "bg-chart-3"),
    ("var(--color-chart-4)", "bg-chart-4"),
    ("var(--color-chart-5)", "bg-chart-5"),
];

/// 商品別ドーナツの系列番号。商品名に固定で対応させ、絞り込みや並び替えで色が変わらないようにする
fn product_chart_index(label: &str) -> Option<usize> {
    match label {
        "国内株式" => Some(0),
        "投資信託" => Some(1),
        _ => None,
    }
}

/// 口座別の系列番号。口座名はバッジと同じ表示名に正規化してから対応表に当てる
fn account_chart_index(label: &str) -> Option<usize> {
    match account_display_name(label) {
        "特定" => Some(0),
        "NISA成長" => Some(1),
        "旧NISA" => Some(2),
        "NISAつみたて" => Some(3),
        "一般" => Some(4),
        _ => None,
    }
}

/// 対応表にない系列の色。表外はすべて同じ中立色で、ほかの系列の増減や
/// 表示の並び(評価額順)で色が変わらない。複数出ても同色で、凡例の名前で区別する
const CHART_COLOR_OTHER: (&str, &str) = ("var(--color-text-muted)", "bg-text-muted");

fn chart_color(label: &str, fixed: impl Fn(&str) -> Option<usize>) -> (&'static str, &'static str) {
    match fixed(label) {
        Some(i) => CHART_COLORS[i % CHART_COLORS.len()],
        None => CHART_COLOR_OTHER,
    }
}

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
        format_latest_import_date(stamps.iter().map(String::as_str))
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
            // ログイン前の画面に、ログインが要る操作を出さない
            {move || {
                session.user.get().is_some().then(|| {
                    view! {
                        <a class="page-secondary-link-sm" href="/csv">
                            "CSV管理"
                        </a>
                    }
                })
            }}
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
        <Show
            when=move || session.user.get().is_some()
            fallback=|| view! { <LoginPrompt class="mx-auto w-full max-w-md" /> }
        >
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
            class="summary-band home-summary"
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
        MISSING.to_string()
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
        SummaryTone::Loss
    } else if is_positive_valuation(pnl_amount) {
        SummaryTone::Gain
    } else {
        SummaryTone::Neutral
    };
    let profit_text = if loading {
        MISSING.to_string()
    } else {
        valuation
            .as_ref()
            .map(|valuation| format_signed_valuation_amount(valuation.profit_loss.amount))
            .unwrap_or_else(|| MISSING.to_string())
    };
    let profit_rate = valuation
        .as_ref()
        .and_then(|valuation| valuation.profit_loss.rate)
        .map(|rate| format_signed_valuation_rate(Some(rate), 1));
    let estimates_pending = estimates.is_none();
    let maps = estimates.as_ref().and_then(|slot| slot.as_ref());
    let kpi = maps.map(|maps| model::annual_dividend_forecast(&stocks, maps));
    let dividend_text = if estimates_pending || maps.is_none() {
        MISSING.to_string()
    } else {
        kpi.as_ref()
            .and_then(|kpi| kpi.total_annual_dividends)
            .map_or_else(
                || MISSING.to_string(),
                |amount| format_valuation_amount(Some(amount)),
            )
    };
    let estimate_note = match (maps, kpi.as_ref().and_then(|k| k.dividend_yield)) {
        (Some(maps), None) if model::has_pending_estimate(maps) => {
            Some("予想を取得中です".to_string())
        }
        _ => None,
    };
    view! {
        <div class="summary-band-grid">
            <dl class="summary-band-group summary-band-group-main">
                <SummaryItem
                    label=MARKET_VALUE
                    value=market_text
                    size=SummarySize::Primary
                />
                <SummaryItem
                    label=VALUATION_PROFIT_LOSS
                    value=profit_text
                    size=SummarySize::Secondary
                    note=profit_rate.unwrap_or_default()
                    tone=pnl_tone
                    extra_class="summary-band-item-pnl"
                />
            </dl>
            <div class="summary-band-group summary-band-group-sub">
                <dl class="summary-band-items">
                    <SummaryItem
                        label=ANNUAL_EXPECTED_DIVIDEND
                        value=dividend_text
                        size=SummarySize::Secondary
                        status=estimate_note.unwrap_or_default()
                    />
                </dl>
            </div>
        </div>
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
        .unwrap_or_else(|| MISSING.to_string());
    let progress_pct = model.progress.map(|p| p * 100.0);
    let progress_text = progress_pct.map_or_else(|| MISSING.to_string(), format_integer_percent);
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
        .unwrap_or_else(|| MISSING.to_string());
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
        <div class="home-progress">
            {progress_pct.map(|pct| {
                view! {
                    <div
                        class="home-progress-fill"
                        style:width=format!("{}%", pct.clamp(0.0, 100.0))
                    />
                }
            })}
        </div>
        <p class="px-4 pt-1.5 text-xs text-text-muted">
            {format!("{remaining_text} {previous_span}・ 前年同期より {yoy_text}")}
        </p>
        <div
            class="mt-3 px-4"
            role="group"
            aria-label=format!("月別の税引後配当({year}年と{previous_year}年)")
        >
            <p class="home-chart-max">{format_valuation_amount(Some(dec_to_f64(&max)))}</p>
            <div class="home-chart-plot">
                <div class="home-chart-grid" aria-hidden="true">
                    <span class="home-chart-grid-line" data-level="top"></span>
                    <span class="home-chart-grid-line" data-level="mid"></span>
                    <span class="home-chart-grid-line" data-level="base"></span>
                </div>
                <div class="home-chart-months">
                    {model
                        .monthly
                        .iter()
                        .map(|pair| {
                            let cur = model::bar_height(pair.current, max);
                            let prev = model::bar_height(pair.previous, max);
                            let cur_text = format_currency(dec_to_f64(&pair.current));
                            let prev_text = format_currency(dec_to_f64(&pair.previous));
                            let diff_text = format_signed_valuation_amount(Some(dec_to_f64(
                                &(pair.current - pair.previous),
                            )));
                            let current_note = if pair.month == current_month {
                                "(今月)"
                            } else {
                                ""
                            };
                            view! {
                                <div
                                    class="home-chart-month"
                                    tabindex="0"
                                    role="group"
                                    aria-label=format!(
                                        "{}月{}: {}年 {} / {}年 {} / 差 {}",
                                        pair.month,
                                        current_note,
                                        year,
                                        cur_text,
                                        previous_year,
                                        prev_text,
                                        diff_text
                                    )
                                >
                                    <div class="home-chart-bars">
                                        <span
                                            class="home-month-bar bg-chart-previous"
                                            style:height=format!("{prev}%")
                                        />
                                        <span
                                            class="home-month-bar bg-chart-1"
                                            style:height=format!("{cur}%")
                                        />
                                    </div>
                                    <div class="home-chart-tip" aria-hidden="true">
                                        <p class="font-bold text-text">
                                            {format!("{}月", pair.month)}
                                        </p>
                                        <dl class="mt-1 space-y-0.5">
                                            <div class="flex items-center gap-1.5">
                                                <dt class="home-swatch bg-chart-1"></dt>
                                                <dd>{format!("{}年 {}", year, cur_text)}</dd>
                                            </div>
                                            <div class="flex items-center gap-1.5">
                                                <dt class="home-swatch bg-chart-previous"></dt>
                                                <dd>{format!("{}年 {}", previous_year, prev_text)}</dd>
                                            </div>
                                            <div class="flex items-center gap-1.5">
                                                <dt class="text-text-muted">"差"</dt>
                                                <dd class="tabular-nums">{diff_text.clone()}</dd>
                                            </div>
                                        </dl>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>
            </div>
            <div class="home-chart-month-labels" aria-hidden="true">
                {model
                    .monthly
                    .iter()
                    .map(|pair| {
                        let current = pair.month == current_month;
                        view! {
                            <span
                                class="home-chart-month-label"
                                data-current=current.to_string()
                            >
                                {format!("{}月", pair.month)}
                            </span>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
        <div class="mt-2 flex justify-center gap-4 px-4 text-xs text-text-muted">
            <span class="flex items-center gap-1">
                <span class="home-swatch bg-chart-1"></span>
                {format!("{year}年")}
            </span>
            <span class="flex items-center gap-1">
                <span class="home-swatch bg-chart-previous"></span>
                {format!("{previous_year}年")}
            </span>
        </div>
    }
}

fn slice_style(
    slices: &[CompositionSlice],
    total: f64,
    chart_index: impl Fn(&str) -> Option<usize>,
) -> (String, Vec<(&'static str, f64)>) {
    // 系列の境目は面の色の隙間にする。96px のドーナツでは両側 0.35% で計およそ 2px
    const HALF_GAP: f64 = 0.35;
    let gapped = slices.len() > 1;
    let mut stops = Vec::new();
    let mut legend = Vec::new();
    if gapped {
        stops.push(format!("var(--color-surface) 0% {HALF_GAP:.2}%"));
    }
    let mut cursor = 0.0;
    for slice in slices {
        let (color, class) = chart_color(&slice.label, &chart_index);
        let end = cursor + slice.amount / total * 100.0;
        if gapped {
            let start = cursor + HALF_GAP;
            let color_end = end - HALF_GAP;
            let gap_end = (end + HALF_GAP).min(100.0);
            stops.push(format!(
                "{color} {start:.2}% {color_end:.2}%, var(--color-surface) {color_end:.2}% {gap_end:.2}%"
            ));
        } else {
            stops.push(format!("{color} {cursor:.2}% {end:.2}%"));
        }
        legend.push((class, slice.amount / total * 100.0));
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
                                <a class="home-card-link mt-2" href="/csv">
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
                        let (donut, legend) =
                            slice_style(&products, product_total, product_chart_index);
                        view! {
                            <div class="flex items-center gap-4 px-4 pt-3.5">
                                <div
                                    class="home-donut"
                                    style:background=donut
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
                                        .map(|(slice, (class, pct))| {
                                            view! {
                                                <li class="flex items-center gap-1.5">
                                                    <span class=format!("home-swatch {class}")></span>
                                                    <span class="truncate text-text-soft">{slice.label.clone()}</span>
                                                    <span class="tabular-nums text-text-muted">
                                                        {format_integer_percent(*pct)}
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
                                        .map(|slice| {
                                            let width = slice.amount / account_total * 100.0;
                                            let class =
                                                chart_color(&slice.label, account_chart_index).1;
                                            view! {
                                                <span
                                                    class=format!("block h-full {class}")
                                                    style:width=format!("{width:.2}%")
                                                />
                                            }
                                        })
                                        .collect_view()}
                                </div>
                                <ul class="mt-1.5 flex flex-wrap gap-x-3 gap-y-0.5 text-xs">
                                    {accounts
                                        .iter()
                                        .map(|slice| {
                                            let class =
                                                chart_color(&slice.label, account_chart_index).1;
                                            let pct = slice.amount / account_total * 100.0;
                                            view! {
                                                <li class="flex items-center gap-1">
                                                    <span class=format!("home-swatch {class}")></span>
                                                    <AccountBadge label=slice.label.clone() />
                                                    <span class="tabular-nums text-text-muted">
                                                        {format_integer_percent(pct)}
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
                    <ul class="home-dividend-list divide-y divide-border-faint" data-testid="home-recent-list">
                        {rows
                            .iter()
                            .map(|row| {
                                let code = if row.security_code.is_empty() {
                                    MISSING.to_string()
                                } else {
                                    row.security_code.clone()
                                };
                                view! {
                                    <li class="home-dividend-row">
                                        <span class="home-dividend-date">
                                            {format_short_date(&row.settlement_date)}
                                        </span>
                                        <span class="home-dividend-code">{code}</span>
                                        <span class="home-dividend-name" title=row.security_name.clone()>
                                            {row.security_name.clone()}
                                        </span>
                                        <span class="home-dividend-account">
                                            <AccountBadge label=row.account.clone() />
                                        </span>
                                        <span class="home-dividend-amount">
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
                    <ul
                        class="mt-1.5 divide-y divide-border-faint"
                        data-testid="home-pnl-list"
                    >
                        {gains
                            .iter()
                            .map(|holding| view! { <PnlRow holding=holding.clone() /> })
                            .collect_view()}
                        {(!losses.is_empty())
                            .then(|| {
                                view! { <li class="home-pnl-group">"含み損"</li> }
                            })}
                        {losses
                            .iter()
                            .map(|holding| view! { <PnlRow holding=holding.clone() /> })
                            .collect_view()}
                    </ul>
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
    let title = label.clone();
    view! {
        <li>
            <a
                class="home-pnl-row hover:bg-surface-raised max-sm:min-h-11"
                href=holding.href
            >
                <span class="home-pnl-name" title=title>{label}</span>
                <span class=format!("home-pnl-amount {tone}")>
                    {format_signed_row_valuation_amount(Some(holding.profit_loss))}
                </span>
                <span class=format!("home-pnl-rate {tone}")>
                    {holding
                        .rate
                        .map(|rate| format_signed_valuation_rate(Some(rate), 1))
                        .unwrap_or_else(|| MISSING.to_string())}
                </span>
            </a>
        </li>
    }
}

#[cfg(test)]
mod tests;
