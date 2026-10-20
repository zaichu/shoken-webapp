use super::day_change::day_change_cell;
use super::sort_header::sort_header;
use crate::features::asset_balance::format::{
    format_currency, format_row_valuation_amount, pnl_cells, price_date_outliers,
};
use crate::features::asset_balance::holdings::{
    HoldingView, format_dividend_annual, format_dividend_yield, holding_dividend,
};
use crate::features::asset_balance::model::{
    ValuationInput, format_number_value, holding_valuation,
};
use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::features::dividend_per_share::DividendMaps;
use leptos::prelude::*;

#[component]
pub fn AssetHoldingsTable(
    views: Vec<HoldingView>,
    /// 見出しに出している代表の基準日("YYYY-MM-DD")。違う行だけ日付を出す
    price_date_rep: Option<String>,
    dividends: RwSignal<DividendMaps>,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let date_outliers = price_date_outliers(
        &views
            .iter()
            .map(|view| view.price_as_of.clone())
            .collect::<Vec<_>>(),
        price_date_rep.as_deref(),
    );
    view! {
        <div data-testid="asset-holdings-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col">"銘柄"</th>
                        <th scope="col" class="text-right">"数量"</th>
                        <th scope="col" class="text-right">"取得単価"</th>
                        <th scope="col" class="text-right">"現在値"</th>
                        <th scope="col" class="text-right">"前日比"</th>
                        {sort_header("評価額", SortKey::MarketValue, sort_key, sort_order)}
                        {sort_header("含み損益", SortKey::ProfitLoss, sort_key, sort_order)}
                        {sort_header("年間配当", SortKey::Dividend, sort_key, sort_order)}
                        {sort_header("取得額利回り", SortKey::Yield, sort_key, sort_order)}
                    </tr>
                </thead>
                <tbody>
                    {views
                        .into_iter()
                        .zip(date_outliers)
                        .map(|(view, is_outlier)| {
                            let valuation = holding_valuation(&ValuationInput::from(&view));
                            let market_text = format_row_valuation_amount(valuation.market_value);
                            let (pnl_text, pnl_rate_text, pnl_text_class, pnl_badge_class) =
                                pnl_cells(valuation.profit_loss.amount, valuation.profit_loss.rate);
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
                            let outlier_date = if is_outlier {
                                view.price_as_of.clone()
                            } else {
                                None
                            };
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
                                    <td title=view.name.clone()>
                                        {match href {
                                            Some(href) => view! {
                                                <a
                                                    href=href
                                                    class="flex min-h-6 items-center no-underline hover:underline"
                                                >
                                                    <span class="min-w-0 truncate">
                                                        {name_cell}
                                                    </span>
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
                                    </td>
                                    <td class="text-right tabular-nums">
                                        {day_change_cell(view.day_change, outlier_date)}
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
