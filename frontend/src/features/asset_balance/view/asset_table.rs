use crate::features::asset_balance::format::{
    format_currency, format_price_date, format_signed_valuation_amount,
    format_signed_valuation_rate, format_valuation_amount, is_negative_valuation,
    is_positive_valuation,
};
use crate::features::asset_balance::holdings::{
    HoldingView, format_dividend_annual, format_dividend_yield, holding_dividend,
};
use crate::features::asset_balance::model::{
    ValuationInput, format_number_value, holding_valuation,
};
use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::button::{Button, ButtonVariant};
use leptos::prelude::*;

fn header_label(order: SortOrder, active: bool) -> String {
    if !active {
        return String::new();
    }
    match order {
        SortOrder::Desc => " ▼".to_string(),
        SortOrder::Asc => " ▲".to_string(),
    }
}

#[component]
fn SortHeader(
    label: &'static str,
    key: SortKey,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let active = move || sort_key.get() == key;
    let arrow = move || header_label(sort_order.get(), active());
    let on_click = move |_| {
        if sort_key.get_untracked() == key {
            sort_order.update(|order| {
                *order = match *order {
                    SortOrder::Desc => SortOrder::Asc,
                    SortOrder::Asc => SortOrder::Desc,
                }
            });
        } else {
            sort_key.set(key);
            sort_order.set(SortOrder::Desc);
        }
    };
    view! {
        <th
            scope="col"
            class="text-right"
            aria-sort=move || {
                active().then(|| {
                    match sort_order.get() {
                        SortOrder::Desc => "descending",
                        SortOrder::Asc => "ascending",
                    }
                })
            }
        >
            <Button
                variant=ButtonVariant::AssetSortColumn
                aria_pressed=Signal::derive(move || active().to_string())
                on_click=on_click
            >
                {label}
                <span class="asset-sort-arrow">{arrow}</span>
            </Button>
        </th>
    }
}

#[component]
pub fn AssetHoldingsTable(
    views: Vec<HoldingView>,
    dividends: RwSignal<DividendMaps>,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    view! {
        <div data-testid="asset-holdings-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col">"銘柄"</th>
                        <th scope="col" class="text-right">"数量"</th>
                        <th scope="col" class="text-right">"取得単価"</th>
                        <th scope="col" class="text-right">"現在値"</th>
                        <SortHeader label="評価額" key=SortKey::MarketValue sort_key=sort_key sort_order=sort_order />
                        <SortHeader label="含み損益" key=SortKey::ProfitLoss sort_key=sort_key sort_order=sort_order />
                        <SortHeader label="年間配当" key=SortKey::Dividend sort_key=sort_key sort_order=sort_order />
                        <SortHeader label="取得額利回り" key=SortKey::Yield sort_key=sort_key sort_order=sort_order />
                    </tr>
                </thead>
                <tbody>
                    {views
                        .into_iter()
                        .map(|view| {
                            let valuation = holding_valuation(&ValuationInput::from(&view));
                            let market_text = format_valuation_amount(valuation.market_value);
                            let pnl_text = format_signed_valuation_amount(
                                valuation.profit_loss.amount,
                            );
                            let pnl_rate_text = format_signed_valuation_rate(
                                valuation.profit_loss.rate,
                                1,
                            );
                            let pnl_negative = is_negative_valuation(
                                valuation.profit_loss.amount,
                            );
                            let (pnl_text_class, pnl_badge_class) = if pnl_negative {
                                ("text-negative", "asset-badge asset-badge-loss")
                            } else if is_positive_valuation(valuation.profit_loss.amount) {
                                ("text-positive", "asset-badge asset-badge-gain")
                            } else {
                                ("text-text-soft", "asset-badge")
                            };
                            let price_date_text =
                                view.price_as_of.as_deref().map(format_price_date);
                            let dividend = holding_dividend(
                                &view.code,
                                view.shares,
                                view.average_price,
                                &dividends.get(),
                            );
                            let dividend_annual = format_dividend_annual(&dividend);
                            let dividend_yield = format_dividend_yield(&dividend);
                            let href = view
                                .id
                                .clone()
                                .map(|id| format!("/assetbalance/holdings/{id}"));
                            let name_cell = view! {
                                <>
                                    <span class="mr-2 text-xs text-text-muted tabular-nums">
                                        {view.code.clone()}
                                    </span>
                                    {view.name.clone()}
                                </>
                            };
                            view! {
                                <tr>
                                    <td>
                                        {match href {
                                            Some(href) => view! {
                                                <a
                                                    href=href
                                                    class="inline-flex min-h-6 items-center no-underline hover:underline"
                                                >
                                                    {name_cell}
                                                </a>
                                            }
                                                .into_any(),
                                            None => name_cell.into_any(),
                                        }}
                                    </td>
                                    <td class="text-right tabular-nums text-text-soft">
                                        {format!("{}株", format_number_value(view.shares))}
                                    </td>
                                    <td class="text-right tabular-nums text-text-soft">
                                        {format_currency(view.average_price)}
                                    </td>
                                    <td class="text-right tabular-nums text-text-soft">
                                        {format_currency(view.current_price)}
                                        {price_date_text.map(|date| {
                                            view! {
                                                <div class="asset-price-date">{date}</div>
                                            }
                                        })
                                    }
                                    </td>
                                    <td class="text-right font-bold tabular-nums text-brand-text">{market_text}</td>
                                    <td class="text-right tabular-nums">
                                        {if valuation.profit_loss.amount.is_some() {
                                            let rate_badge = valuation.profit_loss.rate.is_some().then(|| {
                                                view! {
                                                    " "
                                                    <span class=pnl_badge_class>
                                                        {pnl_rate_text.clone()}
                                                    </span>
                                                }
                                            });
                                            view! {
                                                <span class=format!("font-bold tabular-nums {pnl_text_class}")>
                                                    {pnl_text}
                                                </span>
                                                {rate_badge}
                                            }
                                                .into_any()
                                        } else {
                                            view! { "—" }.into_any()
                                        }}
                                    </td>
                                    <td class="text-right tabular-nums">{dividend_annual}</td>
                                    <td class="text-right tabular-nums text-brand-text">{dividend_yield}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}
