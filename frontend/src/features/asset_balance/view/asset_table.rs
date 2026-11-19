use super::day_change::day_change_cell;
use super::sort_header::{sort_header, sort_header_titled};
use crate::features::asset_balance::format as balance_format;
use crate::features::asset_balance::holdings;
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model;
use crate::features::asset_balance::model::ValuationInput;
use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::asset_balance::{AssetMarker, AssetRateAmount};
use crate::ui::badge::{AccountBadge, AccountLegend};
use crate::ui::format;
use crate::ui::labels;
use crate::ui::security_link::{TableNameLink, TableSecurityCode};
use leptos::prelude::*;

#[component]
pub fn AssetHoldingsTable(
    views: Vec<HoldingView>,
    /// 見出しに出している代表の基準日("YYYY-MM-DD")。違う行だけ日付を出す
    #[expect(
        clippy::needless_pass_by_value,
        reason = "Leptos component props are owned by the view"
    )]
    price_date_rep: Option<String>,
    dividends: RwSignal<DividendMaps>,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let price_date_rep = price_date_rep.clone();
    let date_outliers = balance_format::price_date_outliers(
        &views
            .iter()
            .map(|view| view.price_as_of.clone())
            .collect::<Vec<_>>(),
        price_date_rep.as_deref(),
    );
    // 列の中の小数桁をそろえる(整数円の値しかない列は .00 を付けない)
    let average_price_decimals =
        format::unit_price_decimals(views.iter().map(|view| view.average_price));
    let current_price_decimals =
        format::unit_price_decimals(views.iter().map(|view| view.current_price));
    let row_dividends: Vec<_> = views
        .iter()
        .map(|view| {
            holdings::holding_dividend(
                &view.code,
                view.shares,
                view.average_price,
                &dividends.get(),
            )
        })
        .collect();
    let dividend_decimals = format::unit_price_decimals(
        row_dividends
            .iter()
            .filter_map(|dividend| dividend.per_share),
    );
    view! {
        <div data-testid="asset-holdings-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col" class="col-text">{labels::CODE_COLUMN}</th>
                        <th scope="col" class="col-text">{labels::NAME_COLUMN}</th>
                        <th scope="col" class="col-text">{labels::ACCOUNT}</th>
                        <th scope="col" class="col-num">{labels::QUANTITY}</th>
                        <th scope="col" class="col-num hidden xl:table-cell print:table-cell">{labels::AVERAGE_PRICE}</th>
                        <th scope="col" class="col-num">{labels::CURRENT_PRICE}</th>
                        // 1行にすると1280で表がはみ出すため単位だけ改行する。
                        // 読み上げ名は変わらない(「前日比 (1株あたり)」)
                        <th scope="col" class="col-num">
                            {labels::DAY_CHANGE}
                            <br />
                            {labels::PER_SHARE_UNIT}
                        </th>
                        {sort_header(labels::MARKET_VALUE, SortKey::MarketValue, sort_key, sort_order)}
                        {sort_header(
                            labels::VALUATION_PROFIT_LOSS,
                            SortKey::ProfitLoss,
                            sort_key,
                            sort_order,
                        )}
                        <th scope="col" class="col-num hidden xl:table-cell print:table-cell">{labels::PER_SHARE_DIVIDEND}</th>
                        {sort_header_titled(
                            labels::ANNUAL_DIVIDEND_HEAD,
                            Some(labels::ANNUAL_DIVIDEND_HEAD_SUB),
                            Some(labels::ANNUAL_DIVIDEND_NOTE),
                            SortKey::Dividend,
                            sort_key,
                            sort_order,
                        )}
                        {sort_header_titled(
                            labels::ACQUISITION_YIELD,
                            None,
                            Some(labels::ACQUISITION_YIELD_NOTE),
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
                            let valuation = model::holding_valuation(&ValuationInput::from(&view));
                            let market_text = balance_format::format_row_valuation_amount(valuation.market_value);
                            let pnl_text = balance_format::format_signed_row_valuation_amount(
                                valuation.profit_loss.amount,
                            );
                            let pnl_rate_text = balance_format::format_signed_valuation_rate(
                                valuation.profit_loss.rate,
                                1,
                            );
                            let pnl_negative =
                                balance_format::is_negative_valuation(valuation.profit_loss.amount);
                            let pnl_positive =
                                balance_format::is_positive_valuation(valuation.profit_loss.amount);
                            let rate_negative = balance_format::is_negative_rate(valuation.profit_loss.rate);
                            let rate_positive = balance_format::is_positive_rate(valuation.profit_loss.rate);
                            let dividend_annual = holdings::format_dividend_annual(&dividend);
                            let dividend_per_share =
                                holdings::format_dividend_per_share(&dividend, dividend_decimals);
                            let dividend_yield = holdings::format_dividend_yield(&dividend);
                            let dividend_yield_negative = balance_format::is_negative_rate(dividend.yield_value);
                            let dividend_yield_positive = balance_format::is_positive_rate(dividend.yield_value);
                            let href = Some(holdings::holding_stock_href(&view.code));
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
                                            || view! { <AssetMarker text=labels::unavailable_text(labels::UnavailableKind::Unassigned).to_string() soft=true /> }.into_any(),
                                            |account| {
                                                view! { <AccountBadge label=account /> }.into_any()
                                            },
                                        )}
                                    </td>
                                    <td class="col-num text-text-soft">
                                        {labels::stock_shares_label(
                                            &format::format_number_value(view.shares),
                                        )}
                                    </td>
                                    <td class="col-num text-text-soft hidden xl:table-cell print:table-cell">
                                        <Amount text=format::format_unit_price(view.average_price, average_price_decimals) />
                                    </td>
                                    <td class="col-num text-text-soft">
                                        <Amount text=format::format_unit_price(view.current_price, current_price_decimals) />
                                    </td>
                                    <td class="col-num">
                                        {day_change_cell(view.day_change, outlier_date)}
                                    </td>
                                    <td class="col-num font-bold text-text-deep"><Amount text=market_text /></td>
                                    <td class="col-num">
                                        {if valuation.profit_loss.amount.is_some() {
                                            view! {
                                                <Amount
                                                    kind=AmountKind::Delta
                                                    text=pnl_text
                                                    negative=pnl_negative
                                                    positive=pnl_positive
                                                    class="font-bold"
                                                />
                                                {valuation.profit_loss.rate.is_some().then(|| {
                                                    view! {
                                                        " "
                                                        <AssetRateAmount
                                                            text=pnl_rate_text.clone()
                                                            negative=rate_negative
                                                            positive=rate_positive
                                                        />
                                                    }
                                                })}
                                            }
                                                .into_any()
                                        } else {
                                            view! { "—" }.into_any()
                                        }}
                                    </td>
                                    <td class="col-num hidden xl:table-cell print:table-cell">
                                        {holdings::dividend_marker_text(&dividend).map_or_else(
                                            || view! {
                                                <Amount
                                                    kind=AmountKind::Delta
                                                    text=dividend_per_share.clone()
                                                    negative=balance_format::is_negative_rate(dividend.per_share)
                                                    positive=balance_format::is_positive_rate(dividend.per_share)
                                                />
                                            }.into_any(),
                                            |marker| view! { <AssetMarker text=marker.to_string() /> }.into_any(),
                                        )}
                                    </td>
                                    <td class="col-num">
                                        {holdings::dividend_marker_text(&dividend).map_or_else(
                                            || view! {
                                                <Amount
                                                    kind=AmountKind::Delta
                                                    text=dividend_annual.clone()
                                                    negative=balance_format::is_negative_valuation(dividend.annual)
                                                    positive=balance_format::is_positive_valuation(dividend.annual)
                                                />
                                            }.into_any(),
                                            |marker| view! { <AssetMarker text=marker.to_string() /> }.into_any(),
                                        )}
                                    </td>
                                    <td class="col-num">
                                        {holdings::dividend_marker_text(&dividend).map_or_else(
                                            || view! {
                                                <Amount
                                                    kind=AmountKind::Delta
                                                    text=dividend_yield.clone()
                                                    negative=dividend_yield_negative
                                                    positive=dividend_yield_positive
                                                />
                                            }.into_any(),
                                            |marker| view! { <AssetMarker text=marker.to_string() /> }.into_any(),
                                        )}
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
            <AccountLegend />
            // 算出条件は表の下に1行で添える(注記の凡例と並べる)
            <p class="text-cell text-text-muted">{labels::dividend_formula_note()}</p>
        </div>
    }
}
