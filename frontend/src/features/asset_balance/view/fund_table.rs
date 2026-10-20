use super::day_change::day_change_subline;
use crate::features::asset_balance::format::{
    format_currency, format_row_valuation_amount, format_signed_row_valuation_amount,
    format_signed_valuation_rate, is_negative_valuation, is_positive_valuation,
};
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::model::format_number_value;
use crate::features::asset_balance::portfolio::SortOrder;
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
fn FundSortHeader(
    label: &'static str,
    key: FundSortKey,
    sort_key: RwSignal<FundSortKey>,
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

/// ファンド1行の損益セル(表・一覧で共用)
pub(crate) fn fund_pnl_cells(view: &FundView) -> (String, String, &'static str, &'static str) {
    let amount = view.profit_loss;
    let rate = view.profit_loss_rate;
    let pnl_text = format_signed_row_valuation_amount(Some(amount));
    let pnl_rate_text = format_signed_valuation_rate(rate, 1);
    let negative = is_negative_valuation(Some(amount));
    let (text_class, badge_class) = if negative {
        ("text-negative", "asset-badge asset-badge-loss")
    } else if is_positive_valuation(Some(amount)) {
        ("text-positive", "asset-badge asset-badge-gain")
    } else {
        ("text-text-soft", "asset-badge")
    };
    (pnl_text, pnl_rate_text, text_class, badge_class)
}

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
                        <th scope="col">"ファンド"</th>
                        <FundSortHeader label="保有口数" key=FundSortKey::Shares sort_key=sort_key sort_order=sort_order />
                        <FundSortHeader label="基準価額" key=FundSortKey::BasePrice sort_key=sort_key sort_order=sort_order />
                        <FundSortHeader label="評価額" key=FundSortKey::MarketValue sort_key=sort_key sort_order=sort_order />
                        <FundSortHeader label="評価損益" key=FundSortKey::ProfitLoss sort_key=sort_key sort_order=sort_order />
                    </tr>
                </thead>
                <tbody>
                    {views
                        .into_iter()
                        .map(|view| {
                            let (pnl_text, pnl_rate_text, pnl_text_class, pnl_badge_class) =
                                fund_pnl_cells(&view);
                            let day_change = view
                                .base_price_day_change
                                .map(|change| day_change_subline(Some(change)));
                            let href = view
                                .id
                                .clone()
                                .map(|id| format!("/assetbalance/funds/{id}"));
                            let name_cell = view! {
                                <>
                                    {view.name.clone()}
                                    <div class="asset-price-date">
                                        {view.account.clone()}
                                            {view
                                                .course
                                                .clone()
                                                .map(|course| format!(" ・ {course}"))
                                                .unwrap_or_default()}
                                    </div>
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
                                        {format!("{}口", format_number_value(view.shares))}
                                    </td>
                                    <td class="text-right tabular-nums text-text-soft">
                                        {format_currency(view.base_price)}
                                        {day_change}
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
                        fund_pnl_cells(&view);
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

/// 表の下に出す基準価額の時点
#[component]
pub fn FundPriceAsOfNote(views: Vec<FundView>) -> impl IntoView {
    let dates: Vec<String> = views
        .iter()
        .filter_map(|view| view.price_as_of.clone())
        .collect();
    let text =
        crate::features::asset_balance::format::price_date_range(dates.iter().map(String::as_str))
            .map(|(min, max)| {
                if min == max {
                    format!("基準価額 {min} 時点")
                } else {
                    format!("基準価額 {min} 〜 {max} 時点")
                }
            });
    move || {
        text.clone().map(|text| {
            view! {
                <p class="mt-1 px-4 text-xs text-text-muted lg:px-0">{text}</p>
            }
        })
    }
}
