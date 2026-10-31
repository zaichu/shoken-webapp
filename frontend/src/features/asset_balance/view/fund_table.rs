use super::day_change::{DAY_CHANGE_COLUMN_NOTE, day_change_cell, pending_day_change_cell};
use super::sort_header::sort_header;
use crate::features::asset_balance::format::{
    format_row_valuation_amount, format_signed_row_valuation_amount, format_signed_valuation_rate,
    is_negative_valuation, is_positive_valuation,
};
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::portfolio::SortOrder;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::format::{format_number_value, format_unit_price, unit_price_decimals};
use crate::ui::labels::{
    ACCOUNT, BASE_PRICE, DAY_CHANGE, DISTRIBUTION, FUND, FUND_SHARES, MARKET_VALUE,
    VALUATION_PROFIT_LOSS,
};
use crate::ui::security_link::TableNameLink;
use leptos::prelude::*;

#[component]
pub fn FundHoldingsTable(
    views: Vec<FundView>,
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    // 列の中の小数桁をそろえる(整数円の値しかない列は .00 を付けない)
    let base_price_decimals = unit_price_decimals(views.iter().map(|view| view.base_price));
    view! {
        <div data-testid="asset-fund-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col" class="col-text">{FUND}</th>
                        <th scope="col" class="col-text">{ACCOUNT}</th>
                        <th scope="col" class="col-text">{DISTRIBUTION}</th>
                        {sort_header(FUND_SHARES, FundSortKey::Shares, sort_key, sort_order)}
                        {sort_header(BASE_PRICE, FundSortKey::BasePrice, sort_key, sort_order)}
                        <th
                            scope="col"
                            class="col-num"
                            title=DAY_CHANGE_COLUMN_NOTE
                            abbr=DAY_CHANGE_COLUMN_NOTE
                        >
                            {DAY_CHANGE}
                        </th>
                        {sort_header(MARKET_VALUE, FundSortKey::MarketValue, sort_key, sort_order)}
                        {sort_header(
                            VALUATION_PROFIT_LOSS,
                            FundSortKey::ProfitLoss,
                            sort_key,
                            sort_order,
                        )}
                    </tr>
                </thead>
                <tbody>
                    {views
                        .into_iter()
                        .map(|view| {
                            let pnl_text = format_signed_row_valuation_amount(Some(
                                view.profit_loss,
                            ));
                            let pnl_rate_text =
                                format_signed_valuation_rate(view.profit_loss_rate, 1);
                            let pnl_negative =
                                is_negative_valuation(Some(view.profit_loss));
                            let pnl_positive =
                                is_positive_valuation(Some(view.profit_loss));
                            let rate_negative =
                                is_negative_valuation(view.profit_loss_rate);
                            let rate_positive =
                                is_positive_valuation(view.profit_loss_rate);
                            let href = view
                                .id
                                .clone()
                                .map(|id| format!("/assetbalance/funds/{id}"));
                            let shares_text = format!(
                                "{}口",
                                format_number_value(view.shares)
                            );
                            let base_price_text = format_unit_price(
                                view.base_price,
                                base_price_decimals,
                            );
                            view! {
                                <tr>
                                    <td class="asset-name-cell col-text" title=view.name.clone()>
                                        <TableNameLink href=href name=view.name.clone() />
                                    </td>
                                    <td class="col-text">
                                        <AccountBadge label=view.account.clone() />
                                    </td>
                                    <td class="col-text text-text-soft">
                                        {view.course.clone().unwrap_or_else(|| "—".to_string())}
                                    </td>
                                    <td class="col-num text-text-soft">
                                        <Amount kind=AmountKind::Plain text=shares_text />
                                    </td>
                                    <td class="col-num text-text-soft">
                                        <Amount kind=AmountKind::Plain text=base_price_text />
                                    </td>
                                    <td
                                        class="col-num"
                                        title=view.day_change_title()
                                    >
                                        {if view.nav_pending() {
                                            pending_day_change_cell().into_any()
                                        } else {
                                            day_change_cell(view.view_day_change(), None)
                                                .into_any()
                                        }}
                                    </td>
                                    <td class="col-num font-bold text-text-deep">
                                        <Amount
                                            kind=AmountKind::Plain
                                            text=format_row_valuation_amount(
                                                Some(view.market_value),
                                            )
                                        />
                                    </td>
                                    <td class="col-num">
                                        <Amount
                                            kind=AmountKind::Delta
                                            text=pnl_text
                                            negative=pnl_negative
                                            positive=pnl_positive
                                            class="font-bold"
                                        />
                                        <Amount
                                            kind=AmountKind::Delta
                                            text=pnl_rate_text
                                            negative=rate_negative
                                            positive=rate_positive
                                            class="ml-1 text-xs"
                                        />
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}

/// スマホのファンド一覧。1行は「ファンド名 + 評価額」と「口座・口数・基準価額 + 損益」
#[component]
pub fn FundHoldingsList(views: Vec<FundView>) -> impl IntoView {
    // 列の中の小数桁をそろえる(整数円の値しかない列は .00 を付けない)
    let base_price_decimals = unit_price_decimals(views.iter().map(|view| view.base_price));
    view! {
        <ul class="asset-compact-list" data-testid="asset-fund-list">
            {views
                .into_iter()
                .map(|view| {
                    let pnl_text = format_signed_row_valuation_amount(Some(view.profit_loss));
                    let pnl_rate_text =
                        format_signed_valuation_rate(view.profit_loss_rate, 1);
                    let pnl_negative = is_negative_valuation(Some(view.profit_loss));
                    let pnl_positive = is_positive_valuation(Some(view.profit_loss));
                    let rate_negative = is_negative_valuation(view.profit_loss_rate);
                    let rate_positive = is_positive_valuation(view.profit_loss_rate);
                    let shares_text =
                        format!("{}口", format_number_value(view.shares));
                    let base_price_text =
                        format_unit_price(view.base_price, base_price_decimals);
                    let inner = view! {
                        <>
                            <div class="flex items-baseline justify-between gap-2">
                                <span class="min-w-0 truncate font-medium text-text">
                                    {view.name.clone()}
                                </span>
                                <Amount
                                    kind=AmountKind::Plain
                                    text=format_row_valuation_amount(Some(view.market_value))
                                    class="shrink-0 font-bold text-text-deep"
                                />
                            </div>
                            <div class="mt-0.5 flex items-start justify-between gap-2 text-xs">
                                <span class="min-w-0 flex-1 break-words text-text-muted">
                                    <AccountBadge label=view.account.clone() />
                                    " ・ "
                                    <Amount kind=AmountKind::Plain text=shares_text />
                                    " ・ 基準価額 "
                                    <Amount kind=AmountKind::Plain text=base_price_text />
                                </span>
                                <span class="flex shrink-0 items-baseline whitespace-nowrap">
                                    <Amount
                                        kind=AmountKind::Delta
                                        text=pnl_text
                                        negative=pnl_negative
                                        positive=pnl_positive
                                        class="font-bold"
                                    />
                                    <Amount
                                        kind=AmountKind::Delta
                                        text=pnl_rate_text
                                        negative=rate_negative
                                        positive=rate_positive
                                        class="ml-1 text-xs"
                                    />
                                </span>
                            </div>
                        </>
                    };
                    view! {
                        <li class="asset-compact-row" data-testid="asset-compact-row">
                            {match &view.id {
                                Some(id) => view! {
                                    <a
                                        href=format!("/assetbalance/funds/{id}")
                                        class="asset-compact-link"
                                    >
                                        {inner}
                                    </a>
                                }
                                    .into_any(),
                                None => view! { <div class="asset-compact-link">{inner}</div> }
                                    .into_any(),
                            }}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}
