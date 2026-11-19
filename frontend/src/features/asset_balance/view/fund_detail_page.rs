use super::use_asset_balance_state;
use crate::api::dto::FundPriceSource;
use crate::features::asset_balance::format as balance_format;
use crate::features::asset_balance::fund;
use crate::session;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::asset_balance::{
    AssetPageEmpty, AssetPageError, AssetRateAmount, FundDetailBody, FundDetailCard,
    FundDetailHeader, FundDetailMeta,
};
use crate::ui::badge::AccountBadge;
use crate::ui::format;
use crate::ui::holding_tiles::{HoldingTile, HoldingTiles};
use crate::ui::labels;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

#[component]
pub fn AssetFundHoldingPage(id: String) -> impl IntoView {
    let session = session::use_session();
    let state = use_asset_balance_state(session);
    let funds = state.funds;
    let data_ops = state.data_ops;
    if session.user.get_untracked().is_some() {
        crate::features::asset_balance::store::load_fund_balances(
            session,
            session.generation.get_untracked(),
            funds,
            data_ops,
        );
    }
    let resolved = move || {
        let generation = session.generation.get();
        funds.with(|slot| {
            crate::features::asset_balance::store::csv::resolve_fund_balance(generation, slot)
        })
    };
    let reload = move || {
        if session.user.get_untracked().is_none() {
            return;
        }
        funds.set(None);
        crate::features::asset_balance::store::load_fund_balances(
            session,
            session.generation.get_untracked(),
            funds,
            data_ops,
        );
    };
    view! {
        <PageFrame
            title=labels::ASSET_FUND_DETAIL_TITLE
            back_href="/assetbalance"
            back_label=labels::ASSET_BALANCE_BACK
        >
            <div class="flex flex-1 flex-col">
                {move || {
                    data_ops
                        .with(|ops| ops.refresh_error.clone())
                        .map(|message| {
                            view! { <AssetPageError message=message /> }
                        })
                }}
                {move || {
                    let generation = session.generation.get();
                    let list_error = funds.with(|slot| match slot {
                        Some((cached, Err(message))) if *cached == generation => {
                            Some(message.clone())
                        }
                        _ => None,
                    });
                    let view = resolved();
                    if let Some(message) = list_error {
                        return view! { <ListLoadError message=message on_retry=reload /> }
                            .into_any();
                    }
                    match view {
                        None => view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }.into_any(),
                        Some(view) => {
                            let found = view.rows.iter().find(|row| {
                                row.id.to_string() == id
                            });
                            let Some(row) = found else {
                                return view! {
                                    <AssetPageEmpty title=labels::ASSET_FUND_NOT_FOUND />
                                }
                                    .into_any();
                            };
                                let view = fund::fund_view(row);
                                let shares_text =
                                    labels::fund_units_label(&format::format_number_value(view.shares));
                                let average_price_text = format::format_unit_price(
                                    view.average_purchase_price,
                                    format::unit_price_decimals(
                                        std::iter::once(view.average_purchase_price),
                                    ),
                                );
                                let purchase_text = format::format_currency(view.purchase);
                                let base_price_text = format::format_currency(view.base_price);
                                let market_text =
                                    balance_format::format_row_valuation_amount(Some(view.market_value));
                                let pnl_text =
                                    balance_format::format_signed_row_valuation_amount(Some(view.profit_loss));
                                let pnl_rate_text =
                                    balance_format::format_signed_valuation_rate(view.profit_loss_rate, 1);
                                let pnl_rate_negative = balance_format::is_negative_rate(view.profit_loss_rate);
                                let pnl_rate_positive = balance_format::is_positive_rate(view.profit_loss_rate);
                                let pnl_negative =
                                    balance_format::is_negative_valuation(Some(view.profit_loss));
                                let pnl_positive =
                                    balance_format::is_positive_valuation(Some(view.profit_loss));
                                let day_text = if view.nav_pending() {
                                    labels::PENDING_UPDATE.to_string()
                                } else {
                                    view.base_price_day_change.map_or_else(|| labels::MISSING.to_string(), |value| {
                                            balance_format::format_signed_row_valuation_amount(Some(value))
                                        })
                                };
                                let month_text = view
                                    .base_price_month_change.map_or_else(|| labels::MISSING.to_string(), |value| {
                                        balance_format::format_signed_row_valuation_amount(Some(value))
                                    });
                                let total_return_text = view
                                    .total_return.map_or_else(|| labels::MISSING.to_string(), |value| {
                                        balance_format::format_signed_row_valuation_amount(Some(value))
                                    });
                                let day_negative = balance_format::is_negative_valuation(view.base_price_day_change);
                                let day_positive = balance_format::is_positive_valuation(view.base_price_day_change);
                                let month_negative = balance_format::is_negative_valuation(view.base_price_month_change);
                                let month_positive = balance_format::is_positive_valuation(view.base_price_month_change);
                                let total_negative = balance_format::is_negative_valuation(view.total_return);
                                let total_positive = balance_format::is_positive_valuation(view.total_return);
                                let asof_text = view
                                    .price_as_of
                                    .map_or_else(|| labels::MISSING.to_string(), |date| format::format_iso_date(&date));
                                let asof_label = match view.price_source {
                                    FundPriceSource::Api => labels::BASE_PRICE_AS_OF,
                                    FundPriceSource::Csv => labels::CSV_IMPORTED_AT,
                                };
                                view! {
                                    <FundDetailCard>
                                        <FundDetailHeader>
                                            <h2 class="text-heading font-bold text-text">
                                                {view.name.clone()}
                                            </h2>
                                            <FundDetailMeta>
                                                <AccountBadge label=view.account.clone() />
                                                {view
                                                    .course
                                                    .clone()
                                                    .map(|course| labels::course_with_dot(&course))}
                                            </FundDetailMeta>
                                        </FundDetailHeader>
                                        <FundDetailBody>
                                            <HoldingTiles>
                                                <HoldingTile label=labels::MARKET_VALUE.to_string() wide=true>
                                                    <Amount text=market_text />
                                                </HoldingTile>
                                                <HoldingTile label=labels::VALUATION_PROFIT_LOSS.to_string() wide=true>
                                                    <Amount
                                                        kind=AmountKind::Delta
                                                        class="tabular-nums"
                                                        text=pnl_text
                                                        negative=pnl_negative
                                                        positive=pnl_positive
                                                    />
                                                    <AssetRateAmount
                                                        text=pnl_rate_text
                                                        negative=pnl_rate_negative
                                                        positive=pnl_rate_positive
                                                    />
                                                </HoldingTile>
                                                <HoldingTile label=labels::BASE_PRICE.to_string()>
                                                    <Amount text=base_price_text />
                                                </HoldingTile>
                                                <HoldingTile label=labels::BASE_PRICE_DAY_CHANGE.to_string()>
                                                    <Amount
                                                        kind=AmountKind::Delta
                                                        class="tabular-nums"
                                                        text=day_text
                                                        negative=day_negative
                                                        positive=day_positive
                                                    />
                                                </HoldingTile>
                                                <HoldingTile label=labels::BASE_PRICE_MONTH_CHANGE.to_string()>
                                                    <Amount
                                                        kind=AmountKind::Delta
                                                        class="tabular-nums"
                                                        text=month_text
                                                        negative=month_negative
                                                        positive=month_positive
                                                    />
                                                </HoldingTile>
                                                <HoldingTile label=asof_label.to_string()>{asof_text}</HoldingTile>
                                                <HoldingTile label=labels::FUND_SHARES.to_string()>
                                                    <span class="tabular-nums">{shares_text}</span>
                                                </HoldingTile>
                                                <HoldingTile label=labels::AVERAGE_PRICE_PER_10K.to_string()>
                                                    <Amount text=average_price_text />
                                                </HoldingTile>
                                                <HoldingTile label=labels::TOTAL_ACQUISITION.to_string()>
                                                    <Amount text=purchase_text />
                                                </HoldingTile>
                                                <HoldingTile label=labels::TOTAL_RETURN.to_string()>
                                                    <Amount
                                                        kind=AmountKind::Delta
                                                        class="tabular-nums"
                                                        text=total_return_text
                                                        negative=total_negative
                                                        positive=total_positive
                                                    />
                                                </HoldingTile>
                                            </HoldingTiles>
                                        </FundDetailBody>
                                    </FundDetailCard>
                                }
                                    .into_any()
                        }
                    }
                }}
            </div>
        </PageFrame>
    }
}
