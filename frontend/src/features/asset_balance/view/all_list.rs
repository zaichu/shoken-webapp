//! 「すべて」タブ。国内株式と投資信託を同じ一覧に並べる

use super::sort_header::sort_header;
use crate::features::asset_balance::format::{format_row_valuation_amount, pnl_cells};
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::asset_balance::portfolio::SortOrder;
use leptos::prelude::*;

/// 株とファンドをまとめる一覧行
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UnifiedRow {
    pub name: String,
    pub kind: &'static str,
    pub sub: String,
    pub market: Option<f64>,
    pub profit: Option<f64>,
    pub rate: Option<f64>,
    pub href: Option<String>,
}

pub(crate) fn unified_from_stock(view: &HoldingView) -> UnifiedRow {
    let valuation = holding_valuation(&ValuationInput::from(view));
    UnifiedRow {
        name: view.name.clone(),
        kind: "国内株式",
        sub: view.code.clone(),
        market: valuation.market_value,
        profit: valuation.profit_loss.amount,
        rate: valuation.profit_loss.rate,
        href: view
            .id
            .clone()
            .map(|id| format!("/assetbalance/holdings/{id}")),
    }
}

pub(crate) fn unified_from_fund(view: &FundView) -> UnifiedRow {
    let sub = match &view.course {
        Some(course) => format!("{} ・ {course}", view.account),
        None => view.account.clone(),
    };
    UnifiedRow {
        name: view.name.clone(),
        kind: "投資信託",
        sub,
        market: Some(view.market_value),
        profit: Some(view.profit_loss),
        rate: view.profit_loss_rate,
        href: view
            .id
            .clone()
            .map(|id| format!("/assetbalance/funds/{id}")),
    }
}

/// 「すべて」タブの並べ替え。算出不能は末尾、同値は名前順
pub(crate) fn sort_unified(rows: &[UnifiedRow], key: FundSortKey, order: SortOrder) -> Vec<usize> {
    let value = |row: &UnifiedRow| match key {
        FundSortKey::MarketValue => row.market,
        _ => row.profit,
    };
    let mut indexed: Vec<usize> = (0..rows.len()).collect();
    indexed.sort_by(|&a, &b| {
        let ordering = match (value(&rows[a]), value(&rows[b])) {
            (Some(a), Some(b)) => {
                let ordering = b.total_cmp(&a);
                match order {
                    SortOrder::Desc => ordering,
                    SortOrder::Asc => ordering.reverse(),
                }
            }
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
        };
        if ordering == std::cmp::Ordering::Equal {
            rows[a].name.cmp(&rows[b].name)
        } else {
            ordering
        }
    });
    indexed
}

#[component]
pub fn AssetMergedTable(
    rows: Vec<UnifiedRow>,
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let sorted = Memo::new(move |_| {
        sort_unified(&rows, sort_key.get(), sort_order.get())
            .into_iter()
            .map(|index| rows[index].clone())
            .collect::<Vec<_>>()
    });
    view! {
        <div data-testid="asset-merged-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col">"銘柄・ファンド"</th>
                        {sort_header("評価額", FundSortKey::MarketValue, sort_key, sort_order)}
                        {sort_header("評価損益", FundSortKey::ProfitLoss, sort_key, sort_order)}
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        sorted
                            .get()
                            .into_iter()
                            .map(|row| {
                                let row = &row;
                                let (pnl_text, rate_text, pnl_text_class, pnl_badge_class) =
                                    pnl_cells(row.profit, row.rate);
                                let name_cell = view! {
                                    <>
                                        {row.name.clone()}
                                        <div class="asset-price-date">
                                            {format!("{} ・ {}", row.kind, row.sub)}
                                        </div>
                                    </>
                                };
                                view! {
                                    <tr>
                                        <td>
                                            {match &row.href {
                                                Some(href) => view! {
                                                    <a
                                                        href=href.clone()
                                                        class="inline-flex min-h-6 items-center no-underline hover:underline"
                                                    >
                                                        {name_cell}
                                                    </a>
                                                }
                                                    .into_any(),
                                                None => name_cell.into_any(),
                                            }}
                                        </td>
                                        <td class="text-right font-bold tabular-nums text-brand-text">
                                            {format_row_valuation_amount(row.market)}
                                        </td>
                                        <td class="text-right tabular-nums">
                                            {if row.profit.is_some() {
                                                view! {
                                                    <span class=format!(
                                                        "font-bold tabular-nums {pnl_text_class}",
                                                    )>
                                                        {pnl_text}
                                                    </span>
                                                    {row.rate.is_some().then(|| {
                                                        view! {
                                                            <span class=pnl_badge_class>
                                                                {rate_text.clone()}
                                                            </span>
                                                        }
                                                    })}
                                                }
                                                    .into_any()
                                            } else {
                                                view! { "—" }.into_any()
                                            }}
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </div>
    }
}

#[component]
pub fn AssetMergedList(rows: Vec<UnifiedRow>) -> impl IntoView {
    view! {
        <ul class="asset-compact-list" data-testid="asset-merged-list">
            {rows
                .into_iter()
                .map(|row| {
                    let (pnl_text, rate_text, pnl_text_class, pnl_badge_class) =
                        pnl_cells(row.profit, row.rate);
                    let inner = view! {
                        <>
                            <div class="flex items-baseline justify-between gap-2">
                                <span class="min-w-0 truncate font-medium text-text">
                                    {row.name.clone()}
                                </span>
                                <span class="shrink-0 font-bold tabular-nums text-brand-text">
                                    {format_row_valuation_amount(row.market)}
                                </span>
                            </div>
                            <div class="mt-0.5 flex items-baseline justify-between gap-2 text-xs">
                                <span class="min-w-0 truncate text-text-muted">
                                    {format!("{} ・ {}", row.kind, row.sub)}
                                </span>
                                {if row.profit.is_some() {
                                    view! {
                                        <span class="flex shrink-0 items-baseline">
                                            <span class=format!(
                                                "font-bold tabular-nums {pnl_text_class}",
                                            )>
                                                {pnl_text}
                                            </span>
                                            {row.rate.is_some().then(|| {
                                                view! {
                                                    <span class=pnl_badge_class>
                                                        {rate_text.clone()}
                                                    </span>
                                                }
                                            })}
                                        </span>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <span class="shrink-0 text-text-muted tabular-nums">
                                            "—"
                                        </span>
                                    }
                                        .into_any()
                                }}
                            </div>
                        </>
                    };
                    view! {
                        <li class="asset-compact-row" data-testid="asset-compact-row">
                            {match &row.href {
                                Some(href) => view! {
                                    <a href=href.clone() class="asset-compact-link">
                                        {inner}
                                    </a>
                                }
                                    .into_any(),
                                None => view! { <div class="asset-compact-link">{inner}</div> }
                                    .into_any(),
                            }}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}
