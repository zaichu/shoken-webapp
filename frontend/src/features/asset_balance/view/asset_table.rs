use super::day_change::day_change_cell;
use super::sort_header::sort_header;
use crate::features::asset_balance::format::{
    format_row_valuation_amount, pnl_cells, price_date_outliers,
};
use crate::features::asset_balance::holdings::{
    HoldingView, format_dividend_annual, format_dividend_per_share, format_dividend_yield,
    holding_dividend,
};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::badge::AccountBadge;
use crate::ui::format::{format_number_value, format_unit_price, unit_price_decimals};
use crate::ui::labels::{
    ACCOUNT, ACQUISITION_YIELD, ANNUAL_DIVIDEND, AVERAGE_PRICE, CODE_COLUMN, CURRENT_PRICE,
    DAY_CHANGE, MARKET_VALUE, NAME_COLUMN, PER_SHARE_DIVIDEND, QUANTITY, VALUATION_PROFIT_LOSS,
};
use crate::ui::security_link::{TableNameLink, TableSecurityCode};
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
    // 列の中の小数桁をそろえる(整数円の値しかない列は .00 を付けない)
    let average_price_decimals = unit_price_decimals(views.iter().map(|view| view.average_price));
    let current_price_decimals = unit_price_decimals(views.iter().map(|view| view.current_price));
    let row_dividends: Vec<_> = views
        .iter()
        .map(|view| {
            holding_dividend(
                &view.code,
                view.shares,
                view.average_price,
                &dividends.get(),
            )
        })
        .collect();
    let dividend_decimals = unit_price_decimals(
        row_dividends
            .iter()
            .filter_map(|dividend| dividend.per_share),
    );
    view! {
        <div data-testid="asset-holdings-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col" class="col-text">{CODE_COLUMN}</th>
                        <th scope="col" class="col-text">{NAME_COLUMN}</th>
                        <th scope="col" class="col-text">{ACCOUNT}</th>
                        <th scope="col" class="col-num">{QUANTITY}</th>
                        <th scope="col" class="col-num hidden xl:table-cell print:table-cell">{AVERAGE_PRICE}</th>
                        <th scope="col" class="col-num">{CURRENT_PRICE}</th>
                        <th scope="col" class="col-num">{DAY_CHANGE}</th>
                        {sort_header(MARKET_VALUE, SortKey::MarketValue, sort_key, sort_order)}
                        {sort_header(
                            VALUATION_PROFIT_LOSS,
                            SortKey::ProfitLoss,
                            sort_key,
                            sort_order,
                        )}
                        <th scope="col" class="col-num hidden xl:table-cell print:table-cell">{PER_SHARE_DIVIDEND}</th>
                        {sort_header(ANNUAL_DIVIDEND, SortKey::Dividend, sort_key, sort_order)}
                        {sort_header(
                            ACQUISITION_YIELD,
                            SortKey::Yield,
                            sort_key,
                            sort_order,
                        )}
                    </tr>
                </thead>
                <tbody>
                    {views
                        .into_iter()
                        .zip(date_outliers)
                        .zip(row_dividends)
                        .map(|((view, is_outlier), dividend)| {
                            let valuation = holding_valuation(&ValuationInput::from(&view));
                            let market_text = format_row_valuation_amount(valuation.market_value);
                            let (pnl_text, pnl_rate_text, pnl_text_class, pnl_badge_class) =
                                pnl_cells(valuation.profit_loss.amount, valuation.profit_loss.rate);
                            let dividend_annual = format_dividend_annual(&dividend);
                            let dividend_per_share =
                                format_dividend_per_share(&dividend, dividend_decimals);
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
                            view! {
                                <tr>
                                    <td class="col-text">
                                        <TableSecurityCode code=view.code.clone() />
                                    </td>
                                    <td class="asset-name-cell asset-name-cell-narrow col-text" title=view.name.clone()>
                                        <TableNameLink href=href name=view.name.clone() />
                                    </td>
                                    <td class="col-text">
                                        {view.account.clone().map_or_else(
                                            || view! { "—" }.into_any(),
                                            |account| {
                                                view! { <AccountBadge label=account /> }.into_any()
                                            },
                                        )}
                                    </td>
                                    <td class="col-num text-text-soft">
                                        {format!("{}株", format_number_value(view.shares))}
                                    </td>
                                    <td class="col-num text-text-soft hidden xl:table-cell print:table-cell">
                                        {format_unit_price(view.average_price, average_price_decimals)}
                                    </td>
                                    <td class="col-num text-text-soft">
                                        {format_unit_price(view.current_price, current_price_decimals)}
                                    </td>
                                    <td class="col-num">
                                        {day_change_cell(view.day_change, outlier_date)}
                                    </td>
                                    <td class="col-num font-bold text-text-deep">{market_text}</td>
                                    <td class="col-num">
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
                                    <td class="col-num hidden xl:table-cell print:table-cell">{dividend_per_share}</td>
                                    <td class="col-num">{dividend_annual}</td>
                                    <td class="col-num text-text-deep">{dividend_yield}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}
