use super::use_asset_balance_state;
use crate::features::asset_balance::format::{format_currency, format_fixed_percent};
use crate::features::asset_balance::holdings::HoldingView;
use crate::features::asset_balance::portfolio::chart_plan;
use crate::features::asset_balance::store::filtered_portfolio;
use crate::session::use_session;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

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
    view! {
        <PageFrame
            title="構成比"
            back_href="/assetbalance"
            back_label="資産管理に戻る"
        >
            <div class="page-gutter flex flex-1 flex-col py-4">
                {move || {
                    let view = resolved();
                    match view {
                        None => view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }.into_any(),
                        Some(view) if view.rows.is_empty() => {
                            view! {
                                <div class="rounded-lg border border-border bg-surface p-6">
                                    <p class="text-center text-text-muted">
                                        "データがありません"
                                    </p>
                                </div>
                            }
                                .into_any()
                        }
                        Some(view) => {
                            let generation = session.generation.get();
                            let filtered = filtered_portfolio(&view.rows, "", lookup, generation);
                            let views = filtered.views;
                            let total_purchase: f64 = views.iter().map(|v| v.purchase).sum();
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
                                <div class="rounded-lg border border-border bg-surface">
                                    <div class="border-b border-border p-4">
                                        <p class="text-sm text-text-muted">
                                            "対象: 保存済みの保有全体"
                                        </p>
                                        <p class="mt-1 text-sm">
                                            "基準(取得総額): "
                                            <strong class="tabular-nums">
                                                {format_currency(total_purchase)}
                                            </strong>
                                        </p>
                                    </div>
                                    {if total_purchase == 0.0 {
                                        view! {
                                            <div class="p-6 text-center text-text-muted">
                                                "算出不可"
                                            </div>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <ul class="divide-y divide-border-faint">
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
                                                        view! {
                                                            <li class="px-4 py-3">
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
                                                                        style=format!("width: {bar_width}")
                                                                    />
                                                                </div>
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
