use crate::features::asset_balance::format::{
    format_signed_valuation_amount, format_signed_valuation_rate, format_valuation_amount,
    is_negative_valuation, is_positive_valuation,
};
use crate::features::asset_balance::holdings::{
    HoldingView, format_dividend_annual, format_dividend_yield, holding_dividend,
};
use crate::features::asset_balance::model::format_number_value;
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::dividend_per_share::DividendMaps;
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
                    let market_text = format_valuation_amount(valuation.market_value);
                    let pnl_text = format_signed_valuation_amount(valuation.profit_loss.amount);
                    let pnl_rate_text = format_signed_valuation_rate(valuation.profit_loss.rate, 1);
                    let pnl_negative = is_negative_valuation(valuation.profit_loss.amount);
                    let (pnl_text_class, pnl_badge_class) = if pnl_negative {
                        ("text-negative", "asset-badge asset-badge-loss")
                    } else if is_positive_valuation(valuation.profit_loss.amount) {
                        ("text-positive", "asset-badge asset-badge-gain")
                    } else {
                        ("text-text-soft", "asset-badge")
                    };
                    let dividend = holding_dividend(
                        &view.code,
                        view.shares,
                        view.average_price,
                        &dividends.get(),
                    );
                    let dividend_annual = format_dividend_annual(&dividend);
                    let dividend_yield = format_dividend_yield(&dividend);
                    let shares_text = format!("{}株", format_number_value(view.shares));
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
                                <span class="shrink-0 text-text-muted">
                                    {view.code.clone()}
                                    " · "
                                    {shares_text}
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
