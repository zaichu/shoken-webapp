use super::palette::holding_bar_class;
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::format::{
    format_currency, format_fixed_percent, format_percentage_value, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
};
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model::{
    KpiHolding, ValuationInput, calculate_portfolio_kpi, portfolio_valuation, summary_valuation,
};
use crate::features::asset_balance::portfolio::chart_display;
use crate::features::asset_balance::store::FilteredPortfolio;
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use crate::ui::empty_state::EmptyState;
use crate::ui::summary_strip::{SummaryItem, SummaryStrip};
use leptos::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct ChartItem {
    pub(crate) view: HoldingView,
    pub(crate) percentage: Option<f64>,
}

#[component]
pub(crate) fn PortfolioSummary(
    filtered: FilteredPortfolio,
    fallback_totals: Option<AssetBalanceSummary>,
    on_clear_filter: impl Fn() + Send + Sync + 'static,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
) -> impl IntoView {
    let on_clear_filter = std::sync::Arc::new(on_clear_filter);
    let FilteredPortfolio { views, all_views } = filtered;
    let views_empty = views.is_empty();
    if views_empty {
        show_all.set(false);
    }
    if all_views.is_empty() {
        return view! {
            <div class="sum-card mb-3">
                <div class="sum-card-body">
                    <EmptyState
                        title="該当する銘柄がありません"
                        description="検索条件を変更するか、絞り込みを解除してください。"
                    />
                    <div class="mt-3 text-center">
                        <Button
                            variant=ButtonVariant::Ghost
                            on_click=move |_| on_clear_filter()
                        >
                            "絞り込みを解除"
                        </Button>
                    </div>
                </div>
            </div>
        }
        .into_any();
    }
    // 合計は絞り込み前の全保有分から求める(検索中も全保有分のまま)
    let kpi_holdings: Vec<KpiHolding> = all_views
        .iter()
        .map(|view| KpiHolding {
            security_code: view.code.clone(),
            shares: view.shares,
            total_purchase_amount: view.purchase,
        })
        .collect();
    let valuation_inputs: Vec<ValuationInput> =
        all_views.iter().map(ValuationInput::from).collect();
    let row_valuation = portfolio_valuation(&valuation_inputs);
    // 取得上限超過で行が欠けるときだけ API 集計で合計を補う(個別カードと一致させるため通常時は行から計算)
    let valuation = fallback_totals
        .as_ref()
        .map(summary_valuation)
        .unwrap_or(row_valuation);
    let kpi =
        Memo::new(move |_| calculate_portfolio_kpi(&kpi_holdings, &dividends.get().per_share));

    let market_text = format_valuation_amount(valuation.market_value);
    let mut market_note = "現在値の基準日は銘柄ごとに異なります".to_string();
    if valuation.has_unpriced {
        market_note.push_str("・価格なし銘柄を除く");
    }
    let pnl_text = format_signed_valuation_amount(valuation.profit_loss.amount);
    let pnl_note = format_signed_valuation_rate(valuation.profit_loss.rate, 1);
    let pnl_negative = is_negative_valuation(valuation.profit_loss.amount);

    let items = Signal::derive(move || {
        let kpi = kpi.get();
        vec![
            SummaryItem::new("評価額合計", Some(market_text.clone())),
            SummaryItem {
                label: "含み損益",
                value: Some(pnl_text.clone()),
                note: Some(pnl_note.clone()),
                negative: pnl_negative,
            },
            SummaryItem::new(
                "年間配当参考額",
                Some(
                    kpi.total_annual_dividends
                        .map_or("—".to_string(), format_currency),
                ),
            ),
            SummaryItem::new(
                "取得額基準利回り",
                Some(
                    kpi.dividend_yield
                        .map_or("—".to_string(), format_percentage_value),
                ),
            ),
        ]
    });

    view! {
        <section id="assetbalance-summary" class="mb-3" data-testid="asset-portfolio-summary">
            <div data-testid="portfolio-kpi-strip">
                <div data-testid="portfolio-kpi-grid">
                    <SummaryStrip items=items />
                </div>
                <p class="mt-1 text-xs font-medium text-text-muted">{market_note}</p>
            </div>
            {views_empty.then(move || {
                view! {
                    <div class="sum-card mt-3">
                        <div class="sum-card-body">
                            <EmptyState
                                title="該当する銘柄がありません"
                                description="検索条件を変更するか、絞り込みを解除してください。"
                            />
                            <div class="mt-3 text-center">
                                <Button
                                    variant=ButtonVariant::Ghost
                                    on_click=move |_| on_clear_filter()
                                >
                                    "絞り込みを解除"
                                </Button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </section>
    }
    .into_any()
}

/// 取得額構成比の帯グラフと凡例。一覧の後ろに置き、初期は折りたたむ。
#[component]
pub(crate) fn PortfolioComposition(
    items: Vec<ChartItem>,
    open: RwSignal<bool>,
    show_all: RwSignal<bool>,
) -> impl IntoView {
    // 帯・凡例は構成比を持つ行だけ。取得額0の行はカードには出すが帯には置かない
    let composition_items: Vec<ChartItem> = items
        .iter()
        .filter(|item| item.percentage.is_some())
        .cloned()
        .collect();
    if composition_items.is_empty() {
        return ().into_any();
    }
    let composition_percentages: Vec<Option<f64>> = composition_items
        .iter()
        .map(|item| item.percentage)
        .collect();
    let composition_total = composition_items.len();
    view! {
        <section class="sum-card mt-3" data-testid="portfolio-composition">
            <DisclosureToggle
                style=DisclosureStyle::HeaderFlat
                expanded=Signal::derive(move || open.get())
                controls="assetbalance-composition-body".to_string()
                testid="portfolio-composition-toggle"
                hint=true
                on_toggle=move || open.update(|value| *value = !*value)
            >
                <strong>"取得額構成比"</strong>
            </DisclosureToggle>
            <div id="assetbalance-composition-body" hidden=move || !open.get()>
                <div class="mt-2.5">
                {move || {
                    let display = chart_display(
                        composition_total,
                        &composition_percentages,
                        show_all.get(),
                    );
                    view! {
                        <div
                            class="sum-comp-bar"
                            aria-hidden="true"
                        >
                            {composition_items
                                .iter()
                                .take(display.visible_count)
                                .enumerate()
                                .filter_map(|(index, item)| {
                                    item.percentage.map(|percentage| {
                                        view! {
                                            <div
                                                class=format!("sum-comp-segment {}", holding_bar_class(index))
                                                style=format!("width: {}%", percentage.min(100.0))
                                            />
                                        }
                                    })
                                })
                                .collect_view()}
                            {display.others.as_ref().map(|others| {
                                view! {
                                    <div
                                        class="sum-comp-segment sum-comp-others"
                                        style=format!("width: {}%", others.percentage.min(100.0))
                                    />
                                }
                            })}
                        </div>
                        <ul class="sum-comp-legend">
                            {composition_items
                                .iter()
                                .take(display.visible_count)
                                .enumerate()
                                .map(|(index, item)| {
                                    view! {
                                        <li class="sum-comp-legend-item">
                                            <span
                                                class=format!("sum-comp-legend-dot {}", holding_bar_class(index))
                                                aria-hidden="true"
                                            />
                                            <span class="sum-comp-legend-name">{item.view.name.clone()}</span>
                                            <span class="sum-comp-legend-pct">
                                                {item.percentage.map_or("—".to_string(), |percentage| {
                                                    format_fixed_percent(percentage, 1)
                                                })}
                                            </span>
                                        </li>
                                    }
                                })
                                .collect_view()}
                            {display.others.as_ref().map(|others| {
                                view! {
                                    <li class="sum-comp-legend-item">
                                        <span class="sum-comp-legend-dot sum-comp-legend-dot-others" aria-hidden="true" />
                                        <span class="sum-comp-legend-name">
                                            {format!("その他 {}銘柄", others.count)}
                                        </span>
                                        <span class="sum-comp-legend-pct">
                                            {format_fixed_percent(others.percentage, 1)}
                                        </span>
                                    </li>
                                }
                            })}
                        </ul>
                    }
                }}
                </div>
            </div>
        </section>
    }
    .into_any()
}
