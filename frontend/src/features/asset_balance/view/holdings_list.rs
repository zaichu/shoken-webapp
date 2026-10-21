use crate::features::asset_balance::format::{
    format_row_valuation_amount, format_signed_row_valuation_amount, pnl_cells,
};
use crate::features::asset_balance::holdings::{
    HoldingView, format_dividend_annual, format_dividend_per_share, format_dividend_yield,
    holding_dividend,
};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::badge::AccountBadge;
use crate::ui::format::{format_number_value, format_unit_price, unit_price_decimals};
use crate::ui::labels::{
    ACQUISITION_YIELD, CURRENT_PRICE, DAY_CHANGE, DIVIDEND, PER_SHARE_DIVIDEND,
};
use leptos::prelude::*;

#[component]
pub fn HoldingsList(
    views: Vec<HoldingView>,
    dividends: RwSignal<DividendMaps>,
    #[prop(optional)] show_dividend: bool,
) -> impl IntoView {
    // 列の中の小数桁をそろえる(整数円の値しかない列は .00 を付けない)
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
        <ul class="asset-compact-list" data-testid="asset-holdings-list">
            {views
                .into_iter()
                .zip(row_dividends)
                .map(|(view, dividend)| {
                    let valuation = holding_valuation(&ValuationInput::from(&view));
                    let market_text = format_row_valuation_amount(valuation.market_value);
                    let (pnl_text, pnl_rate_text, pnl_text_class, pnl_badge_class) =
                        pnl_cells(valuation.profit_loss.amount, valuation.profit_loss.rate);
                    let dividend_annual = format_dividend_annual(&dividend);
                    let dividend_per_share =
                        format_dividend_per_share(&dividend, dividend_decimals);
                    let dividend_yield = format_dividend_yield(&dividend);
                    let shares_text = format!("{}株", format_number_value(view.shares));
                    let meta_tail = format!(
                        " · {} · {} {} · {} {}",
                        shares_text,
                        CURRENT_PRICE,
                        format_unit_price(view.current_price, current_price_decimals),
                        DAY_CHANGE,
                        format_signed_row_valuation_amount(view.day_change),
                    );
                    let inner = view! {
                        <>
                            <div class="flex items-baseline justify-between gap-2">
                                <span class="min-w-0 truncate font-medium text-text">
                                    {view.name.clone()}
                                </span>
                                <span class="shrink-0 font-bold tabular-nums text-text-deep">
                                    {market_text}
                                </span>
                            </div>
                            <div class="mt-0.5 flex items-baseline justify-between gap-2 text-xs">
                                <span class="min-w-0 truncate text-text-muted">
                                    {view.code.clone()}
                                    {view
                                        .account
                                        .clone()
                                        .map(|account| {
                                            view! {
                                                " · "
                                                <AccountBadge label=account />
                                            }
                                        })}
                                    {meta_tail}
                                </span>
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
                                        <span class="flex shrink-0 items-baseline">
                                            <span class=format!("font-bold tabular-nums {pnl_text_class}")>
                                                {pnl_text}
                                            </span>
                                            {rate_badge}
                                        </span>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <span class="shrink-0 text-text-muted tabular-nums">
                                            "—"
                                        </span>
                                    }
                                        .into_any()
                                }}
                            </div>
                            {show_dividend
                                .then(|| {
                                    view! {
                                        <div class="mt-0.5 flex items-baseline justify-between gap-2 text-xs">
                                            <span class="text-text-muted">
                                                {DIVIDEND}
                                                " "
                                                {dividend_annual}
                                                " · "
                                                {PER_SHARE_DIVIDEND}
                                                " "
                                                {dividend_per_share}
                                            </span>
                                            <span class="text-text-muted tabular-nums">
                                                {ACQUISITION_YIELD}
                                                " "
                                                {dividend_yield}
                                            </span>
                                        </div>
                                    }
                                })}
                        </>
                    };
                    // 詳細は保存済みレコードの ID で開く。CSV プレビュー行はリンクにしない
                    view! {
                        <li class="asset-compact-row" data-testid="asset-compact-row">
                            {match &view.id {
                                Some(id) => view! {
                                    <a
                                        href=format!("/assetbalance/holdings/{id}")
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
