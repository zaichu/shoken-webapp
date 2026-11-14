//! ホームダッシュボードの表示。集計は model、取得は store に寄せ、
//! ここではスロットの値を表示用の部品に展開する。

use super::model::{self, CompositionSlice};
use super::store::{self, HomeOverviewData};
use crate::api::dto::{AssetBalance, AssetBalanceSummary, FundBalance};
use crate::features::asset_balance::{
    LoadedAssetBalances, LoadedFundBalances, dec_to_f64, format_signed_row_valuation_amount,
    format_signed_valuation_amount, format_signed_valuation_rate, format_valuation_amount,
    is_negative_rate, is_negative_valuation, is_positive_rate, is_positive_valuation,
    summary_valuation,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::use_session;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::{AccountBadge, account_display_name};
use crate::ui::card::{Card, CardVariant};
use crate::ui::detail_dialog::{DetailCell, DetailDialog};
use crate::ui::empty_state::EmptyState;
use crate::ui::format::{
    format_currency, format_integer_percent, format_iso_date, format_latest_import_date,
    format_number_value, format_short_date,
};
use crate::ui::labels;
use crate::ui::list_row::{ListGroupHeader, ListRowButtonWithoutSecondary, ListRowLink};
use crate::ui::login_prompt::LoginPrompt;
use crate::ui::more_link::MoreLink;
use crate::ui::page_frame::PageFrame;
use crate::ui::page_skeleton::SkeletonSummary;
use crate::ui::state::{ListLoadError, ListSkeleton, ListSkeletonVariant};
use crate::ui::summary::{
    SummaryBand as UiSummaryBand, SummaryBreakdownEntry, SummarySubItem, SummaryTone, daily_rate,
};
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
        labels::DOMESTIC_STOCK => Some(0),
        labels::MUTUAL_FUND => Some(1),
        _ => None,
    }
}

/// 口座別の系列番号。口座名はバッジと同じ表示名に正規化してから対応表に当てる
fn account_chart_index(label: &str) -> Option<usize> {
    match account_display_name(label) {
        labels::ACCOUNT_SPECIFIC => Some(0),
        labels::ACCOUNT_NISA_GROWTH => Some(1),
        labels::ACCOUNT_OLD_NISA => Some(2),
        labels::ACCOUNT_NISA_TSUMITATE => Some(3),
        labels::ACCOUNT_GENERAL => Some(4),
        _ => None,
    }
}

/// 対応表にない系列の色。表外はすべて同じ中立色で、ほかの系列の増減や
/// 表示の並び(評価額順)で色が変わらない。複数出ても同色で、凡例の名前で区別する
const CHART_COLOR_OTHER: (&str, &str) = ("var(--color-text-muted)", "bg-text-muted");

fn chart_color(label: &str, fixed: impl Fn(&str) -> Option<usize>) -> (&'static str, &'static str) {
    fixed(label).map_or(CHART_COLOR_OTHER, |i| CHART_COLORS[i % CHART_COLORS.len()])
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
                            {labels::home_import_label(&label)}
                        </span>
                    }
                })
            }}
            // ログイン前の画面に、ログインが要る操作を出さない
            {move || {
                session.user.get().is_some().then(|| {
                    view! {
                        <a class="page-secondary-link-sm" href="/csv">
                            {labels::CSV_MANAGE_TITLE}
                        </a>
                    }
                })
            }}
        }
        .into_any()
    });
    view! {
        <PageFrame title=labels::HOME_TITLE full_bleed=true links=links>
            <HomeDashboard data=data />
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
            <div class="page-gutter">
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
                                <span class="mt-1 block">{labels::HOME_PARTIAL_ERROR}</span>
                            }
                        })
                    }}
                </p>
            </div>
            <div class="content-gutter lg:mt-3">
                <SkeletonSummary>
                    <SummaryBand data=data />
                </SkeletonSummary>
            </div>
            <div class="page-gutter flex flex-1 flex-col pb-4">
                <div class="home-card-row home-card-row-dividend">
                    <DividendCard data=data />
                    <CompositionCard data=data />
                </div>
                <div class="home-card-row">
                    <RecentDividendsCard data=data />
                    <PnlRankingCard data=data />
                </div>
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
            class="summary-band"
            role="region"
            aria-label=labels::HOME_OVERVIEW_LABEL
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
                let (merged, daily, as_of, pending) = if loading {
                    (None, None, String::new(), false)
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
                    let daily = model::merged_daily_change(
                        stock_summary.as_ref(),
                        stocks_of(&assets).is_some_and(|rows| !rows.is_empty()),
                        fund_summary.as_ref(),
                    );
                    let merged = model::merged_asset_summary(
                        stock_summary.as_ref(),
                        fund_summary.as_ref(),
                    );
                    let stock_rows = stocks_of(&assets).unwrap_or(&[]);
                    let fund_rows = funds_of(&funds).unwrap_or(&[]);
                    let as_of = model::oldest_home_as_of(stock_rows, fund_rows)
                        .unwrap_or_default();
                    let pending = model::home_has_pending(
                        stock_rows,
                        fund_rows,
                        stock_summary.as_ref(),
                    );
                    (merged, daily, as_of, pending)
                };
                view! {
                    <BandTiles
                        merged=merged
                        daily=daily
                        estimates=estimates
                        stocks=stocks_of(&assets).unwrap_or(&[]).to_vec()
                        loading=loading
                        as_of=as_of
                        show_pending=pending
                    />
                }
            }}
        </section>
    }
}

fn daily_rate_text(market: Option<f64>, daily: Option<f64>) -> String {
    daily_rate(market, daily).map_or_default(|rate| format_signed_valuation_rate(Some(rate), 1))
}

#[component]
fn BandTiles(
    merged: Option<AssetBalanceSummary>,
    daily: Option<f64>,
    estimates: Option<Option<DividendMaps>>,
    stocks: Vec<AssetBalance>,
    loading: bool,
    #[prop(optional, into)] as_of: String,
    #[prop(optional)] show_pending: bool,
) -> impl IntoView {
    let market_value = merged
        .as_ref()
        .map(|summary| dec_to_f64(&summary.total_market_value));
    let market_text = if loading {
        labels::MISSING.to_string()
    } else {
        format_valuation_amount(market_value)
    };
    let valuation = merged.as_ref().map(summary_valuation);
    let pnl_amount = valuation
        .as_ref()
        .and_then(|valuation| valuation.profit_loss.amount);
    let pnl_negative = is_negative_valuation(pnl_amount);
    let pnl_positive = is_positive_valuation(pnl_amount);
    let profit_text = if loading {
        labels::MISSING.to_string()
    } else {
        valuation.as_ref().map_or_else(
            || labels::MISSING.to_string(),
            |valuation| format_signed_valuation_amount(valuation.profit_loss.amount),
        )
    };
    let profit_rate = valuation
        .as_ref()
        .and_then(|valuation| valuation.profit_loss.rate)
        .map_or_default(|rate| format_signed_valuation_rate(Some(rate), 1));
    let daily_value = daily;
    let daily_text = if loading {
        labels::MISSING.to_string()
    } else {
        format_signed_row_valuation_amount(daily_value)
    };
    let daily_rate = if loading {
        String::new()
    } else {
        daily_rate_text(market_value, daily_value)
    };
    let daily_negative = is_negative_valuation(daily_value);
    let daily_positive = is_positive_valuation(daily_value);
    let purchase_value = merged
        .as_ref()
        .map(|summary| dec_to_f64(&summary.total_purchase_amount));
    let purchase_text = if loading {
        labels::MISSING.to_string()
    } else {
        format_valuation_amount(purchase_value)
    };
    let estimates_pending = estimates.is_none();
    let maps = estimates.as_ref().and_then(|slot| slot.as_ref());
    let kpi = maps.map(|maps| model::annual_dividend_forecast(&stocks, maps));
    let dividend_text = if estimates_pending || maps.is_none() {
        labels::MISSING.to_string()
    } else {
        kpi.as_ref()
            .and_then(|kpi| kpi.total_annual_dividends)
            .map_or_else(
                || labels::MISSING.to_string(),
                |amount| format_valuation_amount(Some(amount)),
            )
    };
    let estimate_pending_text = match (maps, kpi.as_ref().and_then(|k| k.dividend_yield)) {
        (Some(maps), None) if model::has_pending_estimate(maps) => {
            labels::HOME_ESTIMATE_PENDING.to_string()
        }
        _ => String::new(),
    };
    view! {
        <UiSummaryBand
            primary_label=labels::MARKET_VALUE
            primary_value=market_text
            primary_as_of=as_of
            show_pending=show_pending
            aux_label=labels::VALUATION_PROFIT_LOSS
            aux_value=profit_text
            aux_rate=profit_rate
            aux_negative=pnl_negative
            aux_positive=pnl_positive
            pc_items=vec![
                SummarySubItem {
                    label: labels::ANNUAL_EXPECTED_DIVIDEND.to_string(),
                    value: dividend_text.clone(),
                    note: String::new(),
                    tone: SummaryTone::Neutral,
                    extra_class: String::new(),
                },
            ]
            breakdown=vec![
                SummaryBreakdownEntry {
                    label: labels::DAY_CHANGE.to_string(),
                    value: daily_text,
                    negative: daily_negative,
                    positive: daily_positive,
                    rate: daily_rate,
                },
                SummaryBreakdownEntry {
                    label: labels::ANNUAL_EXPECTED_DIVIDEND.to_string(),
                    value: dividend_text,
                    negative: false,
                    positive: false,
                    rate: estimate_pending_text,
                },
                SummaryBreakdownEntry {
                    label: labels::TOTAL_ACQUISITION.to_string(),
                    value: purchase_text,
                    negative: false,
                    positive: false,
                    rate: String::new(),
                },
            ]
            testid_toggle="summary-details-toggle"
        />
    }
}

#[component]
fn CardHead(title: &'static str, href: &'static str, link_label: &'static str) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 px-4 pt-3.5">
            <h2 class="text-sm font-bold text-text">{title}</h2>
            <MoreLink href=href.to_string() label=link_label.to_string() />
        </div>
    }
}

#[component]
fn CardSkeleton() -> impl IntoView {
    view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
}

#[component]
fn CardError(#[prop(into)] message: String) -> impl IntoView {
    let session = use_session();
    view! {
        <div class="px-4 py-3">
            <ListLoadError
                message=message
                on_retry=move || session.generation.update(|generation| *generation = generation.next())
            />
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
                title=labels::HOME_THIS_YEAR_DIVIDEND
                href="/receipts"
                link_label=labels::HOME_RECEIPTS_VIEW_LINK
            />
            {move || {
                let (current, previous, assets, estimates) = card();
                if current.is_none() || previous.is_none() {
                    return view! { <CardSkeleton /> }.into_any();
                }
                if matches!(current, Some(None)) {
                    return view! { <CardError message=labels::HOME_DIVIDEND_FETCH_FAILED.to_string() /> }
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
    let received_value = Some(dec_to_f64(&model.received));
    let received = format_signed_valuation_amount(received_value);
    let forecast = model.forecast_net.map_or_else(
        || labels::MISSING.to_string(),
        |v| format_valuation_amount(Some(v)),
    );
    let progress_pct = model.progress.map(|p| p * 100.0);
    let progress_text =
        progress_pct.map_or_else(|| labels::MISSING.to_string(), format_integer_percent);
    let remaining_text = model::remaining_label(model.remaining);
    let current_month = js_sys::Date::new_0().get_month() + 1;
    let previous_span = model
        .previous_remaining
        .map_or_else(String::new, |remaining| {
            labels::previous_dividend_span(
                current_month,
                &format_valuation_amount(Some(dec_to_f64(&remaining))),
            )
        });
    let yoy_text = model.yoy_diff.map_or_else(
        || labels::MISSING.to_string(),
        |diff| format_signed_valuation_amount(Some(dec_to_f64(&diff))),
    );
    view! {
        <div class="flex flex-wrap items-baseline gap-x-2 gap-y-1 px-4 pt-3">
            <span class="text-sm">
                {labels::HOME_RECEIVED_LABEL}
                <strong class="text-lg">
                    <Amount
                        kind=AmountKind::Delta
                        text=received
                        negative=is_negative_valuation(received_value)
                        positive=is_positive_valuation(received_value)
                    />
                </strong>
            </span>
            <span class="text-sm text-text-muted">
                {if pending {
                    labels::HOME_FORECAST_LOADING.to_string()
                } else {
                    labels::home_forecast(&forecast)
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
            {labels::home_dividend_summary(&remaining_text, &previous_span, &yoy_text)}
        </p>
        <div
            class="mt-3 px-4"
            role="group"
            aria-label=labels::home_monthly_aria(year, previous_year)
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
                                labels::HOME_CURRENT_MONTH
                            } else {
                                ""
                            };
                            view! {
                                <div
                                    class="home-chart-month"
                                    tabindex="0"
                                    role="group"
                                    aria-label=labels::home_month_aria(
                                        pair.month,
                                        current_note,
                                        year,
                                        &cur_text,
                                        previous_year,
                                        &prev_text,
                                        &diff_text,
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
                                            {labels::receipt_month_label(pair.month)}
                                        </p>
                                        <dl class="mt-1 space-y-0.5">
                                            <div class="flex items-center gap-1.5">
                                                <dt class="home-swatch bg-chart-1"></dt>
                                                <dd>{labels::home_year_amount(year, &cur_text)}</dd>
                                            </div>
                                            <div class="flex items-center gap-1.5">
                                                <dt class="home-swatch bg-chart-previous"></dt>
                                                <dd>{labels::home_year_amount(previous_year, &prev_text)}</dd>
                                            </div>
                                            <div class="flex items-center gap-1.5">
                                                <dt class="text-text-muted">{labels::HOME_DIFF_LABEL}</dt>
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
                                {labels::receipt_month_label(pair.month)}
                            </span>
                        }
                    })
                    .collect_view()}
            </div>
        </div>
        <div class="mt-2 flex justify-center gap-4 px-4 text-xs text-text-muted">
            <span class="flex items-center gap-1">
                <span class="home-swatch bg-chart-1"></span>
                {labels::receipt_year_label(&year.to_string())}
            </span>
            <span class="flex items-center gap-1">
                <span class="home-swatch bg-chart-previous"></span>
                {labels::receipt_year_label(&previous_year.to_string())}
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
            <CardHead title=labels::HOME_COMPOSITION_TITLE href="/assetbalance" link_label=labels::ASSET_BALANCE_TITLE />
            {move || {
                let (assets, funds) = card();
                if assets.is_none() && funds.is_none() {
                    return view! { <CardSkeleton /> }.into_any();
                }
                if matches!(assets, Some(None)) && matches!(funds, Some(None)) {
                    return view! { <CardError message=labels::HOME_ASSET_FETCH_FAILED.to_string() /> }
                        .into_any();
                }
                let empty_holdings = stocks_of(&assets).is_none_or(<[_]>::is_empty)
                    && funds_of(&funds).is_none_or(<[_]>::is_empty);
                if matches!((&assets, &funds), (Some(_), Some(_))) && empty_holdings {
                    return view! {
                        <div class="px-4 py-4">
                            <EmptyState
                                title=labels::HOME_NO_HOLDINGS
                                description=labels::HOME_NO_HOLDINGS_HINT
                            >
                                <MoreLink
                                    href="/csv".to_string()
                                    label=labels::SETTINGS_CSV_MANAGE_LINK.to_string()
                                />
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
                            <div class="px-4 py-4">
                                <EmptyState title=labels::HOME_NO_MARKET_VALUE />
                            </div>
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
                                    aria-label=labels::HOME_PRODUCT_BREAKDOWN_LABEL
                                >
                                    <div class="home-donut-hole"></div>
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
                                    {labels::HOME_ACCOUNT_FUND_LABEL}
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

fn home_dividend_dialog_cells(
    full_date: &str,
    shares: String,
    per_share: String,
    gross: (String, bool, bool),
    tax: (String, bool, bool),
    net: (String, bool, bool),
) -> (Vec<DetailCell>, Vec<DetailCell>) {
    let left = vec![
        DetailCell::plain(labels::PAYMENT_DATE.to_string(), format_iso_date(full_date)),
        DetailCell::plain(labels::QUANTITY.to_string(), shares),
        DetailCell::plain(labels::PER_SHARE_DIVIDEND.to_string(), per_share),
    ];
    let right = vec![
        DetailCell {
            label: labels::BEFORE_TAX.to_string(),
            value: gross.0,
            sub: None,
            negative: gross.1,
            positive: gross.2,
            bold: false,
            delta: true,
        },
        DetailCell {
            label: labels::TAX.to_string(),
            value: tax.0,
            sub: None,
            negative: tax.1,
            positive: tax.2,
            bold: false,
            delta: tax.1 || tax.2,
        },
        DetailCell::strong(labels::AFTER_TAX.to_string(), net.0, net.1, net.2, true),
    ];
    (left, right)
}

fn signed_gross_text(value: f64, text: &str) -> String {
    if value > 0.0 && !text.trim_start().starts_with('+') {
        format!("+{text}")
    } else {
        text.to_string()
    }
}

fn signed_tax_text(value: f64, text: &str) -> String {
    if value > 0.0 {
        format!("-{text}")
    } else if value < 0.0 {
        format!("+{}", text.trim_start_matches('-'))
    } else {
        text.to_string()
    }
}

#[component]
fn HomeDividendRow(row: crate::api::dto::Dividend) -> impl IntoView {
    let code = if row.security_code.is_empty() {
        labels::MISSING.to_string()
    } else {
        row.security_code.clone()
    };
    let date = format_short_date(&row.settlement_date);
    let full_date = row.settlement_date.clone();
    let name = row.security_name.clone();
    let sheet_title = name.clone();
    let row_name = name.clone();
    let account = String::from(row.account.clone());
    let account_for_meta = account.clone();
    let net_value = dec_to_f64(&row.net_amount_received);
    let net = format_currency(net_value);
    let net_signed = if net_value > 0.0 {
        format!("+{net}")
    } else {
        net.clone()
    };
    let net_negative = net_value < 0.0;
    let net_positive = net_value > 0.0;
    let net_for_row = net_signed.clone();
    let gross_value = dec_to_f64(&row.dividends_before_tax);
    let gross_raw = format_currency(gross_value);
    let gross_signed = signed_gross_text(gross_value, &gross_raw);
    let gross_negative = gross_value < 0.0;
    let gross_positive = gross_value > 0.0;
    let tax_value = dec_to_f64(&row.taxes);
    let tax_raw = format_currency(tax_value);
    let tax_signed = signed_tax_text(tax_value, &tax_raw);
    let tax_negative = tax_value > 0.0;
    let tax_positive = tax_value < 0.0;
    let per_share = format_currency(dec_to_f64(&row.unit_price));
    let shares = format_number_value(dec_to_f64(&row.shares));
    let code_for_meta = code.clone();
    let date_for_meta = date.clone();
    let sheet_open = RwSignal::new(false);
    let dialog_open = sheet_open;
    let dialog_name = sheet_title.clone();
    let dialog_code = if code == labels::MISSING {
        String::new()
    } else {
        code.clone()
    };
    let dialog_account = account.clone();
    let (dialog_left, dialog_right) = home_dividend_dialog_cells(
        &full_date,
        shares.clone(),
        per_share.clone(),
        (gross_signed, gross_negative, gross_positive),
        (tax_signed, tax_negative, tax_positive),
        (net_signed.clone(), net_negative, net_positive),
    );
    view! {
        <li class="list-row-item" data-testid="home-dividend-row">
            <ListRowButtonWithoutSecondary
                testid="home-dividend-button"
                on_click=move |_| sheet_open.set(true)
                name=move || row_name.clone()
                primary=move || view! {
                    <Amount
                        kind=AmountKind::Delta
                        text=net_for_row.clone()
                        negative=net_negative
                        positive=net_positive
                    />
                }
                meta=move || view! {
                    {date_for_meta.clone()}
                    {" · "}
                    {code_for_meta.clone()}
                    {" · "}
                    <AccountBadge label=account_for_meta.clone() />
                }
            />
            <Show when=move || dialog_open.get()>
                <DetailDialog
                    name=dialog_name.clone()
                    code=dialog_code.clone()
                    account=dialog_account.clone()
                    left=dialog_left.clone()
                    right=dialog_right.clone()
                    on_close=move || dialog_open.set(false)
                    testid="home-dividend-dialog"
                />
            </Show>
        </li>
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
            <CardHead title=labels::HOME_RECENT_DIVIDENDS href="/receipts" link_label=labels::HOME_SEE_ALL />
            {move || match card() {
                None => view! { <CardSkeleton /> }.into_any(),
                Some(None) => view! { <CardError message=labels::HOME_RECENT_FETCH_FAILED.to_string() /> }.into_any(),
                Some(Some(rows)) if rows.is_empty() => {
                    view! {
                        <div class="px-4 py-4">
                            <EmptyState title=labels::HOME_NO_RECENT />
                        </div>
                    }
                        .into_any()
                }
                Some(Some(rows)) => view! {
                    <ul class="asset-compact-list" data-testid="home-recent-list">
                        {rows
                            .iter()
                            .cloned()
                            .map(|row| view! { <HomeDividendRow row=row /> })
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
                title=labels::HOME_PNL_TITLE
                href="/assetbalance"
                link_label=labels::ASSET_BALANCE_TITLE
            />
            {move || {
                let (assets, funds) = card();
                if assets.is_none() && funds.is_none() {
                    return view! { <CardSkeleton /> }.into_any();
                }
                if matches!(assets, Some(None)) && matches!(funds, Some(None)) {
                    return view! { <CardError message=labels::HOME_HOLDING_FETCH_FAILED.to_string() /> }
                        .into_any();
                }
                let stocks = stocks_of(&assets).unwrap_or(&[]);
                let fund_rows = funds_of(&funds).unwrap_or(&[]);
                if stocks.is_empty() && fund_rows.is_empty() {
                    return view! {
                        <div class="px-4 py-4">
                            <EmptyState title=labels::HOME_NO_STOCKS />
                        </div>
                    }
                        .into_any();
                }
                let holdings = model::holding_pnls(stocks, fund_rows);
                let (gains, losses) = model::pnl_extremes(&holdings, 3, 2);
                view! {
                    <ul
                        class="asset-compact-list"
                        data-testid="home-pnl-list"
                    >
                        {gains
                            .iter()
                            .map(|holding| view! { <PnlRow holding=holding.clone() /> })
                            .collect_view()}
                        {(!losses.is_empty())
                            .then(|| {
                                view! {
                                    <li aria-hidden="true">
                                        <ListGroupHeader label=labels::HOME_LOSS_GROUP.to_string() summary=None />
                                    </li>
                                }
                            })}
                        {losses
                            .iter()
                            .map(|holding| view! { <PnlRow holding=holding.clone() /> })
                            .collect_view()}
                    </ul>
                    {if gains.is_empty() && losses.is_empty() {
                        view! {
                            <div class="px-4 py-4">
                                <EmptyState title=labels::HOME_NO_PNL />
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
fn PnlRow(holding: model::HoldingPnl) -> impl IntoView {
    let negative = is_negative_valuation(Some(holding.profit_loss));
    let positive = is_positive_valuation(Some(holding.profit_loss));
    let rate_negative = holding
        .rate
        .is_some_and(|rate| is_negative_rate(Some(rate)));
    let rate_positive = holding
        .rate
        .is_some_and(|rate| is_positive_rate(Some(rate)));
    let name = holding.name.clone();
    let code = holding.code.clone();
    let href = holding.href.clone();
    let amount_text = format_signed_row_valuation_amount(Some(holding.profit_loss));
    let rate_text = holding.rate.map_or_else(
        || labels::MISSING.to_string(),
        |rate| format_signed_valuation_rate(Some(rate), 1),
    );
    view! {
        <li class="list-row-item">
            <ListRowLink
                href=href
                name=move || name.clone()
                primary=move || view! {
                    <Amount
                        kind=AmountKind::Delta
                        text=amount_text.clone()
                        negative=negative
                        positive=positive
                    />
                }
                meta=move || view! {
                    {code.clone()}
                }
                secondary=move || view! {
                    <Amount
                        kind=AmountKind::Delta
                        text=rate_text.clone()
                        negative=rate_negative
                        positive=rate_positive
                    />
                }
            />
        </li>
    }
}

#[cfg(test)]
mod tests;
