use crate::features::asset_balance::format::{
    format_currency, format_row_valuation_amount, format_signed_row_valuation_amount, pnl_cells,
};
use crate::features::asset_balance::holdings::{
    HoldingView, format_dividend_annual, format_dividend_yield, holding_dividend,
};
use crate::features::asset_balance::model::format_number_value;
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::badge::account_display_name;
use leptos::prelude::*;

#[component]
pub fn HoldingsList(
    views: Vec<HoldingView>,
    dividends: RwSignal<DividendMaps>,
    #[prop(optional)] show_dividend: bool,
) -> impl IntoView {
    view! {
        <ul class="asset-compact-list" data-testid="asset-holdings-list">
            {views
                .into_iter()
                .map(|view| {
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
                    let shares_text = format!("{}株", format_number_value(view.shares));
                    let account_text = view
                        .account
                        .as_deref()
                        .map(|account| format!(" · {}", account_display_name(account)))
                        .unwrap_or_default();
                    let meta = format!(
                        "{}{} · {} · 現在値 {} · 前日比 {}",
                        view.code,
                        account_text,
                        shares_text,
                        format_currency(view.current_price),
                        format_signed_row_valuation_amount(view.day_change),
                    );
                    let inner = view! {
                        <>
                            <div class="flex items-baseline justify-between gap-2">
                                <span class="min-w-0 truncate font-medium text-text">
                                    {view.name.clone()}
                                </span>
                                <span class="shrink-0 font-bold tabular-nums text-brand-text">
                                    {market_text}
                                </span>
                            </div>
                            <div class="mt-0.5 flex items-baseline justify-between gap-2 text-xs">
                                <span class="min-w-0 truncate text-text-muted">
                                    {meta}
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
                                                "配当 "
                                                {dividend_annual}
                                            </span>
                                            <span class="text-text-muted tabular-nums">
                                                "利回り "
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
