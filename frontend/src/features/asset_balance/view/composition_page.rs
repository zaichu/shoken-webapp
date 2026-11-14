use super::use_asset_balance_state;
use crate::features::asset_balance::format::format_amount;
use crate::features::asset_balance::fund::{FundView, fund_view};
use crate::features::asset_balance::holdings::{HoldingView, holding_dividend};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::asset_balance::portfolio::{
    CompositionBasis, CompositionGroup, CompositionRow, DonutGeometry, composition_entries,
    composition_legend_split, composition_percentages, composition_rows, composition_sources,
};
use crate::features::asset_balance::store::{
    all_portfolio_views, load_asset_balances, load_fund_balances,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::use_session;
use crate::ui::button::TabButton;
use crate::ui::empty_state::EmptyState;
use crate::ui::format::{format_currency, format_fixed_percent};
use crate::ui::labels;
use crate::ui::list_row::{ListRow, ListRowStatic};
use crate::ui::page_frame::PageFrame;
use crate::ui::page_skeleton::SkeletonNote;
use crate::ui::segmented::Segmented;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use crate::ui::underline_tabs::{focus_tab_by_id, next_tab_index};
use leptos::prelude::*;

const BASIS_OPTIONS: &[(&str, &str)] = &[
    ("market_value", labels::MARKET_VALUE),
    ("dividend", labels::COMPOSITION_DIVIDEND_BASIS),
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
    format!("bg-chart-{}", color + 1)
}

const fn color_slice_class(color: usize) -> &'static str {
    match color {
        0 => "comp-slice-0",
        1 => "comp-slice-1",
        2 => "comp-slice-2",
        3 => "comp-slice-3",
        _ => "comp-slice-4",
    }
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
    if session.user.get_untracked().is_some() {
        let generation = session.generation.get_untracked();
        crate::features::asset_balance::store::load_asset_balances(
            session, generation, balances, dividends, lookup, data_ops,
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
        load_asset_balances(session, generation, balances, dividends, lookup, data_ops);
        funds.set(None);
        load_fund_balances(session, generation, funds, data_ops);
    };
    view! {
        <PageFrame title=labels::COMPOSITION back_href="/assetbalance" back_label=labels::ASSET_BALANCE_BACK>
            <div class="flex flex-1 flex-col">
                {move || {
                    data_ops
                        .with(|ops| ops.refresh_error.clone())
                        .map(|message| {
                            view! {
                                <div class="no-print mb-3">
                                    <Alert variant=AlertVariant::Danger>
                                        <strong>{labels::ERROR_PREFIX}</strong>
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
                                        <EmptyState title=labels::NO_DATA />
                                    </div>
                                }
                                    .into_any();
                            }
                            view! {
                                <CompositionContent
                                    views=views.clone()
                                    funds=fund_views
                                    dividends=dividends
                                />
                            }
                                .into_any()
                        }
                    }
                }}
            </div>
        </PageFrame>
    }
}

/// 構成比(PCもスマホも同じ形。上にタブ・基準切り替え・ドーナツ、下に一覧)。
/// 国内株式と投資信託の両方を区分に入れる
#[component]
fn CompositionContent(
    views: Vec<HoldingView>,
    funds: Vec<FundView>,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    let group = RwSignal::new(CompositionGroup::Holding);
    let basis = RwSignal::new(CompositionBasis::MarketValue);
    let basis_value = Signal::derive(move || basis.get().value().to_string());
    let sources = composition_sources(&views, &funds);
    view! {
        <div class="mx-auto w-full max-w-2xl" data-testid="asset-composition">
            <div
                class="comp-tab-row"
            >
            <div
                class="page-tab-list"
                role="tablist"
                aria-label=labels::COMPOSITION_GROUP_LABEL
                data-testid="asset-composition-tabs"
            >
                {CompositionGroup::ALL
                    .into_iter()
                    .map(|item| {
                        let selected = move || group.get() == item;
                        view! {
                            <TabButton
                                id=format!("comp-tab-{}", item.value())
                                selected=selected
                                testid=format!("comp-tab-{}", item.value())
                                on_click=move |_| group.set(item)
                                on_keydown=move |event| {
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
                            </TabButton>
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
                    aria_label=labels::COMPOSITION_BASIS_LABEL
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
                    CompositionGroup::Holding => labels::composition_total(entries.len()),
                    CompositionGroup::Product => labels::composition_kinds(entries.len()),
                    CompositionGroup::Account => labels::composition_accounts(entries.len()),
                };
                view! {
                    <SkeletonNote note=labels::composition_basis_note(
                        current_basis.label(),
                        &format_amount(total),
                    ) />
                    {if total <= 0.0 {
                        view! {
                            <div class="px-4 py-4">
                                <EmptyState title=labels::COMPOSITION_UNAVAILABLE />
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <CompositionDonut rows=rows.clone() center=center />
                            <ul
                                class="asset-compact-list"
                                data-testid="asset-composition-list"
                            >
                                {rows
                                    .into_iter()
                                    .map(|row| {
                                        let fill = color_fill_class(row.color);
                                        let percentage = if row.computable {
                                            format_fixed_percent(row.percentage, 1)
                                        } else {
                                            labels::MISSING.to_string()
                                        };
                                        let amount = row
                                            .value
                                            .map_or(labels::MISSING.to_string(), format_currency);
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
                                        let static_fill = color_fill_class(row.color);
                                        let static_bar_width = if static_computable {
                                            format!("{}%", static_percentage.min(100.0))
                                        } else {
                                            "0%".to_string()
                                        };
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
                                                            primary=move || static_value.map_or(labels::MISSING.to_string(), format_currency)
                                                            meta=move || static_sub.clone()
                                                            secondary=move || if static_computable { format_fixed_percent(static_percentage, 1) } else { labels::MISSING.to_string() }
                                                        />
                                                        <div class="comp-bar" aria-hidden="true">
                                                            <span
                                                                class=format!("comp-bar-fill {static_fill}")
                                                                style:width=static_bar_width
                                                            />
                                                        </div>
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

/// ドーナツと横の凡例。切れは全部の区分を1切れずつ出し、
/// 凡例は比率の大きい順に上位5件、6件目以降は「ほか N 件」にまとめる。
/// 切れの間のすき間は地の色の線で空け、同じ色が隣り合っても区別できるようにする
#[component]
fn CompositionDonut(rows: Vec<CompositionRow>, center: String) -> impl IntoView {
    let geometry = DonutGeometry::SQUARE;
    let mut cursor = 0.0;
    let slices: Vec<(CompositionRow, f64)> = rows
        .clone()
        .into_iter()
        .map(|row| {
            let start = cursor;
            cursor += row.percentage;
            (row, start)
        })
        .collect();
    let center_x = geometry.center_x;
    let center_y = geometry.center_y;
    let radius = geometry.radius;
    let inner = geometry.inner();
    let outer = geometry.outer();
    let gaps: Vec<(f64, f64, f64, f64)> = slices
        .iter()
        .enumerate()
        .filter_map(|(index, (row, start))| {
            let previous = &slices[(index + slices.len() - 1) % slices.len()].0;
            if row.percentage < 1.0 || previous.percentage < 1.0 {
                return None;
            }
            let angle = (start / 100.0 * 360.0 - 90.0).to_radians();
            let (sine, cosine) = angle.sin_cos();
            Some((
                center_x + inner * cosine,
                center_y + inner * sine,
                center_x + outer * cosine,
                center_y + outer * sine,
            ))
        })
        .collect();
    let (visible, others) = composition_legend_split(rows.len());
    let legend: Vec<CompositionRow> = rows.into_iter().take(visible).collect();
    view! {
        <div class="comp-donut-row">
            <div class="comp-donut-wrap">
                <svg
                    class="comp-donut"
                    viewBox=format!("0 0 {} {}", geometry.width, geometry.height)
                    role="img"
                    aria-label=labels::COMPOSITION
                    data-testid="asset-composition-donut"
                >
                    <circle
                        cx=center_x.to_string()
                        cy=center_y.to_string()
                        r=radius.to_string()
                        class="comp-donut-track"
                    />
                    {slices
                        .iter()
                        .filter(|(row, _)| row.percentage > 0.0)
                        .map(|(row, start)| {
                            let slice_class = color_slice_class(row.color);
                            view! {
                                <g data-testid="comp-slice">
                                    <circle
                                        cx=center_x.to_string()
                                        cy=center_y.to_string()
                                        r=radius.to_string()
                                        pathLength="100"
                                        stroke-dasharray=format!(
                                            "{} {}",
                                            row.percentage,
                                            100.0 - row.percentage,
                                        )
                                        stroke-dashoffset=format!("{}", -start)
                                        transform=format!(
                                            "rotate(-90 {center_x} {center_y})",
                                        )
                                        class=slice_class
                                    />
                                </g>
                            }
                        })
                        .collect_view()}
                    {gaps
                        .into_iter()
                        .map(|(x1, y1, x2, y2)| {
                            view! {
                                <line
                                    x1=x1.to_string()
                                    y1=y1.to_string()
                                    x2=x2.to_string()
                                    y2=y2.to_string()
                                    class="comp-slice-gap"
                                />
                            }
                        })
                        .collect_view()}
                </svg>
                <div class="comp-donut-center" aria-hidden="true">
                    {center.clone()}
                </div>
            </div>
            <ul class="comp-legend" data-testid="asset-composition-legend">
                {legend
                    .into_iter()
                    .map(|row| {
                        let fill = color_fill_class(row.color);
                        let rate = if row.computable {
                            format_fixed_percent(row.percentage, 1)
                        } else {
                            labels::MISSING.to_string()
                        };
                        let name = row.name.clone();
                        view! {
                            <li class="comp-legend-item" data-testid="comp-legend-item">
                                <span
                                    class=format!("comp-legend-dot {fill}")
                                    aria-hidden="true"
                                />
                                <span class="comp-legend-name lg:whitespace-normal">{name}</span>
                                <span class="comp-legend-rate">{rate}</span>
                            </li>
                        }
                    })
                    .collect_view()}
                {if others > 0 {
                    view! {
                        <li
                            class="comp-legend-others"
                            data-testid="comp-legend-others"
                        >
                            {labels::composition_others(others)}
                        </li>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }}
            </ul>
        </div>
    }
}
