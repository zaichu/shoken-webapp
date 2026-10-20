use super::day_change::day_change_cell;
use super::sort_header::sort_header;
use crate::features::asset_balance::format::{
    format_currency, format_price_date, format_row_valuation_amount, pnl_cells, price_date_outliers,
};
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::model::format_number_value;
use crate::features::asset_balance::portfolio::SortOrder;
use leptos::prelude::*;

#[component]
pub fn FundHoldingsTable(
    views: Vec<FundView>,
    /// 見出しに出している代表の基準日("YYYY-MM-DD")。違う行だけ日付を出す
    price_date_rep: Option<String>,
    sort_key: RwSignal<FundSortKey>,
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
        <div data-testid="asset-fund-table">
            <table class="asset-table">
                <thead>
                    <tr>
                        <th scope="col">"ファンド"</th>
                        <th scope="col">"口座"</th>
                        <th scope="col">"分配金"</th>
                        {sort_header("保有口数", FundSortKey::Shares, sort_key, sort_order)}
                        {sort_header("基準価額", FundSortKey::BasePrice, sort_key, sort_order)}
                        <th scope="col" class="text-right">"前日比"</th>
                        {sort_header("評価額", FundSortKey::MarketValue, sort_key, sort_order)}
                        {sort_header("評価損益", FundSortKey::ProfitLoss, sort_key, sort_order)}
                    </tr>
                </thead>
                <tbody>
                    {views
                        .into_iter()
                        .zip(date_outliers)
                        .map(|(view, is_outlier)| {
                            let (pnl_text, pnl_rate_text, pnl_text_class, pnl_badge_class) =
                                pnl_cells(Some(view.profit_loss), view.profit_loss_rate);
                            let outlier_date = if is_outlier {
                                view.price_as_of.clone()
                            } else {
                                None
                            };
                            let href = view
                                .id
                                .clone()
                                .map(|id| format!("/assetbalance/funds/{id}"));
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
                                                        {view.name.clone()}
                                                    </span>
                                                </a>
                                            }
                                                .into_any(),
                                            None => view! { {view.name.clone()} }.into_any(),
                                        }}
                                    </td>
                                    <td class="text-text-soft">{view.account.clone()}</td>
                                    <td class="text-text-soft">
                                        {view.course.clone().unwrap_or_else(|| "—".to_string())}
                                    </td>
                                    <td class="text-right tabular-nums text-text-soft">
                                        {format!("{}口", format_number_value(view.shares))}
                                    </td>
                                    <td class="text-right tabular-nums text-text-soft">
                                        {format_currency(view.base_price)}
                                    </td>
                                    <td class="text-right tabular-nums">
                                        {day_change_cell(view.base_price_day_change, outlier_date)}
                                    </td>
                                    <td class="text-right font-bold tabular-nums text-brand-text">
                                        {format_row_valuation_amount(Some(view.market_value))}
                                    </td>
                                    <td class="text-right tabular-nums">
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
                        "{} ・ {}口 ・ 基準価額 {}",
                        view.account,
                        format_number_value(view.shares),
                        format_currency(view.base_price),
                    );
                    let inner = view! {
                        <>
                            <div class="flex items-baseline justify-between gap-2">
                                <span class="min-w-0 truncate font-medium text-text">
                                    {view.name.clone()}
                                </span>
                                <span class="shrink-0 font-bold tabular-nums text-brand-text">
                                    {format_row_valuation_amount(Some(view.market_value))}
                                </span>
                            </div>
                            <div class="mt-0.5 flex items-baseline justify-between gap-2 text-xs">
                                <span class="min-w-0 truncate text-text-muted">{meta}</span>
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

/// 表の下に出す基準価額の時点。見出しと同じ代表の基準日("YYYY-MM-DD")を受け取る
#[component]
pub fn FundPriceAsOfNote(date: Option<String>) -> impl IntoView {
    let text = date.map(|date| format!("基準価額 {} 時点", format_price_date(&date)));
    move || {
        text.clone().map(|text| {
            view! {
                <p class="mt-1 px-4 text-xs text-text-muted lg:px-0">{text}</p>
            }
        })
    }
}
