use super::use_asset_balance_state;
use crate::features::asset_balance;
use crate::features::asset_balance::format as balance_format;
use crate::features::asset_balance::fund;
use crate::features::asset_balance::fund::FundView;
use crate::features::asset_balance::holdings;
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model;
use crate::features::asset_balance::model::ValuationInput;
use crate::features::asset_balance::portfolio;
use crate::features::asset_balance::portfolio::{
    CompositionBasis, CompositionGroup, CompositionRow, DonutGeometry,
};
use crate::features::asset_balance::store;
use crate::features::dividend_per_share::DividendMaps;
use crate::session;
use crate::ui::amount::Amount;
use crate::ui::asset_balance::{AssetPageEmpty, AssetPageError, CompositionFrame};
use crate::ui::badge::AccountLegend;
use crate::ui::button::TabButton;
use crate::ui::format;
use crate::ui::labels;
use crate::ui::labels::UnavailableKind;
use crate::ui::list_row::{ListRow, ListRowStatic};
use crate::ui::page_frame::PageFrame;
use crate::ui::page_skeleton::SkeletonNote;
use crate::ui::segmented::Segmented;
use crate::ui::state::{ListLoadError, ListSkeleton, ListSkeletonVariant};
use crate::ui::underline_tabs;
use leptos::prelude::*;

const BASIS_OPTIONS: &[(&str, &str)] = &[
    ("market_value", labels::MARKET_VALUE),
    ("dividend", labels::ANNUAL_EXPECTED_DIVIDEND),
];

/// 行ごとの基準金額と算出不可の理由。投資信託の年間配当は
/// 1株配当の見積もりが無いため対象外にする。欠測・取得中・無配は
/// 株式の配当見積もりの状態で区別する
fn basis_cells(
    views: &[HoldingView],
    funds: &[FundView],
    basis: CompositionBasis,
    dividends: &DividendMaps,
) -> Vec<(Option<f64>, Option<labels::UnavailableKind>)> {
    use labels::UnavailableKind::{NoDividend, OutOfScope, Pending, Unavailable};
    views
        .iter()
        .map(|view| match basis {
            CompositionBasis::MarketValue => {
                let value = model::holding_valuation(&ValuationInput::from(view)).market_value;
                let cause = match value.filter(|value| *value != 0.0) {
                    Some(_) => None,
                    None => Some(Unavailable),
                };
                (value, cause)
            }
            CompositionBasis::Dividend => {
                let dividend = holdings::holding_dividend(
                    &view.code,
                    view.shares,
                    view.average_price,
                    dividends,
                );
                let cause = if !dividends.responded || dividend.status.as_deref() == Some("pending")
                {
                    Some(Pending)
                } else if asset_balance::is_no_dividend(&dividend) {
                    Some(NoDividend)
                } else if dividend.annual.as_ref().is_some_and(|value| *value != 0.0) {
                    None
                } else {
                    Some(Unavailable)
                };
                (dividend.annual, cause)
            }
        })
        .chain(funds.iter().map(|view| match basis {
            CompositionBasis::MarketValue => {
                let cause = match Some(view.market_value).filter(|value| *value != 0.0) {
                    Some(_) => None,
                    None => Some(Unavailable),
                };
                (Some(view.market_value), cause)
            }
            CompositionBasis::Dividend => (None, Some(OutOfScope)),
        }))
        .collect()
}

/// 算出不可の区分の金額欄の表示。無配は無配と出し、取得中は取得中と出す
fn missing_primary_text(missing: Option<UnavailableKind>) -> String {
    labels::unavailable_text(missing.unwrap_or(UnavailableKind::Unavailable)).to_string()
}

/// 算出不可の区分の割合欄の表示。無配の割合は 0.0% と出す
fn missing_secondary_text(missing: Option<UnavailableKind>, display_percent: f64) -> String {
    match missing {
        Some(UnavailableKind::NoDividend) => format::format_fixed_percent(display_percent, 1),
        Some(kind) => labels::unavailable_text(kind).to_string(),
        None => labels::FETCH_UNAVAILABLE.to_string(),
    }
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
        4 => "comp-slice-4",
        _ => "comp-slice-5",
    }
}

#[component]
pub fn AssetBalanceCompositionPage() -> impl IntoView {
    let session = session::use_session();
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
        store::load_fund_balances(session, generation, funds, data_ops);
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
        store::load_asset_balances(session, generation, balances, dividends, lookup, data_ops);
        funds.set(None);
        store::load_fund_balances(session, generation, funds, data_ops);
    };
    view! {
        <PageFrame title=labels::COMPOSITION back_href="/assetbalance" back_label=labels::ASSET_BALANCE_BACK>
            <div class="flex flex-1 flex-col">
                {move || {
                    data_ops
                        .with(|ops| ops.refresh_error.clone())
                        .map(|message| {
                            view! { <AssetPageError message=message /> }
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
                            let views = store::all_portfolio_views(
                                &view.rows,
                                lookup,
                                generation,
                            );
                            let fund_views: Vec<FundView> = fund_data
                                .rows
                                .iter()
                                .map(fund::fund_view)
                                .collect();
                            if views.is_empty() && fund_views.is_empty() {
                                return view! { <AssetPageEmpty title=labels::NO_DATA /> }
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
    let sources = portfolio::composition_sources(&views, &funds);
    view! {
        <CompositionFrame>
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
                                    if let Some(next) = underline_tabs::next_tab_index(
                                        CompositionGroup::ALL.len(),
                                        current,
                                        &event.key(),
                                    ) {
                                        event.prevent_default();
                                        group.set(CompositionGroup::ALL[next]);
                                        underline_tabs::focus_tab_by_id(
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
                let cells = basis_cells(&views, &funds, current_basis, &dividends.get());
                let values: Vec<Option<f64>> =
                    cells.iter().map(|(value, _)| *value).collect();
                let cell_causes: Vec<Option<labels::UnavailableKind>> =
                    cells.iter().map(|(_, cause)| *cause).collect();
                let entries = portfolio::composition_entries_with_causes(
                    &sources,
                    current_group,
                    &values,
                    &cell_causes,
                );
                let raw: Vec<Option<f64>> = entries
                    .iter()
                    .map(|entry| entry.value)
                    .collect();
                let (total, percentages) = portfolio::composition_percentages(&raw);
                let rows = portfolio::composition_rows(&entries, &percentages);
                let raw_abs: Vec<f64> =
                    raw.iter().map(|value| value.unwrap_or(0.0)).collect();
                let display_percents = model::largest_remainder_percents(&raw_abs, 1);
                let center = match current_group {
                    CompositionGroup::Holding => labels::composition_total(entries.len()),
                    CompositionGroup::Product => labels::composition_kinds(entries.len()),
                    CompositionGroup::Account => labels::composition_accounts(entries.len()),
                };
                view! {
                    <SkeletonNote note=labels::composition_basis_note(current_basis.label())>
                        <Amount text=balance_format::format_amount(total) />
                        {labels::COMPOSITION_NOTE_SUFFIX}
                    </SkeletonNote>
                    {if total <= 0.0 {
                        view! { <AssetPageEmpty title=labels::COMPOSITION_UNAVAILABLE /> }
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
                                    .enumerate()
                                    .map(|(index, row)| {
                                        let fill = color_fill_class(row.color);
                                        let static_computable = row.computable;
                                        let static_percentage = display_percents
                                            .get(index)
                                            .copied()
                                            .unwrap_or(row.percentage);
                                        let row_value = row.value;
                                        let missing_primary =
                                            missing_primary_text(row.missing);
                                        let missing_secondary = missing_secondary_text(
                                            row.missing,
                                            static_percentage,
                                        );
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
                                                primary=move || row_value.map_or_else(
                                                    || view! { {missing_primary.clone()} }.into_any(),
                                                    |value| view! {
                                                        <Amount
                                                            text=format::format_currency(value)
                                                        />
                                                    }.into_any(),
                                                )
                                                meta=move || sub.clone()
                                                secondary=move || view! {
                                                    {if static_computable {
                                                        view! {
                                                            <Amount
                                                                text=format::format_fixed_percent(static_percentage, 1)
                                                            />
                                                        }.into_any()
                                                    } else {
                                                        view! { {missing_secondary.clone()} }.into_any()
                                                    }}
                                                }
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
                                        let static_value = row_value;
                                        let static_primary_missing = missing_primary_text(row.missing);
                                        let static_secondary_missing = missing_secondary_text(
                                            row.missing,
                                            static_percentage,
                                        );
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
                                                {href.map_or_else(|| view! {
                                                        <ListRowStatic
                                                            name=move || static_name.clone()
                                                            primary=move || static_value.map_or_else(
                                                                || view! { {static_primary_missing.clone()} }.into_any(),
                                                                |value| view! {
                                                                    <Amount
                                                                        text=format::format_currency(value)
                                                                    />
                                                                }.into_any(),
                                                            )
                                                            meta=move || static_sub.clone()
                                                            secondary=move || if static_computable { view! {
                                                                <Amount
                                                                    text=format::format_fixed_percent(static_percentage, 1)
                                                                />
                                                            }.into_any() } else { view! { {static_secondary_missing.clone()} }.into_any() }
                                                        />
                                                        <div class="comp-bar" aria-hidden="true">
                                                            <span
                                                                class=format!("comp-bar-fill {static_fill}")
                                                                style:width=static_bar_width
                                                            />
                                                        </div>
                                                    }
                                                        .into_any(), |href| view! {
                                                        <a href=href class="list-row-action">
                                                            {row_body}
                                                        </a>
                                                    }
                                                        .into_any())}
                                            </li>
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                            {matches!(current_group, CompositionGroup::Account)
                                .then(|| view! { <AccountLegend /> })}
                        }
                            .into_any()
                    }}
                }
            }}
        </CompositionFrame>
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
    let amounts: Vec<f64> = rows.iter().map(|row| row.value.unwrap_or(0.0)).collect();
    let display = crate::features::asset_balance::model::largest_remainder_percents(&amounts, 1);
    let center_x = geometry.center_x;
    let center_y = geometry.center_y;
    let radius = geometry.radius;
    let inner = geometry.inner();
    let outer = geometry.outer();
    let (visible, others) = portfolio::composition_legend_split(rows.len());
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
                    {slices
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
                {rows
                    .into_iter()
                    .take(visible)
                    .enumerate()
                    .map(|(index, row)| {
                        let fill = color_fill_class(row.color);
                        let display_percent =
                            display.get(index).copied().unwrap_or(row.percentage);
                        let rate = if row.computable {
                            format::format_fixed_percent(display_percent, 1)
                        } else {
                            missing_secondary_text(row.missing, display_percent)
                        };
                        let name = row.name.clone();
                        view! {
                            <li class="comp-legend-item" data-testid="comp-legend-item">
                                <span
                                    class=format!("comp-legend-dot {fill}")
                                    aria-hidden="true"
                                />
                                <span class="comp-legend-name lg:whitespace-normal">{name}</span>
                                <span class="comp-legend-rate">
                                    <Amount text=rate />
                                </span>
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

#[cfg(test)]
mod tests {
    use super::{basis_cells, missing_primary_text, missing_secondary_text};
    use crate::features::asset_balance::fund::FundView;
    use crate::features::asset_balance::holdings::HoldingView;
    use crate::features::asset_balance::portfolio::CompositionBasis;
    use crate::features::dividend_per_share::DividendMaps;
    use crate::ui::labels::UnavailableKind;
    use std::collections::HashMap;

    fn holding(code: &str, current_price: f64) -> HoldingView {
        HoldingView {
            id: None,
            code: code.to_string(),
            name: "架空銘柄".to_string(),
            account: Some("特定".to_string()),
            shares: 100.0,
            average_price: 1000.0,
            purchase: 100_000.0,
            current_price,
            day_change: None,
            price_as_of: Some("2026-10-20".to_string()),
        }
    }

    fn fund(name: &str) -> FundView {
        FundView {
            id: Some(format!("fund-{name}")),
            name: name.to_string(),
            account: "特定".to_string(),
            course: None,
            shares: 10000.0,
            average_purchase_price: 10000.0,
            purchase: 100000.0,
            base_price: 11000.0,
            base_price_day_change: None,
            base_price_month_change: None,
            market_value: 110000.0,
            profit_loss: 10000.0,
            profit_loss_rate: None,
            total_return: None,
            price_as_of: Some("2026-10-20".to_string()),
            price_source: crate::api::dto::FundPriceSource::Api,
            is_nav_latest: true,
            expected_nav_date: None,
        }
    }

    fn maps_with(per_share: &[(&str, f64)], status: &[(&str, &str)]) -> DividendMaps {
        DividendMaps {
            per_share: per_share
                .iter()
                .map(|(code, value)| ((*code).to_string(), *value))
                .collect::<HashMap<_, _>>(),
            status: status
                .iter()
                .map(|(code, value)| ((*code).to_string(), (*value).to_string()))
                .collect::<HashMap<_, _>>(),
            responded: true,
        }
    }

    #[test]
    fn dividend_basis_distinguishes_fund_and_stock_reasons() {
        let views = vec![
            holding("9904", 2600.0),
            holding("9914", 2600.0),
            holding("9920", 2600.0),
            holding("9921", 2600.0),
        ];
        let funds = vec![fund("A")];
        // 9904 は値あり、9914 は取得失敗、9920 は取得中、9921 は無配
        let maps = DividendMaps {
            per_share: [("9904".to_string(), 90.0), ("9921".to_string(), 0.0)]
                .into_iter()
                .collect(),
            status: [
                ("9914".to_string(), "error".to_string()),
                ("9920".to_string(), "pending".to_string()),
                ("9921".to_string(), "zero".to_string()),
            ]
            .into_iter()
            .collect(),
            responded: true,
        };
        let cells = basis_cells(&views, &funds, CompositionBasis::Dividend, &maps);
        assert_eq!(cells.len(), 5);
        assert!(cells[0].0.is_some());
        assert_eq!(cells[0].1, None);
        assert_eq!(cells[1], (None, Some(UnavailableKind::Unavailable)));
        assert_eq!(cells[2], (None, Some(UnavailableKind::Pending)));
        assert_eq!(cells[3], (Some(0.0), Some(UnavailableKind::NoDividend)));
        assert_eq!(cells[4], (None, Some(UnavailableKind::OutOfScope)));
        // 無配の表示は金額も割合も無配の状態を残す
        assert_eq!(
            missing_primary_text(Some(UnavailableKind::NoDividend)),
            "無配"
        );
        assert_eq!(
            missing_secondary_text(Some(UnavailableKind::NoDividend), 0.0),
            "0.0%"
        );
        assert_eq!(
            missing_primary_text(Some(UnavailableKind::OutOfScope)),
            "対象外"
        );
    }

    #[test]
    fn market_basis_marks_unpriced_as_unavailable() {
        let views = vec![holding("9904", 2600.0), holding("9914", 0.0)];
        let cells = basis_cells(
            &views,
            &[],
            CompositionBasis::MarketValue,
            &maps_with(&[], &[]),
        );
        assert!(cells[0].0.is_some());
        assert_eq!(cells[0].1, None);
        assert_eq!(cells[1], (None, Some(UnavailableKind::Unavailable)));
    }

    #[test]
    fn dividend_basis_shows_pending_before_first_response() {
        let views = vec![holding("9904", 2600.0)];
        let funds = vec![fund("A")];
        // 推定の応答前は空のマップ。株式は取得中、投信は対象外のまま
        let cells = basis_cells(
            &views,
            &funds,
            CompositionBasis::Dividend,
            &DividendMaps::default(),
        );
        assert_eq!(cells[0], (None, Some(UnavailableKind::Pending)));
        assert_eq!(cells[1], (None, Some(UnavailableKind::OutOfScope)));
    }
}
