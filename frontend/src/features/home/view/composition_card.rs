use super::super::model::{self, CompositionSlice, ProductCompositionSlice};
use super::super::store::{self, HomeOverviewData};
use super::card_parts::{CardError, CardHead, CardSkeleton};
use super::{fund_market, funds_of, stock_market, stocks_of};
use crate::features::asset_balance::format_valuation_amount;
use crate::session::use_session;
use crate::ui::badge::account_display_name;
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::format::format_integer_percent;
use crate::ui::labels;
use crate::ui::more_link::MoreLink;
use leptos::prelude::*;

// 商品別の積み上げ帯と口座別の系列で使う現行チャートトークン。
pub(super) const CHART_COLORS: &[(&str, &str)] = &[
    ("var(--color-chart-1)", "bg-chart-1"),
    ("var(--color-chart-2)", "bg-chart-2"),
    ("var(--color-chart-3)", "bg-chart-3"),
    ("var(--color-chart-4)", "bg-chart-4"),
    ("var(--color-chart-5)", "bg-chart-5"),
    ("var(--color-chart-6)", "bg-chart-6"),
];
const NON_AQUA_CHART_COLORS: [usize; 5] = [0, 1, 2, 4, 5];

/// 商品名に固定で対応させ、並べ替えで色が変わらないようにする
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
                let products = model::product_slices(
                    stocks_of(&assets).unwrap_or_default(),
                    funds_of(&funds).unwrap_or_default(),
                    stock_market,
                    fund_market,
                );
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
fn ProductDonut(products: Vec<ProductCompositionSlice>) -> impl IntoView {
    let product_total: f64 = products.iter().map(|slice| slice.amount).sum();
    let rows = products
        .iter()
        .map(|slice| {
            let pct = slice.amount / product_total * 100.0;
            view! {
                <CompositionRow
                    label=slice.label.clone()
                    amount=slice.amount
                    detail=format!(
                        "{}{}{}",
                        format_integer_percent(pct),
                        labels::HOME_COMPOSITION_DETAIL_SEPARATOR,
                        labels::count(slice.item_count)
                    )
                />
            }
        })
        .collect_view();
    let bar = products
        .iter()
        .map(|slice| {
            let width = slice.amount / product_total * 100.0;
            let class = chart_color(&slice.label, product_chart_index).1;
            view! {
                <span
                    class=format!("block h-full {class}")
                    style:width=format!("{width:.2}%")
                />
            }
        })
        .collect_view();
    view! {
        <div class="px-4 pt-3.5">
            <div class="home-composition-total">
                <span class="text-text-muted">{labels::HOME_COMPOSITION_TOTAL}</span>
                <span class="font-bold tabular-nums">{format_valuation_amount(Some(product_total))}</span>
            </div>
            <div class="home-stacked-bar mt-3">{bar}</div>
            <ul class="home-composition-rows mt-1.5">{rows}</ul>
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
    let rows = accounts
        .iter()
        .map(|slice| {
            let pct = slice.amount / account_total * 100.0;
            view! {
                <CompositionRow
                    label=slice.label.clone()
                    amount=slice.amount
                    detail=format_integer_percent(pct)
                />
            }
        })
        .collect_view();
    view! {
        <div class="px-4 pt-4">
            <p class="text-label font-medium text-text-muted">
                {labels::HOME_ACCOUNT_FUND_LABEL}
            </p>
            <div class="home-stacked-bar mt-1.5">{bars}</div>
            <ul class="home-composition-rows mt-1.5">{rows}</ul>
        </div>
    }
}

#[component]
fn CompositionRow(label: String, amount: f64, detail: String) -> impl IntoView {
    view! {
        <li class="home-composition-row">
            <div class="min-w-0 flex-1">
                <span class="block truncate text-body text-text">{label}</span>
                <span class="block text-cell tabular-nums text-text-muted">{detail}</span>
            </div>
            <span class="shrink-0 text-right text-body font-bold tabular-nums text-text">
                {format_valuation_amount(Some(amount))}
            </span>
        </li>
    }
}
