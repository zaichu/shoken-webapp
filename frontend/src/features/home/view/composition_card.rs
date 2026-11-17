use super::super::model::{self, CompositionSlice};
use super::super::store::{self, HomeOverviewData};
use super::card_parts::{CardError, CardHead, CardSkeleton};
use super::{fund_market, funds_of, stock_market, stocks_of};
use crate::session::use_session;
use crate::ui::badge::{AccountBadge, account_display_name};
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::format::format_integer_percent;
use crate::ui::labels;
use crate::ui::more_link::MoreLink;
use leptos::prelude::*;

// 内訳チャートの塗り色。ドーナツのグラデーションは CSS 変数、棒・凡例は同じ色の
// クラスで渡す。本番 CSP(style-src 'self')は style 属性を止めるので、静的な色は
// クラス・動的な大きさやグラデーションは style:<prop> で指定する
pub(super) const CHART_COLORS: &[(&str, &str)] = &[
    ("var(--color-chart-1)", "bg-chart-1"),
    ("var(--color-chart-2)", "bg-chart-2"),
    ("var(--color-chart-3)", "bg-chart-3"),
    ("var(--color-chart-4)", "bg-chart-4"),
    ("var(--color-chart-5)", "bg-chart-5"),
    ("var(--color-chart-6)", "bg-chart-6"),
];
const NON_AQUA_CHART_COLORS: [usize; 5] = [0, 1, 2, 4, 5];

/// 商品別ドーナツの系列番号。商品名に固定で対応させ、絞り込みや並べ替えで色が変わらないようにする
pub(super) fn product_chart_index(label: &str) -> Option<usize> {
    match label {
        labels::DOMESTIC_STOCK => Some(0),
        labels::MUTUAL_FUND => Some(1),
        _ => None,
    }
}

/// 口座別の系列番号。口座名はバッジと同じ表示名に正規化してから対応表に当てる
pub(super) fn account_chart_index(label: &str) -> Option<usize> {
    match account_display_name(label) {
        labels::ACCOUNT_SPECIFIC => Some(0),
        labels::ACCOUNT_NISA_GROWTH => Some(1),
        labels::ACCOUNT_OLD_NISA => Some(2),
        labels::ACCOUNT_SPECIFIC_GENERAL => Some(3),
        labels::ACCOUNT_NISA_TSUMITATE => Some(4),
        labels::ACCOUNT_GENERAL => Some(5),
        _ => None,
    }
}

pub(super) fn chart_color(
    label: &str,
    fixed: impl Fn(&str) -> Option<usize>,
) -> (&'static str, &'static str) {
    if let Some(index) = fixed(label) {
        return CHART_COLORS[index % CHART_COLORS.len()];
    }
    let index = label.bytes().fold(2_166_136_261_u32, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(16_777_619)
    }) as usize
        % NON_AQUA_CHART_COLORS.len();
    CHART_COLORS[NON_AQUA_CHART_COLORS[index]]
}

pub(super) fn slice_style(
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
pub(super) fn CompositionCard(data: HomeOverviewData) -> impl IntoView {
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
                        view! { <ProductDonut products=products /> }.into_any()
                    }}
                    {if !accounts.is_empty() {
                        view! { <AccountFundBar accounts=accounts /> }.into_any()
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
fn ProductDonut(products: Vec<CompositionSlice>) -> impl IntoView {
    let product_total: f64 = products.iter().map(|slice| slice.amount).sum();
    let (donut, legend) = slice_style(&products, product_total, product_chart_index);
    let rows = products
        .iter()
        .zip(legend.iter())
        .map(|(slice, (class, pct))| {
            view! {
                <li class="flex items-center gap-1.5">
                    <span class=format!("home-swatch {class}")></span>
                    <span class="truncate text-text-soft">{slice.label.clone()}</span>
                    <span class="text-cell tabular-nums text-text-muted">
                        {format_integer_percent(*pct)}
                    </span>
                </li>
            }
        })
        .collect_view();
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
            <ul class="home-composition-legend grid gap-2 text-body">{rows}</ul>
        </div>
    }
}

#[component]
fn AccountFundBar(accounts: Vec<CompositionSlice>) -> impl IntoView {
    let account_total: f64 = accounts.iter().map(|slice| slice.amount).sum();
    let bars = accounts
        .iter()
        .map(|slice| {
            let width = slice.amount / account_total * 100.0;
            let class = chart_color(&slice.label, account_chart_index).1;
            view! {
                <span
                    class=format!("block h-full {class}")
                    style:width=format!("{width:.2}%")
                />
            }
        })
        .collect_view();
    let legend = accounts
        .iter()
        .map(|slice| {
            let class = chart_color(&slice.label, account_chart_index).1;
            let pct = slice.amount / account_total * 100.0;
            view! {
                <li class="flex items-center gap-1">
                    <span class=format!("home-swatch {class}")></span>
                    <AccountBadge label=slice.label.clone() />
                    <span class="text-cell tabular-nums text-text-muted">
                        {format_integer_percent(pct)}
                    </span>
                </li>
            }
        })
        .collect_view();
    view! {
        <div class="px-4 pt-4">
            <p class="text-label font-medium text-text-muted">
                {labels::HOME_ACCOUNT_FUND_LABEL}
            </p>
            <div class="home-stacked-bar mt-1.5">{bars}</div>
            <ul class="home-account-legend mt-1.5 flex flex-wrap gap-x-3 gap-y-0.5 text-cell">{legend}</ul>
        </div>
    }
}
