use super::use_asset_balance_state;
use crate::api::dto::FundPriceSource;
use crate::features::asset_balance::format::{
    format_currency, format_row_valuation_amount, format_signed_row_valuation_amount,
    format_signed_valuation_rate, is_negative_valuation,
};
use crate::features::asset_balance::fund::fund_view;
use crate::features::asset_balance::model::format_number_value;
use crate::session::use_session;
use crate::ui::amount::Amount;
use crate::ui::badge::account_display_name;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

fn detail_row(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="flex justify-between px-4 py-3">
            <dt class="text-text-muted">{label}</dt>
            <dd class="tabular-nums text-text">{value}</dd>
        </div>
    }
}

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
            title="保有ファンド詳細"
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
                                row.saved_id().is_some_and(|saved| saved == id)
                            });
                            match found {
                                None => {
                                    view! {
                                        <div class="asset-panel-padded">
                                            <p class="text-center text-text-muted">
                                                "ファンドが見つかりません"
                                            </p>
                                        </div>
                                    }
                                        .into_any()
                                }
                                Some(row) => {
                                    let view = fund_view(row);
                                    let shares_text =
                                        format!("{}口", format_number_value(view.shares));
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
                                    let day_text = if view.nav_pending() {
                                        "更新待ち".to_string()
                                    } else {
                                        view.base_price_day_change
                                            .map(|value| {
                                                format_signed_row_valuation_amount(Some(value))
                                            })
                                            .unwrap_or_else(|| "—".to_string())
                                    };
                                    let month_text = view
                                        .base_price_month_change
                                        .map(|value| {
                                            format_signed_row_valuation_amount(Some(value))
                                        })
                                        .unwrap_or_else(|| "—".to_string());
                                    let total_return_text = view
                                        .total_return
                                        .map(|value| {
                                            format_signed_row_valuation_amount(Some(value))
                                        })
                                        .unwrap_or_else(|| "—".to_string());
                                    let account_text = match &view.course {
                                        Some(course) => {
                                            format!(
                                                "{} ・ {course}",
                                                account_display_name(&view.account)
                                            )
                                        }
                                        None => account_display_name(&view.account).to_string(),
                                    };
                                    let asof_text = view
                                        .price_as_of
                                        .map_or("—".to_string(), |d| d.replace('-', "/"));
                                    let asof_label = match view.price_source {
                                        FundPriceSource::Api => "基準価額の基準日",
                                        FundPriceSource::Csv => "CSV 取込日",
                                    };
                                    view! {
                                        <div class="asset-panel">
                                            <div class="border-b border-border p-4">
                                                <h2 class="text-lg font-bold text-text">
                                                    {view.name.clone()}
                                                </h2>
                                                <p class="mt-1 text-sm text-text-muted">
                                                    {account_text}
                                                </p>
                                            </div>
                                            <dl class="divide-y divide-border-faint">
                                                {detail_row("基準価額", base_price_text)}
                                                {detail_row("基準価額(前日比)", day_text)}
                                                {detail_row("基準価額(前月比)", month_text)}
                                                {detail_row(asof_label, asof_text)}
                                                {detail_row("保有口数", shares_text)}
                                                {detail_row("平均取得価額", average_price_text)}
                                                {detail_row("取得総額", purchase_text)}
                                                <div class="flex justify-between px-4 py-3">
                                                    <dt class="text-text-muted">評価額</dt>
                                                    <dd class="tabular-nums font-bold text-text">
                                                        {market_text}
                                                    </dd>
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
                                                {detail_row("トータルリターン", total_return_text)}
                                            </dl>
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
