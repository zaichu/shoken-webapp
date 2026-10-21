use super::day_change::{DAY_CHANGE_COLUMN_NOTE, day_change_cell, pending_day_change_cell};
use super::sort_header::sort_header;
use crate::features::asset_balance::format::{
    format_currency, format_row_valuation_amount, pnl_cells,
};
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::model::format_number_value;
use crate::features::asset_balance::portfolio::SortOrder;
use crate::ui::badge::AccountBadge;
use crate::ui::security_link::TableNameLink;
use leptos::prelude::*;

#[component]
pub fn FundHoldingsTable(
    views: Vec<FundView>,
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    view! {
        <div data-testid="asset-fund-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col" class="col-text">"ファンド"</th>
                        <th scope="col" class="col-text">"口座"</th>
                        <th scope="col" class="col-text">"分配金"</th>
                        {sort_header("保有口数", FundSortKey::Shares, sort_key, sort_order)}
                        {sort_header("基準価額", FundSortKey::BasePrice, sort_key, sort_order)}
                        <th
                            scope="col"
                            class="col-num"
                            title=DAY_CHANGE_COLUMN_NOTE
                            abbr=DAY_CHANGE_COLUMN_NOTE
                        >
                            "前日比"
                        </th>
                        {sort_header("評価額", FundSortKey::MarketValue, sort_key, sort_order)}
                        {sort_header("評価損益", FundSortKey::ProfitLoss, sort_key, sort_order)}
                    </tr>
                </thead>
                <tbody>
                    {views
                        .into_iter()
                        .map(|view| {
                            let (pnl_text, pnl_rate_text, pnl_text_class, pnl_badge_class) =
                                pnl_cells(Some(view.profit_loss), view.profit_loss_rate);
                            let href = view
                                .id
                                .clone()
                                .map(|id| format!("/assetbalance/funds/{id}"));
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
                                        {format!("{}口", format_number_value(view.shares))}
                                    </td>
                                    <td class="col-num text-text-soft">
                                        {format_currency(view.base_price)}
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
                                        {format_row_valuation_amount(Some(view.market_value))}
                                    </td>
                                    <td class="col-num">
                                        <span class=format!("font-bold tabular-nums {pnl_text_class}")>
                                            {pnl_text}
                                        </span>
                                        <span class=pnl_badge_class>
                                            {pnl_rate_text}
                                        </span>
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
    view! {
        <ul class="asset-compact-list" data-testid="asset-fund-list">
            {views
                .into_iter()
                .map(|view| {
                    let (pnl_text, pnl_rate_text, pnl_text_class, pnl_badge_class) =
                        pnl_cells(Some(view.profit_loss), view.profit_loss_rate);
                    let meta = format!(
                        "{}口 ・ 基準価額 {}",
                        format_number_value(view.shares),
                        format_currency(view.base_price),
                    );
                    let inner = view! {
                        <>
                            <div class="flex items-baseline justify-between gap-2">
                                <span class="min-w-0 truncate font-medium text-text">
                                    {view.name.clone()}
                                </span>
                                <span class="shrink-0 font-bold tabular-nums text-text-deep">
                                    {format_row_valuation_amount(Some(view.market_value))}
                                </span>
                            </div>
                            <div class="mt-0.5 flex items-baseline justify-between gap-2 text-xs">
                                <span class="min-w-0 truncate text-text-muted">
                                    <AccountBadge label=view.account.clone() />
                                    " ・ " {meta}
                                </span>
                                <span class="flex shrink-0 items-baseline">
                                    <span class=format!("font-bold tabular-nums {pnl_text_class}")>
                                        {pnl_text}
                                    </span>
                                    <span class=pnl_badge_class>
                                        {pnl_rate_text}
                                    </span>
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
