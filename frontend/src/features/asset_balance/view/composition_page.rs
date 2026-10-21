use super::use_asset_balance_state;
use crate::features::asset_balance::format::format_amount;
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::portfolio::chart_plan;
use crate::features::asset_balance::store::{filtered_portfolio, load_asset_balances};
use crate::session::use_session;
use crate::ui::format::{format_currency, format_fixed_percent};
use crate::ui::labels::TOTAL_ACQUISITION;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

/// 構成比の母数。比率と同じく絶対値合計を使う。負の取得額があると
/// 符号付き合計と比率が食い違うため
pub(crate) fn composition_basis(views: &[HoldingView]) -> f64 {
    views.iter().map(|v| v.purchase.abs()).sum()
}

#[derive(Clone)]
struct ChartItem {
    view: HoldingView,
    percentage: Option<f64>,
}

#[component]
pub fn AssetBalanceCompositionPage() -> impl IntoView {
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
        load_asset_balances(
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
            title="構成比"
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
                        Some(view) if view.rows.is_empty() => {
                            view! {
                                <div class="asset-panel-padded">
                                    <p class="text-center text-text-muted">
                                        "データがありません"
                                    </p>
                                </div>
                            }
                                .into_any()
                        }
                        Some(view) => {
                            let generation = session.generation.get();
                            let filtered = filtered_portfolio(
                                &view.rows,
                                &crate::features::asset_balance::fund::AssetSearch::default(),
                                lookup,
                                generation,
                            );
                            let views = filtered.views;
                            let total_purchase = composition_basis(&views);
                            let chart_values: Vec<f64> = views.iter().map(|v| v.purchase).collect();
                            let plan = chart_plan(&chart_values);
                            let items: Vec<ChartItem> = plan
                                .order
                                .iter()
                                .zip(&plan.percentages)
                                .map(|(&index, &percentage)| ChartItem {
                                    view: views[index].clone(),
                                    percentage,
                                })
                                .collect();
                            view! {
                                <div class="asset-panel">
                                    <div class="border-b border-border p-4">
                                        <p class="text-sm text-text-muted">
                                            "対象: 保存済みの保有全体"
                                        </p>
                                        <p class="mt-1 text-sm">
                                            "基準("
                                            {TOTAL_ACQUISITION}
                                            "): "
                                            <strong class="tabular-nums">
                                                {format_amount(total_purchase)}
                                            </strong>
                                        </p>
                                    </div>
                                    {if total_purchase <= 0.0 {
                                        view! {
                                            <div class="p-6 text-center text-text-muted">
                                                "算出不可"
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <ul
                                                class="divide-y divide-border-faint"
                                                data-testid="asset-composition-list"
                                            >
                                                {items
                                                    .into_iter()
                                                    .map(|item| {
                                                        let name = item.view.name.clone();
                                                        let code = item.view.code.clone();
                                                        let purchase = item.view.purchase;
                                                        let percentage = item.percentage
                                                            .map_or("—".to_string(), |p| {
                                                                format_fixed_percent(p, 1)
                                                            });
                                                        let bar_width = item.percentage
                                                            .map_or("0%".to_string(), |p| {
                                                                format!("{}%", p.min(100.0))
                                                            });
                                                        let inner = view! {
                                                            <>
                                                                <div class="flex items-baseline justify-between gap-2">
                                                                    <span class="min-w-0 truncate font-medium text-text">
                                                                        {name}
                                                                    </span>
                                                                    <span class="shrink-0 tabular-nums text-text">
                                                                        {format_currency(purchase)}
                                                                        " "
                                                                        <span class="text-text-muted">
                                                                            {percentage}
                                                                        </span>
                                                                    </span>
                                                                </div>
                                                                <div class="mt-1 text-sm text-text-muted">
                                                                    {code}
                                                                </div>
                                                                <div class="mt-2 h-1.5 w-full rounded-full bg-surface-raised">
                                                                    <div
                                                                        class="h-full rounded-full bg-accent"
                                                                        style:width=bar_width
                                                                    />
                                                                </div>
                                                            </>
                                                        };
                                                        // 一覧と同じく保存済みレコード ID で詳細を開く
                                                        view! {
                                                            <li>
                                                                {match &item.view.id {
                                                                    Some(id) => view! {
                                                                        <a
                                                                            href=format!("/assetbalance/holdings/{id}")
                                                                            class="asset-comp-link"
                                                                        >
                                                                            {inner}
                                                                        </a>
                                                                    }
                                                                        .into_any(),
                                                                    None => view! { <div class="asset-comp-link">{inner}</div> }
                                                                        .into_any(),
                                                                }}
                                                            </li>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </ul>
                                        }
                                            .into_any()
                                    }}
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
