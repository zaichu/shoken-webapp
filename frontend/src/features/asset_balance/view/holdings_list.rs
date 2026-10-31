use crate::features::asset_balance::format::{
    format_row_valuation_amount, format_signed_row_valuation_amount, format_signed_valuation_rate,
    is_negative_valuation, is_positive_valuation,
};
use crate::features::asset_balance::holdings::{
    HoldingView, format_dividend_annual, format_dividend_per_share, format_dividend_yield,
    holding_dividend,
};
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::format::{format_number_value, format_unit_price, unit_price_decimals};
use crate::ui::labels::{
    ACQUISITION_YIELD_COMPACT, CURRENT_PRICE, DAY_CHANGE, DIVIDEND, PER_SHARE_DIVIDEND_COMPACT,
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
                    let pnl_text = format_signed_row_valuation_amount(
                        valuation.profit_loss.amount,
                    );
                    let pnl_rate_text = format_signed_valuation_rate(
                        valuation.profit_loss.rate,
                        1,
                    );
                    let pnl_negative =
                        is_negative_valuation(valuation.profit_loss.amount);
                    let pnl_positive =
                        is_positive_valuation(valuation.profit_loss.amount);
                    let rate_negative = is_negative_valuation(valuation.profit_loss.rate);
                    let rate_positive = is_positive_valuation(valuation.profit_loss.rate);
                    let day_text =
                        format_signed_row_valuation_amount(view.day_change);
                    let day_negative = is_negative_valuation(view.day_change);
                    let day_positive = is_positive_valuation(view.day_change);
                    let dividend_annual = format_dividend_annual(&dividend);
                    let dividend_per_share =
                        format_dividend_per_share(&dividend, dividend_decimals);
                    let dividend_yield = format_dividend_yield(&dividend);
                    let shares_text = format!("{}株", format_number_value(view.shares));
                    let price_text = format_unit_price(
                        view.current_price,
                        current_price_decimals,
                    );
                    let inner = view! {
                        <>
                            <div class="flex items-baseline justify-between gap-2">
                                <span class="min-w-0 truncate font-medium text-text">
                                    {view.name.clone()}
                                </span>
                                <Amount
                                    kind=AmountKind::Plain
                                    text=market_text
                                    class="shrink-0 font-bold text-text-deep"
                                />
                            </div>
                            <div class="mt-0.5 flex items-start justify-between gap-2 text-xs">
                                <span class="min-w-0 flex-1 break-words text-text-muted">
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
                                </span>
                                {if valuation.profit_loss.amount.is_some() {
                                    view! {
                                        <span class="flex shrink-0 items-baseline whitespace-nowrap">
                                            <Amount
                                                kind=AmountKind::Delta
                                                text=pnl_text
                                                negative=pnl_negative
                                                positive=pnl_positive
                                                class="font-bold"
                                            />
                                            {valuation
                                                .profit_loss
                                                .rate
                                                .is_some()
                                                .then(|| {
                                                    view! {
                                                        <Amount
                                                            kind=AmountKind::Delta
                                                            text=pnl_rate_text.clone()
                                                            negative=rate_negative
                                                            positive=rate_positive
                                                            class="ml-1 text-xs"
                                                        />
                                                    }
                                                })}
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
                            <div class="mt-0.5 flex flex-wrap items-baseline gap-x-2 gap-y-0.5 text-xs text-text-muted">
                                <Amount
                                    kind=AmountKind::Plain
                                    text=shares_text
                                />
                                <span class="whitespace-nowrap">
                                    {CURRENT_PRICE}
                                    " "
                                    <Amount
                                        kind=AmountKind::Plain
                                        text=price_text
                                    />
                                </span>
                                <span class="whitespace-nowrap">
                                    {DAY_CHANGE}
                                    " "
                                    <Amount
                                        kind=AmountKind::Delta
                                        text=day_text
                                        negative=day_negative
                                        positive=day_positive
                                    />
                                </span>
                            </div>
                            {show_dividend
                                .then(|| {
                                    view! {
                                        <div class="mt-0.5 flex flex-wrap items-baseline gap-x-2 gap-y-0.5 text-xs">
                                            <span class="whitespace-nowrap text-text-muted">
                                                {DIVIDEND}
                                                " "
                                                <Amount
                                                    kind=AmountKind::Plain
                                                    text=dividend_annual
                                                />
                                            </span>
                                            <span class="whitespace-nowrap text-text-muted">
                                                {PER_SHARE_DIVIDEND_COMPACT}
                                                " "
                                                <Amount
                                                    kind=AmountKind::Plain
                                                    text=dividend_per_share
                                                />
                                            </span>
                                            <span class="whitespace-nowrap text-text-muted">
                                                {ACQUISITION_YIELD_COMPACT}
                                                " "
                                                <Amount
                                                    kind=AmountKind::Plain
                                                    text=dividend_yield
                                                />
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
