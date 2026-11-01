use crate::features::asset_balance::format::{
    format_row_valuation_amount, format_signed_row_valuation_amount, format_signed_valuation_rate,
    is_negative_rate, is_negative_valuation, is_positive_rate, is_positive_valuation,
};
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::format::format_number_value;
use leptos::prelude::*;

#[component]
pub fn HoldingsList(views: Vec<HoldingView>) -> impl IntoView {
    view! {
        <ul class="asset-compact-list" data-testid="asset-holdings-list">
            {views
                .into_iter()
                .map(|view| {
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
                    let rate_negative = is_negative_rate(valuation.profit_loss.rate);
                    let rate_positive = is_positive_rate(valuation.profit_loss.rate);
                    let shares_text = format!("{}株", format_number_value(view.shares));
                    let inner = view! {
                        <>
                            <div class="flex items-baseline justify-between gap-2">
                                <span class="min-w-0 flex-1 break-words font-medium text-text">
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
                                    {" · "}
                                    <Amount
                                        kind=AmountKind::Plain
                                        text=shares_text
                                    />
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
