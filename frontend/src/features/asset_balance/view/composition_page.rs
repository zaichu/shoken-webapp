use super::use_asset_balance_state;
use crate::features::asset_balance::format::format_amount;
use crate::features::asset_balance::fund::{FundView, fund_view};
use crate::features::asset_balance::holdings::{HoldingView, holding_dividend};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::asset_balance::portfolio::{
    CompositionBasis, CompositionGroup, CompositionRow, composition_entries,
    composition_percentages, composition_rows, composition_sources,
};
use crate::features::asset_balance::store::{
    all_portfolio_views, load_asset_balances, load_fund_balances,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::use_session;
use crate::ui::empty_state::EmptyState;
use crate::ui::format::{format_currency, format_fixed_percent};
use crate::ui::labels::{
    COMPOSITION, COMPOSITION_BASIS_LABEL, COMPOSITION_DIVIDEND_BASIS, COMPOSITION_GROUP_LABEL,
    MARKET_VALUE, MISSING, TOTAL_ACQUISITION, composition_accounts, composition_basis_note,
    composition_kinds, composition_total,
};
use crate::ui::list_row::{ListRow, ListRowStatic};
use crate::ui::page_frame::PageFrame;
use crate::ui::page_skeleton::SkeletonNote;
use crate::ui::segmented::Segmented;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use crate::ui::underline_tabs::{focus_tab_by_id, next_tab_index};
use leptos::prelude::*;

/// 構成比の母数。比率と同じく絶対値合計を使う。負の取得額があると
/// 符号付き合計と比率が食い違うため
pub(crate) fn composition_basis(views: &[HoldingView]) -> f64 {
    views.iter().map(|v| v.purchase.abs()).sum()
}

const BASIS_OPTIONS: &[(&str, &str)] = &[
    ("market_value", MARKET_VALUE),
    ("dividend", COMPOSITION_DIVIDEND_BASIS),
];

/// 行ごとの基準金額。算出不可は `None`。投資信託の年間配当は
/// 1株配当の見積もりが無いため算出不可にする
fn basis_values(
    views: &[HoldingView],
    funds: &[FundView],
    basis: CompositionBasis,
    dividends: &DividendMaps,
) -> Vec<Option<f64>> {
    views
        .iter()
        .map(|view| match basis {
            CompositionBasis::MarketValue => {
                holding_valuation(&ValuationInput::from(view)).market_value
            }
            CompositionBasis::Dividend => {
                holding_dividend(&view.code, view.shares, view.average_price, dividends).annual
            }
        })
        .chain(funds.iter().map(|view| match basis {
            CompositionBasis::MarketValue => Some(view.market_value),
            CompositionBasis::Dividend => None,
        }))
        .collect()
}

/// 区分の色に対応する塗りクラス
fn color_fill_class(color: usize) -> String {
    if color == usize::MAX {
        "bg-text-muted".to_string()
    } else {
        format!("bg-chart-{}", color + 1)
    }
}

fn color_slice_class(color: usize) -> &'static str {
    match color {
        0 => "comp-slice-0",
        1 => "comp-slice-1",
        2 => "comp-slice-2",
        3 => "comp-slice-3",
        4 => "comp-slice-4",
        _ => "comp-slice-other",
    }
}

#[derive(Clone)]
struct ChartItem {
    view: HoldingView,
    percentage: Option<f64>,
}

#[component]
pub fn AssetBalanceCompositionPage() -> impl IntoView {
    let session = use_session();
    let state = use_asset_balance_state(session);
    let balances = state.balances;
    let funds = state.funds;
    let dividends = state.dividends;
    let lookup = state.lookup;
    let data_ops = state.data_ops;
    let csv_slot = state.csv.csv;
    if session.user.get_untracked().is_some() {
        let generation = session.generation.get_untracked();
        crate::features::asset_balance::store::load_asset_balances(
            session, generation, balances, dividends, lookup, csv_slot, data_ops,
        );
        load_fund_balances(session, generation, funds, data_ops);
    }
    let resolved = move || {
        let generation = session.generation.get();
        balances.with(|slot| {
            crate::features::asset_balance::store::csv::resolve_asset_balance(generation, slot)
        })
    };
    let resolved_funds = move || {
        let generation = session.generation.get();
        funds.with(|slot| {
            crate::features::asset_balance::store::csv::resolve_fund_balance(generation, slot)
        })
    };
    let reload = move || {
        if session.user.get_untracked().is_none() {
            return;
        }
        let generation = session.generation.get_untracked();
        balances.set(None);
        load_asset_balances(
            session, generation, balances, dividends, lookup, csv_slot, data_ops,
        );
        funds.set(None);
        load_fund_balances(session, generation, funds, data_ops);
    };
    view! {
        <PageFrame title=COMPOSITION back_href="/assetbalance" back_label="資産管理に戻る">
            <div class="flex flex-1 flex-col">
                {move || {
                    data_ops
                        .with(|ops| ops.refresh_error.clone())
                        .map(|message| {
                            view! {
                                <div class="no-print mb-3">
                                    <Alert variant=AlertVariant::Danger>
                                        <strong>"エラー:"</strong>
                                        " "
                                        {message}
                                    </Alert>
                                </div>
                            }
                        })
                }}
                {move || {
                    let generation = session.generation.get();
                    let list_error = balances.with(|slot| match slot {
                        Some((cached, Err(message))) if *cached == generation => {
                            Some(message.clone())
                        }
                        _ => None,
                    });
                    let fund_error = funds.with(|slot| match slot {
                        Some((cached, Err(message))) if *cached == generation => {
                            Some(message.clone())
                        }
                        _ => None,
                    });
                    let view = resolved();
                    let fund_resolved = resolved_funds();
                    if let Some(message) = list_error.or(fund_error) {
                        return view! { <ListLoadError message=message on_retry=reload /> }
                            .into_any();
                    }
                    match (view, fund_resolved) {
                        (None, _) | (_, None) => view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }.into_any(),
                        (Some(view), Some(fund_data)) => {
                            let generation = session.generation.get();
                            let views = all_portfolio_views(
                                &view.rows,
                                lookup,
                                generation,
                            );
                            let fund_views: Vec<FundView> = fund_data
                                .rows
                                .iter()
                                .map(fund_view)
                                .collect();
                            if views.is_empty() && fund_views.is_empty() {
                                return view! {
                                    <div class="px-4 py-4">
                                        <EmptyState title="データがありません" />
                                    </div>
                                }
                                    .into_any();
                            }
                            view! {
                                <CompositionMobile
                                    views=views.clone()
                                    funds=fund_views
                                    dividends=dividends
                                />
                                <div
                                    class="hidden lg:block"
                                    data-testid="asset-composition-desktop"
                                >
                                    <CompositionDesktop views=views />
                                </div>
                            }
                                .into_any()
                        }
                    }
                }}
            </div>
        </PageFrame>
    }
}

/// PC の構成比。
#[component]
fn CompositionDesktop(views: Vec<HoldingView>) -> impl IntoView {
    let total_purchase = composition_basis(&views);
    let chart_values: Vec<f64> = views.iter().map(|v| v.purchase).collect();
    let plan = crate::features::asset_balance::portfolio::chart_plan(&chart_values);
    let items: Vec<ChartItem> = plan
        .order
        .iter()
        .zip(&plan.percentages)
        .map(|(&index, &percentage)| ChartItem {
            view: views[index].clone(),
            percentage,
        })
        .collect();
    view! {
        <div class="asset-panel">
            <div class="border-b border-border p-4">
                <p class="text-sm text-text-muted">"対象: 保存済みの保有全体"</p>
                <p class="mt-1 text-sm">
                    "基準("
                    {TOTAL_ACQUISITION}
                    "): "
                    <strong class="tabular-nums">{format_amount(total_purchase)}</strong>
                </p>
            </div>
            {if total_purchase <= 0.0 {
                view! {
                    <div class="p-6 text-center text-text-muted">"算出不可"</div>
                }
                    .into_any()
            } else {
                view! {
                    <ul class="divide-y divide-border-faint" data-testid="asset-composition-list">
                        {items
                            .into_iter()
                            .map(|item| {
                                let name = item.view.name.clone();
                                let code = item.view.code.clone();
                                let purchase = item.view.purchase;
                                let percentage = item.percentage
                                    .map_or(MISSING.to_string(), |p| {
                                        format_fixed_percent(p, 1)
                                    });
                                let bar_width = item.percentage
                                    .map_or("0%".to_string(), |p| {
                                        format!("{}%", p.min(100.0))
                                    });
                                let inner = view! {
                                    <>
                                        <div class="flex items-baseline justify-between gap-2">
                                            <span class="min-w-0 truncate font-medium text-text">
                                                {name}
                                            </span>
                                            <span class="shrink-0 tabular-nums text-text">
                                                {format_currency(purchase)}
                                                " "
                                                <span class="text-text-muted">{percentage}</span>
                                            </span>
                                        </div>
                                        <div class="mt-1 text-sm text-text-muted">{code}</div>
                                        <div class="mt-2 h-1.5 w-full rounded-full bg-surface-raised">
                                            <div
                                                class="h-full rounded-full bg-accent"
                                                style:width=bar_width
                                            />
                                        </div>
                                    </>
                                };
                                view! {
                                    <li>
                                        {match &item.view.id {
                                            Some(id) => view! {
                                                <a
                                                    href=format!("/assetbalance/holdings/{id}")
                                                    class="asset-comp-link"
                                                >
                                                    {inner}
                                                </a>
                                            }
                                                .into_any(),
                                            None => view! { <div class="asset-comp-link">{inner}</div> }
                                                .into_any(),
                                        }}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                    .into_any()
            }}
        </div>
    }
}

/// スマホの構成比(下線タブ・基準切り替え・ドーナツ・一覧)。
/// 国内株式と投資信託の両方を区分に入れる
#[component]
fn CompositionMobile(
    views: Vec<HoldingView>,
    funds: Vec<FundView>,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    let group = RwSignal::new(CompositionGroup::Holding);
    let basis = RwSignal::new(CompositionBasis::MarketValue);
    let basis_value = Signal::derive(move || basis.get().value().to_string());
    let sources = composition_sources(&views, &funds);
    view! {
        <div class="lg:hidden" data-testid="asset-composition-mobile">
            <div
                class="comp-tab-row"
            >
            <div
                class="page-tab-list"
                role="tablist"
                aria-label=COMPOSITION_GROUP_LABEL
                data-testid="asset-composition-tabs"
            >
                {CompositionGroup::ALL
                    .into_iter()
                    .map(|item| {
                        let selected = move || group.get() == item;
                        view! {
                            <button
                                type="button"
                                role="tab"
                                id=format!("comp-tab-{}", item.value())
                                aria-selected=move || selected().to_string()
                                tabindex=move || if selected() { "0" } else { "-1" }
                                class="page-tab"
                                data-testid=format!("comp-tab-{}", item.value())
                                on:click=move |_| group.set(item)
                                on:keydown=move |event| {
                                    let current = CompositionGroup::ALL
                                        .iter()
                                        .position(|candidate| {
                                            *candidate == group.get_untracked()
                                        })
                                        .unwrap_or(0);
                                    if let Some(next) = next_tab_index(
                                        CompositionGroup::ALL.len(),
                                        current,
                                        &event.key(),
                                    ) {
                                        event.prevent_default();
                                        group.set(CompositionGroup::ALL[next]);
                                        focus_tab_by_id(
                                            &format!(
                                                "comp-tab-{}",
                                                CompositionGroup::ALL[next].value()
                                            ),
                                        );
                                    }
                                }
                            >
                                {item.label()}
                            </button>
                        }
                    })
                    .collect_view()}
            </div>
            </div>
            <div class="comp-basis-row">
                <Segmented
                    options=BASIS_OPTIONS
                    value=basis_value
                    on_change=move |value| {
                        if value == "dividend" {
                            basis.set(CompositionBasis::Dividend);
                        } else {
                            basis.set(CompositionBasis::MarketValue);
                        }
                    }
                    aria_label=COMPOSITION_BASIS_LABEL
                    testid="comp-basis"
                />
            </div>
            {move || {
                let current_basis = basis.get();
                let current_group = group.get();
                let values = basis_values(&views, &funds, current_basis, &dividends.get());
                let entries = composition_entries(&sources, current_group, &values);
                let raw: Vec<Option<f64>> = entries
                    .iter()
                    .map(|entry| entry.value)
                    .collect();
                let (total, percentages) = composition_percentages(&raw);
                let rows = composition_rows(&entries, &percentages);
                let center = match current_group {
                    CompositionGroup::Holding => composition_total(entries.len()),
                    CompositionGroup::Product => composition_kinds(entries.len()),
                    CompositionGroup::Account => composition_accounts(entries.len()),
                };
                view! {
                    <SkeletonNote note=composition_basis_note(
                        current_basis.label(),
                        &format_amount(total),
                    ) />
                    {if total <= 0.0 {
                        view! {
                            <div class="px-4 py-4">
                                <EmptyState title="算出不可" />
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <CompositionDonut rows=rows.clone() center=center />
                            <ul
                                class="asset-compact-list"
                                data-testid="asset-composition-mobile-list"
                            >
                                {rows
                                    .into_iter()
                                    .map(|row| {
                                        let fill = color_fill_class(row.color);
                                        let percentage = if row.computable {
                                            format_fixed_percent(row.percentage, 1)
                                        } else {
                                            MISSING.to_string()
                                        };
                                        let amount = row
                                            .value
                                            .map_or(MISSING.to_string(), format_currency);
                                        let bar_width = if row.computable {
                                            format!("{}%", row.percentage.min(100.0))
                                        } else {
                                            "0%".to_string()
                                        };
                                        let name = row.name.clone();
                                        let sub = row.sub.clone();
                                        let row_body = view! {
                                            <ListRow
                                                name=move || name.clone()
                                                primary=move || amount.clone()
                                                meta=move || sub.clone()
                                                secondary=move || percentage.clone()
                                            />
                                            <div class="comp-bar" aria-hidden="true">
                                                <span
                                                    class=format!("comp-bar-fill {fill}")
                                                    style:width=bar_width
                                                />
                                            </div>
                                        };
                                        let href = row.href.clone();
                                        let static_name = row.name.clone();
                                        let static_sub = row.sub.clone();
                                        let static_value = row.value;
                                        let static_computable = row.computable;
                                        let static_percentage = row.percentage;
                                        view! {
                                            <li
                                                class="list-row-item"
                                                data-testid="comp-row"
                                            >
                                                {match href {
                                                    Some(href) => view! {
                                                        <a href=href class="list-row-action">
                                                            {row_body}
                                                        </a>
                                                    }
                                                        .into_any(),
                                                    None => view! {
                                                        <ListRowStatic
                                                            name=move || static_name.clone()
                                                            primary=move || static_value.map_or(MISSING.to_string(), format_currency)
                                                            meta=move || static_sub.clone()
                                                            secondary=move || if static_computable { format_fixed_percent(static_percentage, 1) } else { MISSING.to_string() }
                                                        />
                                                    }
                                                        .into_any(),
                                                }}
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        }
                            .into_any()
                    }}
                }
            }}
        </div>
    }
}

/// ドーナツ。区分の割合を環の長さで出し、中央に件数を出す
#[component]
fn CompositionDonut(rows: Vec<CompositionRow>, center: String) -> impl IntoView {
    let mut cursor = 0.0;
    let slices: Vec<(CompositionRow, f64)> = rows
        .into_iter()
        .map(|row| {
            let start = cursor;
            cursor += row.percentage;
            (row, start)
        })
        .collect();
    view! {
        <div class="comp-donut-wrap">
            <svg
                class="comp-donut"
                viewBox="0 0 200 200"
                role="img"
                aria-label=COMPOSITION
                data-testid="asset-composition-donut"
            >
                <circle cx="100" cy="100" r="80" class="comp-donut-track" />
                {slices
                    .into_iter()
                    .filter(|(row, _)| row.percentage > 0.0)
                    .map(|(row, start)| {
                        let slice_class = color_slice_class(row.color);
                        let testid = if row.is_other {
                            "comp-slice-other"
                        } else {
                            "comp-slice"
                        };
                        view! {
                            <g data-testid=testid>
                                <circle
                                    cx="100"
                                    cy="100"
                                    r="80"
                                    pathLength="100"
                                    stroke-dasharray=format!(
                                        "{} {}",
                                        row.percentage,
                                        100.0 - row.percentage,
                                    )
                                    stroke-dashoffset=format!("{}", -start)
                                    transform="rotate(-90 100 100)"
                                    class=slice_class
                                />
                            </g>
                        }
                    })
                    .collect_view()}
            </svg>
            <div class="comp-donut-center" aria-hidden="true">
                {center}
            </div>
        </div>
    }
}
