use super::use_asset_balance_state;
use crate::features::asset_balance::format::{
    format_currency, format_row_valuation_amount, format_signed_row_valuation_amount,
    format_signed_valuation_rate, format_unit_price, is_negative_valuation,
};
use crate::features::asset_balance::holdings::{
    format_dividend_annual, format_dividend_per_share, format_dividend_yield, holding_dividend,
};
use crate::features::asset_balance::model::{
    ValuationInput, format_number_value, holding_valuation,
};
use crate::session::use_session;
use crate::ui::amount::Amount;
use crate::ui::badge::AccountBadge;
use crate::ui::page_frame::PageFrame;
use crate::ui::security_link::{SecurityCodeLink, SecurityLinkVariant};
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

#[component]
pub fn AssetBalanceHoldingPage(id: String) -> impl IntoView {
    let session = use_session();
    let state = use_asset_balance_state(session);
    let balances = state.balances;
    let dividends = state.dividends;
    let lookup = state.lookup;
    let data_ops = state.data_ops;
    let csv_slot = state.csv.csv;
    if session.user.get_untracked().is_some() {
        crate::features::asset_balance::store::load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            csv_slot,
            data_ops,
        );
    }
    let resolved = move || {
        let generation = session.generation.get();
        balances.with(|slot| {
            crate::features::asset_balance::store::csv::resolve_asset_balance(generation, slot)
        })
    };
    let reload = move || {
        if session.user.get_untracked().is_none() {
            return;
        }
        balances.set(None);
        crate::features::asset_balance::store::load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            csv_slot,
            data_ops,
        );
    };
    view! {
        <PageFrame
            title="保有銘柄詳細"
            back_href="/assetbalance"
            back_label="資産管理に戻る"
        >
            <div class="page-gutter flex flex-1 flex-col py-4">
                {move || {
                    data_ops
                        .with(|ops| ops.refresh_error.clone())
                        .map(|message| {
                            view! {
                                <div class="no-print mb-3">
                                    <Alert variant=AlertVariant::Danger>
                                        <strong>"エラー:"</strong>
                                        " "
                                        {message}
                                    </Alert>
                                </div>
                            }
                        })
                }}
                {move || {
                    let generation = session.generation.get();
                    let list_error = balances.with(|slot| match slot {
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
                            let rows = view.rows;
                            let found = rows.iter().find(|row| {
                                row.saved_id().is_some_and(|saved| saved == id)
                            });
                            match found {
                                None => {
                                    view! {
                                        <div class="asset-panel-padded">
                                            <p class="text-center text-text-muted">
                                                "銘柄が見つかりません"
                                            </p>
                                        </div>
                                    }
                                        .into_any()
                                }
                                Some(row) => {
                                    let holding_view = crate::features::asset_balance::holdings::holding_view(row);
                                    let valuation = holding_valuation(&ValuationInput::from(&holding_view));
                                    let market_text = format_row_valuation_amount(valuation.market_value);
                                    let pnl_text = format_signed_row_valuation_amount(valuation.profit_loss.amount);
                                    let pnl_rate_text = format_signed_valuation_rate(valuation.profit_loss.rate, 1);
                                    let pnl_negative = is_negative_valuation(valuation.profit_loss.amount);
                                    let dividend = holding_dividend(
                                        &holding_view.code,
                                        holding_view.shares,
                                        holding_view.average_price,
                                        &dividends.get(),
                                    );
                                    let dividend_per_share = format_dividend_per_share(&dividend);
                                    let dividend_annual = format_dividend_annual(&dividend);
                                    let dividend_yield = format_dividend_yield(&dividend);
                                    let shares_text = format!("{}株", format_number_value(holding_view.shares));
                                    let average_price_text = format_unit_price(holding_view.average_price);
                                    let purchase_text = format_currency(holding_view.purchase);
                                    let current_price_text = format_unit_price(holding_view.current_price);
                                    let price_as_of = holding_view.price_as_of.clone();
                                    view! {
                                        <div class="asset-panel">
                                            <div class="border-b border-border p-4">
                                                <h2 class="text-lg font-bold text-text">
                                                    {holding_view.name.clone()}
                                                </h2>
                                                <p class="mt-1 text-sm text-text-muted">
                                                    {holding_view.code.clone()}
                                                </p>
                                            </div>
                                            <dl class="divide-y divide-border-faint">
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">現在値</dt>
                                                    <dd class="tabular-nums text-text">
                                                        {current_price_text}
                                                    </dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">価格基準日</dt>
                                                    <dd class="text-text">
                                                        {price_as_of
                                                            .map_or("—".to_string(), |d| d.replace('-', "/"))}
                                                    </dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">口座</dt>
                                                    <dd class="text-text">
                                                        {holding_view
                                                            .account
                                                            .clone()
                                                            .map(|value| {
                                                                view! { <AccountBadge label=value /> }
                                                                    .into_any()
                                                            })
                                                            .unwrap_or_else(|| "—".into_any())}
                                                    </dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">数量</dt>
                                                    <dd class="tabular-nums text-text">{shares_text}</dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">取得単価</dt>
                                                    <dd class="tabular-nums text-text">{average_price_text}</dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">取得総額</dt>
                                                    <dd class="tabular-nums text-text">{purchase_text}</dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">評価額</dt>
                                                    <dd class="tabular-nums font-bold text-text">{market_text}</dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">評価損益</dt>
                                                    <dd>
                                                        <Amount
                                                            class="tabular-nums"
                                                            text=pnl_text
                                                            negative=pnl_negative
                                                        />
                                                        <span class="ml-1 text-sm text-text-muted tabular-nums">
                                                            {pnl_rate_text}
                                                        </span>
                                                    </dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">1株配当</dt>
                                                    <dd class="tabular-nums text-text">{dividend_per_share}</dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">年間配当参考額</dt>
                                                    <dd class="tabular-nums text-text">{dividend_annual}</dd>
                                                </div>
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">取得額基準利回り</dt>
                                                    <dd class="tabular-nums text-text">{dividend_yield}</dd>
                                                </div>
                                            </dl>
                                            <div class="border-t border-border p-4">
                                                <SecurityCodeLink
                                                    value=holding_view.code.clone()
                                                    variant=SecurityLinkVariant::Details
                                                />
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                }
                            }
                        }
                    }
                }}
            </div>
        </PageFrame>
    }
}
