mod asset_table;
mod composition_page;
mod csv_page;
mod holding_detail_page;
mod holdings_list;
mod main_content;
mod mobile_bar;
mod summary;

#[cfg(test)]
#[path = "view/tests/suite.rs"]
mod tests;

pub use composition_page::AssetBalanceCompositionPage;
pub use csv_page::{AssetBalanceCsvDeletePage, AssetBalanceCsvPage};
pub use holding_detail_page::AssetBalanceHoldingPage;

use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::features::asset_balance::store::csv::{AssetBalanceCsvStore, resolve_asset_balance};
use crate::features::asset_balance::store::{
    BalanceSlot, DataOps, has_current_balances, load_asset_balances,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::{SessionStore, use_session};
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;
use main_content::AssetBalanceMainContent;
use mobile_bar::AssetMobileActionBar;

#[derive(Clone, Copy)]
pub(crate) struct AssetBalanceState {
    balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    search_query: RwSignal<String>,
    search_count: RwSignal<(usize, usize)>,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
    data_ops: RwSignal<DataOps>,
    csv: AssetBalanceCsvStore,
}

fn use_asset_balance_state(session: SessionStore) -> AssetBalanceState {
    let states = use_context::<crate::app::PageStates>().unwrap_or_default();
    crate::app::page_state(states.asset_balance, || build_asset_balance_state(session))
}

fn build_asset_balance_state(session: SessionStore) -> AssetBalanceState {
    let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
    let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
    let lookup = RwSignal::new(AssetBalanceLookupStore::new());
    let search_query = RwSignal::new(String::new());
    let reset_query = search_query;
    let search_count = RwSignal::new((0usize, 0usize));
    let sort_key = RwSignal::new(SortKey::MarketValue);
    let sort_order = RwSignal::new(SortOrder::Desc);
    let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
    let csv_store = AssetBalanceCsvStore::new(session, balances, dividends, lookup, data_ops);
    let csv_slot = csv_store.csv;
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            lookup.update(|store| store.clear());
            balances.set(None);
            dividends.set(DividendMaps::default());
            data_ops.update(DataOps::reset);
            reset_query.set(String::new());
            search_count.set((0, 0));
            return;
        }
        lookup.update(|store| store.clear_if_stale(generation));
        if has_current_balances(&balances.get_untracked(), generation) {
            return;
        }
        load_asset_balances(
            session, generation, balances, dividends, lookup, csv_slot, data_ops,
        );
    });
    AssetBalanceState {
        balances,
        dividends,
        lookup,
        search_query,
        search_count,
        sort_key,
        sort_order,
        data_ops,
        csv: csv_store,
    }
}

#[component]
pub fn AssetBalancePage() -> impl IntoView {
    let session = use_session();
    let render_session = session;
    let busy_session = session;
    let rail_session = session;
    let state = use_asset_balance_state(session);
    let balances = state.balances;
    let dividends = state.dividends;
    let lookup = state.lookup;
    let search_query = state.search_query;
    let data_ops = state.data_ops;
    let busy_ops = data_ops;
    let csv_slot = state.csv.csv;
    let resolved_view = move || {
        let generation = rail_session.generation.get();
        balances.with(|slot| resolve_asset_balance(generation, slot))
    };
    let search_count = state.search_count;
    let action_bar = view! {
        {move || {
            resolved_view()
                .is_some_and(|view| !view.rows.is_empty())
                .then(|| {
                    view! {
                        <AssetMobileActionBar
                            query=search_query
                            count_text=Signal::derive(move || {
                                if search_query.get().is_empty() {
                                    String::new()
                                } else {
                                    let (filtered, total) = search_count.get();
                                    format!("検索結果 {filtered} / {total}件")
                                }
                            })
                            sort_key=state.sort_key
                            sort_order=state.sort_order
                        />
                    }
                })
        }}
    }
    .into_any();
    if session.user.get_untracked().is_some()
        && has_current_balances(
            &balances.get_untracked(),
            session.generation.get_untracked(),
        )
        && !data_ops.with_untracked(|ops| ops.list_fetch_in_flight())
    {
        load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            csv_slot,
            data_ops,
        );
    }
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
    let pc_subtitle = move || resolved_view().map(|view| summary::list_subtitle(&view));
    let title_row: Children = Box::new(move || {
        view! {
            <span class="hidden self-center text-sm font-normal text-text-muted lg:inline">
                {move || pc_subtitle().unwrap_or_default()}
            </span>
        }
        .into_any()
    });
    let csv_link: Children = Box::new(move || {
        view! {
            <span class="flex items-center gap-2">
                <a class="page-secondary-link-sm hidden lg:inline-flex" href="/assetbalance/composition">"構成比を見る"</a>
                <a class="page-secondary-link-sm" href="/assetbalance/csv">"CSV管理"</a>
            </span>
        }
        .into_any()
    });
    view! {
        <PageFrame title="資産管理" full_bleed=true title_row=title_row action_bar=action_bar action_bar_mobile_only=true links=csv_link>
        <div
            class="flex flex-1 flex-col lg:mt-2 lg:flex-none"
            data-testid="assetbalance-workspace"
            aria-busy=move || {
                if !busy_session.loaded.get()
                    || busy_ops.with(|ops| !ops.inflight.is_empty())
                    || (busy_session.user.get().is_some()
                        && balances
                            .get()
                            .filter(|(cached, _)| {
                                *cached == busy_session.generation.get()
                            })
                            .is_none()) {
                    "true"
                } else {
                    "false"
                }
            }
        >
            <div class="content-gutter flex flex-1 flex-col gap-0 lg:gap-3.5" data-testid="assetbalance-main-stage">
                    {move || {
                        data_ops
                            .with(|ops| ops.refresh_error.clone())
                            .map(|message| {
                                view! {
                                    <div class="no-print px-4 lg:px-0">
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
                        resolved_view().and_then(|view| view.warning).map(|message| {
                            view! {
                                <div class="no-print px-4 lg:px-0">
                                    <Alert variant=AlertVariant::Warning>
                                        <strong>"注意:"</strong>
                                        " "
                                        <span data-testid="truncated-list-warning">{message}</span>
                                    </Alert>
                                </div>
                            }
                        })
                    }}
                    {move || {
                        let generation = render_session.generation.get();
                        let list_error = balances.with(|slot| match slot {
                            Some((cached, Err(message))) if *cached == generation => {
                                Some(message.clone())
                            }
                            _ => None,
                        });
                        let resolved = resolved_view();
                        match (resolved, list_error) {
                            (None, _) => {
                                view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
                                    .into_any()
                            }
                            (Some(_), Some(message)) => {
                                view! {
                                    <ListLoadError message=message on_retry=reload />
                                }
                                    .into_any()
                            }
                            (Some(resolved), None) => {
                                view! {
                                    <AssetBalanceMainContent
                                        rows=resolved.rows
                                        fallback_totals=resolved.fallback_totals
                                        search_query=search_query
                                        search_count=search_count
                                        sort_key=state.sort_key
                                        sort_order=state.sort_order
                                        dividends=dividends
                                        lookup=lookup
                                        generation=generation
                                    />
                                }
                                    .into_any()
                            }
                        }
                    }}
            </div>
        </div>
        </PageFrame>
    }
}
