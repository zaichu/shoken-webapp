use super::use_asset_balance_state;
use crate::api::dto::FundPriceSource;
use crate::features::asset_balance::format::{
    format_row_valuation_amount, format_signed_row_valuation_amount, format_signed_valuation_rate,
    is_negative_valuation, is_positive_valuation,
};
use crate::features::asset_balance::fund::fund_view;
use crate::session::use_session;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::empty_state::EmptyState;
use crate::ui::format::{format_currency, format_iso_date, format_number_value};
use crate::ui::holding_tiles::{HoldingTile, HoldingTiles};
use crate::ui::labels;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

#[component]
pub fn AssetFundHoldingPage(id: String) -> impl IntoView {
    let session = use_session();
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
                            view! {
                                <div class="no-print mb-3">
                                    <Alert variant=AlertVariant::Danger>
                                        <strong>{labels::ERROR_PREFIX}</strong>
                                        " "
                                        {message}
                                    </Alert>
                                </div>
                            }
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
                                    <div class="px-4 py-4">
                                        <EmptyState title=labels::ASSET_FUND_NOT_FOUND />
                                    </div>
                                }
                                    .into_any();
                            };
                                let view = fund_view(row);
                                let shares_text =
                                    labels::fund_units_label(&format_number_value(view.shares));
                                let average_price_text =
                                    format_currency(view.average_purchase_price);
                                let purchase_text = format_currency(view.purchase);
                                let base_price_text = format_currency(view.base_price);
                                let market_text =
                                    format_row_valuation_amount(Some(view.market_value));
                                let pnl_text =
                                    format_signed_row_valuation_amount(Some(view.profit_loss));
                                let pnl_rate_text =
                                    format_signed_valuation_rate(view.profit_loss_rate, 1);
                                let pnl_negative =
                                    is_negative_valuation(Some(view.profit_loss));
                                let pnl_positive =
                                    is_positive_valuation(Some(view.profit_loss));
                                let day_text = if view.nav_pending() {
                                    labels::PENDING_UPDATE.to_string()
                                } else {
                                    view.base_price_day_change.map_or_else(|| labels::MISSING.to_string(), |value| {
                                            format_signed_row_valuation_amount(Some(value))
                                        })
                                };
                                let month_text = view
                                    .base_price_month_change.map_or_else(|| labels::MISSING.to_string(), |value| {
                                        format_signed_row_valuation_amount(Some(value))
                                    });
                                let total_return_text = view
                                    .total_return.map_or_else(|| labels::MISSING.to_string(), |value| {
                                        format_signed_row_valuation_amount(Some(value))
                                    });
                                let day_negative = is_negative_valuation(view.base_price_day_change);
                                let day_positive = is_positive_valuation(view.base_price_day_change);
                                let month_negative = is_negative_valuation(view.base_price_month_change);
                                let month_positive = is_positive_valuation(view.base_price_month_change);
                                let total_negative = is_negative_valuation(view.total_return);
                                let total_positive = is_positive_valuation(view.total_return);
                                let asof_text = view
                                    .price_as_of
                                    .map_or_else(|| labels::MISSING.to_string(), |date| format_iso_date(&date));
                                let asof_label = match view.price_source {
                                    FundPriceSource::Api => labels::BASE_PRICE_AS_OF,
                                    FundPriceSource::Csv => labels::CSV_IMPORTED_AT,
                                };
                                view! {
                                    <div class="asset-panel">
                                        <div class="border-b border-border p-4">
                                            <h2 class="text-lg font-bold text-text">
                                                {view.name.clone()}
                                            </h2>
                                            <p class="mt-1 flex items-center gap-1 text-sm text-text-muted">
                                                <AccountBadge label=view.account.clone() />
                                                {view
                                                    .course
                                                    .clone()
                                                    .map(|course| labels::course_with_dot(&course))}
                                            </p>
                                        </div>
                                        <div class="p-4">
                                            <HoldingTiles>
                                                <HoldingTile label=labels::MARKET_VALUE.to_string() wide=true>
                                                    <span class="tabular-nums">{market_text}</span>
                                                </HoldingTile>
                                                <HoldingTile label=labels::VALUATION_PROFIT_LOSS.to_string() wide=true>
                                                    <Amount
                                                        kind=AmountKind::Delta
                                                        class="tabular-nums"
                                                        text=pnl_text
                                                        negative=pnl_negative
                                                        positive=pnl_positive
                                                    />
                                                    <span class="ml-1 text-xs text-text-muted tabular-nums">
                                                        {pnl_rate_text}
                                                    </span>
                                                </HoldingTile>
                                                <HoldingTile label=labels::BASE_PRICE.to_string()>
                                                    <span class="tabular-nums">{base_price_text}</span>
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
                                                <HoldingTile label=labels::AVERAGE_ACQUISITION_PRICE.to_string()>
                                                    <span class="tabular-nums">{average_price_text}</span>
                                                </HoldingTile>
                                                <HoldingTile label=labels::TOTAL_ACQUISITION.to_string()>
                                                    <span class="tabular-nums">{purchase_text}</span>
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
                                        </div>
                                    </div>
                                }
                                    .into_any()
                        }
                    }
                }}
            </div>
        </PageFrame>
    }
}
