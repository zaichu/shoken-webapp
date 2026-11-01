use super::use_asset_balance_state;
use crate::features::asset_balance::format::format_amount;
use crate::features::asset_balance::holdings::{HoldingView, holding_dividend};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::asset_balance::portfolio::{
    CompositionBasis, CompositionGroup, CompositionRow, composition_entries,
    composition_label_fits, composition_percentages, composition_rows, composition_slice_arc,
};
use crate::features::asset_balance::store::{all_portfolio_views, load_asset_balances};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::use_session;
use crate::ui::format::{format_currency, format_fixed_percent};
use crate::ui::labels::{
    COMPOSITION, COMPOSITION_BACK, COMPOSITION_BASIS_LABEL, COMPOSITION_DIVIDEND_BASIS,
    COMPOSITION_GROUP_LABEL, MARKET_VALUE, MISSING, TOTAL_ACQUISITION, composition_basis_note,
    composition_total,
};
use crate::ui::page_frame::PageFrame;
use crate::ui::segmented::Segmented;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
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

/// 行ごとの基準金額。算出不可は `None`
fn basis_values(
    views: &[HoldingView],
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

fn color_label_class(color: usize) -> &'static str {
    match color {
        0 => "comp-slice-text-0",
        1 => "comp-slice-text-1",
        2 => "comp-slice-text-2",
        3 => "comp-slice-text-3",
        4 => "comp-slice-text-4",
        _ => "comp-slice-text-other",
    }
}

/// ドーナツ内に収めるための区分名。長い名前は先頭だけ出し、一覧で全文を見せる
fn donut_display_name(name: &str) -> String {
    const MAX: usize = 10;
    if name.chars().count() > MAX {
        format!("{}…", name.chars().take(MAX).collect::<String>())
    } else {
        name.to_string()
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
    let dividends = state.dividends;
    let lookup = state.lookup;
    let data_ops = state.data_ops;
    let csv_slot = state.csv.csv;
    if session.user.get_untracked().is_some() {
        crate::features::asset_balance::store::load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            csv_slot,
            data_ops,
        );
    }
    let resolved = move || {
        let generation = session.generation.get();
        balances.with(|slot| {
            crate::features::asset_balance::store::csv::resolve_asset_balance(generation, slot)
        })
    };
    let reload = move || {
        if session.user.get_untracked().is_none() {
            return;
        }
        balances.set(None);
        load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            csv_slot,
            data_ops,
        );
    };
    view! {
        <PageFrame title=COMPOSITION>
            <div class="page-gutter">
                <a class="comp-back-link" href="/assetbalance">
                    {COMPOSITION_BACK}
                </a>
            </div>
            <div class="page-gutter flex flex-1 flex-col py-4">
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
                    let view = resolved();
                    if let Some(message) = list_error {
                        return view! { <ListLoadError message=message on_retry=reload /> }
                            .into_any();
                    }
                    match view {
                        None => view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }.into_any(),
                        Some(view) if view.rows.is_empty() => {
                            view! {
                                <div class="asset-panel-padded">
                                    <p class="text-center text-text-muted">
                                        "データがありません"
                                    </p>
                                </div>
                            }
                                .into_any()
                        }
                        Some(view) => {
                            let generation = session.generation.get();
                            let views = all_portfolio_views(
                                &view.rows,
                                lookup,
                                generation,
                            );
                            view! {
                                <CompositionMobile
                                    views=views.clone()
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
#[component]
fn CompositionMobile(views: Vec<HoldingView>, dividends: RwSignal<DividendMaps>) -> impl IntoView {
    let group = RwSignal::new(CompositionGroup::Holding);
    let basis = RwSignal::new(CompositionBasis::MarketValue);
    let basis_value = Signal::derive(move || basis.get().value().to_string());
    view! {
        <div class="lg:hidden" data-testid="asset-composition-mobile">
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
                                    let next = match event.key().as_str() {
                                        "ArrowRight" => {
                                            Some((current + 1) % CompositionGroup::ALL.len())
                                        }
                                        "ArrowLeft" => {
                                            Some(
                                                (current + CompositionGroup::ALL.len() - 1)
                                                    % CompositionGroup::ALL.len(),
                                            )
                                        }
                                        "Home" => Some(0),
                                        "End" => Some(CompositionGroup::ALL.len() - 1),
                                        _ => None,
                                    };
                                    if let Some(next) = next {
                                        event.prevent_default();
                                        group.set(CompositionGroup::ALL[next]);
                                        if let Some(element) = web_sys::window()
                                            .and_then(|window| window.document())
                                            .and_then(|document| {
                                                document
                                                    .get_element_by_id(
                                                        &format!(
                                                            "comp-tab-{}",
                                                            CompositionGroup::ALL[next].value(),
                                                        ),
                                                    )
                                            })
                                            && let Some(element) = wasm_bindgen::JsCast::dyn_ref::<
                                                web_sys::HtmlElement,
                                            >(&element)
                                        {
                                            let _ = element.focus();
                                        }
                                    }
                                }
                            >
                                {item.label()}
                            </button>
                        }
                    })
                    .collect_view()}
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
                let values = basis_values(&views, current_basis, &dividends.get());
                let entries = composition_entries(&views, group.get(), &values);
                let raw: Vec<Option<f64>> = entries
                    .iter()
                    .map(|entry| entry.value)
                    .collect();
                let (total, percentages) = composition_percentages(&raw);
                let rows = composition_rows(&entries, &percentages);
                let holdings_count = views.len();
                view! {
                    <p class="comp-note" data-testid="asset-composition-note">
                        {composition_basis_note(current_basis.label(), &format_amount(total))}
                    </p>
                    {if total <= 0.0 {
                        view! {
                            <div class="comp-empty">"算出不可"</div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <CompositionDonut rows=rows.clone() total_count=holdings_count />
                            <ul
                                class="comp-list"
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
                                        let inner = view! {
                                            <div class="comp-row-main">
                                                <div class="comp-row-top">
                                                    <span class="comp-row-name">{name}</span>
                                                    <span class="comp-row-amount">
                                                        {amount}
                                                        " "
                                                        <span class="comp-row-pct">{percentage}</span>
                                                    </span>
                                                </div>
                                                {(!sub.is_empty())
                                                    .then(|| {
                                                        view! {
                                                            <div class="comp-row-sub">{sub}</div>
                                                        }
                                                    })}
                                                <div class="comp-bar" aria-hidden="true">
                                                    <span
                                                        class=format!("comp-bar-fill {fill}")
                                                        style:width=bar_width
                                                    />
                                                </div>
                                            </div>
                                        };
                                        view! {
                                            <li
                                                class="comp-row"
                                                data-testid="comp-row"
                                            >
                                                <span
                                                    class=format!("comp-dot {fill}")
                                                    aria-hidden="true"
                                                />
                                                {match &row.href {
                                                    Some(href) => view! {
                                                        <a href=href.clone() class="comp-row-link">
                                                            {inner}
                                                        </a>
                                                    }
                                                        .into_any(),
                                                    None => view! {
                                                        <div class="comp-row-link">{inner}</div>
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

/// ドーナツ。区分の割合を環の長さで出し、中央に全体の件数を出す.
/// 区分名は弧に沿わせるため、環の外に出ない
#[component]
fn CompositionDonut(rows: Vec<CompositionRow>, total_count: usize) -> impl IntoView {
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
                    .enumerate()
                    .filter(|(_, (row, _))| row.percentage > 0.0)
                    .map(|(index, (row, start))| {
                        let slice_class = color_slice_class(row.color);
                        let label_class = color_label_class(row.color);
                        let arc_id = format!("comp-arc-{index}");
                        let (arc_path, _) = composition_slice_arc(start, row.percentage);
                        let testid = if row.is_other {
                            "comp-slice-other"
                        } else {
                            "comp-slice"
                        };
                        let label = format!(
                            "{} {}",
                            donut_display_name(&row.name),
                            format_fixed_percent(row.percentage, 1),
                        );
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
                                {composition_label_fits(&label, row.percentage)
                                    .then(|| {
                                        view! {
                                            <>
                                                <path
                                                    id=arc_id.clone()
                                                    d=arc_path
                                                    class="fill-none"
                                                />
                                                <text
                                                    class=format!(
                                                        "comp-slice-label {label_class}"
                                                    )
                                                >
                                                    <textPath
                                                        href=format!("#{}", arc_id.clone())
                                                        startOffset="50%"
                                                        text-anchor="middle"
                                                    >
                                                        {label.clone()}
                                                    </textPath>
                                                </text>
                                            </>
                                        }
                                    })}
                            </g>
                        }
                    })
                    .collect_view()}
            </svg>
            <div class="comp-donut-center" aria-hidden="true">
                {composition_total(total_count)}
            </div>
        </div>
    }
}
