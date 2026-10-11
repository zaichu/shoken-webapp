use super::palette::holding_bar_class;
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::format::{
    dec_to_f64, format_currency, format_fixed_percent, format_percentage_value,
    format_signed_valuation_amount, format_signed_valuation_rate, format_valuation_amount,
    is_negative_valuation,
};
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model::{
    KpiHolding, ValuationInput, calculate_portfolio_kpi, portfolio_valuation, summary_valuation,
    total_purchase_amount,
};
use crate::features::asset_balance::portfolio::chart_display;
use crate::features::asset_balance::store::FilteredPortfolio;
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::Amount;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use crate::ui::empty_state::EmptyState;
use leptos::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct ChartItem {
    pub(crate) view: HoldingView,
    pub(crate) percentage: Option<f64>,
}

#[component]
fn Kpi(
    label: &'static str,
    #[prop(into)] text: Signal<String>,
    #[prop(into, optional)] note: Signal<String>,
    #[prop(into, optional)] negative: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="kpi">
            <dt>{label}</dt>
            <dd class="kpi-value">
                <Amount block=true text=text class="kpi-amount" negative=negative />
                {move || {
                    let note = note.get();
                    (!note.is_empty()).then(|| view! { <small>{note}</small> })
                }}
            </dd>
        </div>
    }
}

#[component]
pub(crate) fn PortfolioSummary(
    filtered: FilteredPortfolio,
    fallback_totals: Option<AssetBalanceSummary>,
    total_count: usize,
    is_filtered: bool,
    on_clear_filter: impl Fn() + Send + Sync + 'static,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
    preview: bool,
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
        .map(|summary| summary_valuation(summary, row_valuation.has_unpriced))
        .unwrap_or(row_valuation);
    let total_purchase = fallback_totals
        .as_ref()
        .map(|summary| format_currency(dec_to_f64(&summary.total_purchase_amount)))
        .unwrap_or_else(|| format_currency(total_purchase_amount(&kpi_holdings)));
    let kpi =
        Memo::new(move |_| calculate_portfolio_kpi(&kpi_holdings, &dividends.get().per_share));

    let market_text = format_valuation_amount(valuation.market_value);
    let mut market_note = if preview {
        "CSV取込時点の価格で計算".to_string()
    } else {
        "現在値の基準日は銘柄ごとに異なります".to_string()
    };
    if valuation.has_unpriced {
        market_note.push_str("・価格なし銘柄を除く");
    }
    let pnl_text = format_signed_valuation_amount(valuation.profit_loss.amount);
    let pnl_note = format_signed_valuation_rate(valuation.profit_loss.rate, 1);
    let pnl_negative = is_negative_valuation(valuation.profit_loss.amount);

    let total_annual_dividends = Signal::derive(move || {
        kpi.with(|kpi| {
            kpi.total_annual_dividends
                .map_or("—".to_string(), format_currency)
        })
    });
    let dividend_yield = Signal::derive(move || {
        kpi.with(|kpi| {
            kpi.dividend_yield
                .map_or("—".to_string(), format_percentage_value)
        })
    });
    let display_count = views.len();
    let display_count_str = move || {
        if is_filtered {
            format!("{display_count} / {total_count}")
        } else {
            display_count.to_string()
        }
    };

    view! {
        <section id="assetbalance-summary" class="sum-card mb-3" data-testid="asset-portfolio-summary">
            <header class="sum-header">
                <strong>"集計情報"</strong>
                {preview.then(|| {
                    view! {
                        <span class="rounded border border-border-subtle px-1 py-0.5 text-xs font-medium text-text-subtle">
                            "CSVプレビュー"
                        </span>
                    }
                })}
            </header>
            <div data-testid="portfolio-kpi-strip">
            <dl class="kpis" data-testid="portfolio-kpi-grid">
                <Kpi label="評価額合計" text=market_text note=market_note />
                <Kpi label="含み損益" text=pnl_text note=pnl_note negative=pnl_negative />
                <Kpi label="年間配当金額" text=total_annual_dividends />
                <Kpi
                    label="配当利回り（年間）"
                    text=dividend_yield
                    note="年間配当金額 ÷ 合計取得総額".to_string()
                />
                <Kpi label="合計取得総額" text=total_purchase />
                <div class="kpi">
                    <dt>"保有銘柄数"</dt>
                    <dd class="kpi-value kpi-count">
                        <span class="tabular-nums">{display_count_str}</span>
                        <span class="kpi-unit">"銘柄"</span>
                    </dd>
                </div>
            </dl>
            </div>
            {views_empty.then(move || {
                view! {
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
