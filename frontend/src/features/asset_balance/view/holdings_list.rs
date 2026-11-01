use crate::features::asset_balance::format::{
    format_row_valuation_amount, format_signed_row_valuation_amount, format_signed_valuation_rate,
    is_negative_rate, is_negative_valuation, is_positive_rate, is_positive_valuation,
};
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::model::{ValuationInput, holding_valuation};
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::format::format_number_value;
use crate::ui::list_row::ListRow;
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
                    let pnl_negative = is_negative_valuation(valuation.profit_loss.amount);
                    let pnl_positive = is_positive_valuation(valuation.profit_loss.amount);
                    let rate_negative = is_negative_rate(valuation.profit_loss.rate);
                    let rate_positive = is_positive_rate(valuation.profit_loss.rate);
                    let shares_text = format!("{}株", format_number_value(view.shares));
                    let code = view.code.clone();
                    let name = view.name.clone();
                    let account = view.account.clone();
                    let inner = view! {
                        <ListRow
                            name=move || name.clone()
                            primary=move || view! {
                                <Amount
                                    kind=AmountKind::Plain
                                    text=market_text.clone()
                                />
                            }
                            meta=move || view! {
                                {code.clone()}
                                {account
                                    .clone()
                                    .map(|account| {
                                        view! {
                                            " · "
                                            <AccountBadge label=account />
                                        }
                                    })}
                                {" · "}
                                <Amount kind=AmountKind::Plain text=shares_text.clone() />
                            }
                            secondary=move || view! {
                                {if valuation.profit_loss.amount.is_some() {
                                    view! {
                                        <Amount
                                            kind=AmountKind::Delta
                                            text=pnl_text.clone()
                                            negative=pnl_negative
                                            positive=pnl_positive
                                        />
                                        {valuation
                                            .profit_loss
                                            .rate
                                            .is_some()
                                            .then(|| {
                                                view! {
                                                    " "
                                                    <Amount
                                                        kind=AmountKind::Delta
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
                            }
                        />
                    };
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
