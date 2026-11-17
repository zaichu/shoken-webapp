use super::super::model;
use super::super::store::{self, HomeOverviewData};
use super::card_parts::{CardError, CardHead, CardSkeleton};
use super::{funds_of, stocks_of};
use crate::features::asset_balance::{
    format_signed_row_valuation_amount, format_signed_valuation_rate, is_negative_rate,
    is_negative_valuation, is_positive_rate, is_positive_valuation,
};
use crate::session::use_session;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::labels;
use crate::ui::list_row::{ListGroupHeader, ListRowLink};
use leptos::prelude::*;

#[component]
pub(super) fn PnlRankingCard(data: HomeOverviewData) -> impl IntoView {
    let session = use_session();
    let card = move || {
        let generation = session.generation.get();
        let assets = store::current_summary(data.assets, generation);
        let funds = store::current_summary(data.funds, generation);
        (assets, funds)
    };
    view! {
        <Card variant=CardVariant::Panel class="overflow-clip" testid="home-pnl-card">
            <CardHead
                title=labels::HOME_PNL_TITLE
                href="/assetbalance"
                link_label=labels::ASSET_BALANCE_TITLE
            />
            {move || {
                let (assets, funds) = card();
                if assets.is_none() && funds.is_none() {
                    return view! { <CardSkeleton /> }.into_any();
                }
                if matches!(assets, Some(None)) && matches!(funds, Some(None)) {
                    return view! { <CardError message=labels::HOME_HOLDING_FETCH_FAILED.to_string() /> }
                        .into_any();
                }
                let stocks = stocks_of(&assets).unwrap_or(&[]);
                let fund_rows = funds_of(&funds).unwrap_or(&[]);
                if stocks.is_empty() && fund_rows.is_empty() {
                    return view! {
                        <div class="px-4 py-4">
                            <EmptyState title=labels::HOME_NO_STOCKS />
                        </div>
                    }
                        .into_any();
                }
                let holdings = model::holding_pnls(stocks, fund_rows);
                let (gains, losses) = model::pnl_extremes(&holdings, 3, 2);
                view! {
                    <ul
                        class="asset-compact-list home-pnl-list"
                        data-testid="home-pnl-list"
                    >
                        {gains
                            .iter()
                            .map(|holding| view! { <PnlRow holding=holding.clone() /> })
                            .collect_view()}
                        {(!losses.is_empty())
                            .then(|| {
                                view! {
                                    <li aria-hidden="true">
                                        <ListGroupHeader label=labels::HOME_LOSS_GROUP.to_string() summary=None />
                                    </li>
                                }
                            })}
                        {losses
                            .iter()
                            .map(|holding| view! { <PnlRow holding=holding.clone() /> })
                            .collect_view()}
                    </ul>
                    {if gains.is_empty() && losses.is_empty() {
                        view! {
                            <div class="px-4 py-4">
                                <EmptyState title=labels::HOME_NO_PNL />
                            </div>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }}
                }
                    .into_any()
            }}
        </Card>
    }
}

#[component]
fn PnlRow(holding: model::HoldingPnl) -> impl IntoView {
    let negative = is_negative_valuation(Some(holding.profit_loss));
    let positive = is_positive_valuation(Some(holding.profit_loss));
    let rate_negative = holding
        .rate
        .is_some_and(|rate| is_negative_rate(Some(rate)));
    let rate_positive = holding
        .rate
        .is_some_and(|rate| is_positive_rate(Some(rate)));
    let name = holding.name.clone();
    let code = holding.code.clone();
    let href = holding.href.clone();
    let amount_text = format_signed_row_valuation_amount(Some(holding.profit_loss));
    let rate_text = holding.rate.map_or_else(
        || labels::MISSING.to_string(),
        |rate| format_signed_valuation_rate(Some(rate), 1),
    );
    view! {
        <li class="list-row-item">
            <ListRowLink
                href=href
                name=move || name.clone()
                primary=move || view! {
                    <Amount
                        kind=AmountKind::Delta
                        text=amount_text.clone()
                        negative=negative
                        positive=positive
                    />
                }
                meta=move || view! {
                    {code.clone()}
                }
                secondary=move || view! {
                    <Amount
                        kind=AmountKind::Delta
                        text=rate_text.clone()
                        negative=rate_negative
                        positive=rate_positive
                    />
                }
            />
        </li>
    }
}
