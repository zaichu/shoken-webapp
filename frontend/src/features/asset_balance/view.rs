mod all_list;
mod asset_table;
mod composition_page;
mod csv_page;
mod csv_preview;
mod day_change;
mod fund_detail_page;
mod fund_table;
mod holding_detail_page;
mod holdings_list;
mod kind_tabs;
mod main_content;
mod search_bar;
mod sort_bar;
mod sort_header;
mod summary;

#[cfg(test)]
#[path = "view/tests/suite.rs"]
mod tests;

pub use composition_page::AssetBalanceCompositionPage;
pub use csv_page::AssetBalanceCsvPage;
pub use csv_preview::{AssetBalanceCsvPreview, FundBalanceCsvPreview};
pub use fund_detail_page::AssetFundHoldingPage;
pub use holding_detail_page::AssetBalanceHoldingPage;

use crate::features::asset_balance::fund::{AssetKind, AssetSearch, FundSortKey};
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::features::asset_balance::store::csv::{
    AssetBalanceCsvStore, resolve_asset_balance, resolve_fund_balance,
};
use crate::features::asset_balance::store::{
    BalanceSlot, DataOps, FundSlot, has_current_balances, load_asset_balances, load_fund_balances,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::{SessionStore, use_session};
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;
use main_content::AssetBalanceMainContent;
use sort_bar::AssetMobileSortBar;

#[derive(Clone, Copy)]
pub(crate) struct AssetBalanceState {
    balances: RwSignal<BalanceSlot>,
    funds: RwSignal<FundSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    search: RwSignal<AssetSearch>,
    search_count: RwSignal<(usize, usize)>,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
    fund_sort_key: RwSignal<FundSortKey>,
    fund_sort_order: RwSignal<SortOrder>,
    kind: RwSignal<AssetKind>,
    data_ops: RwSignal<DataOps>,
    csv: AssetBalanceCsvStore,
}

fn use_asset_balance_state(session: SessionStore) -> AssetBalanceState {
    let states = use_context::<crate::app::PageStates>().unwrap_or_default();
    crate::app::page_state(states.asset_balance, || build_asset_balance_state(session))
}

fn build_asset_balance_state(session: SessionStore) -> AssetBalanceState {
    let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
    let funds: RwSignal<FundSlot> = RwSignal::new(None);
    let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
    let lookup = RwSignal::new(AssetBalanceLookupStore::new());
    let search = RwSignal::new(AssetSearch::default());
    let reset_search = search;
    let search_count = RwSignal::new((0usize, 0usize));
    let sort_key = RwSignal::new(SortKey::MarketValue);
    let sort_order = RwSignal::new(SortOrder::Desc);
    // 初期の並び順は設定から。設定が変わったら並び順に反映する(ユーザーの手動選択は設定変更まで保持)
    let configured_sort = crate::settings::try_use_settings().asset_sort;
    Effect::new(move |_| {
        let sort = configured_sort.get();
        sort_key.set(sort.key);
        sort_order.set(sort.order);
    });
    let fund_sort_key = RwSignal::new(FundSortKey::MarketValue);
    let fund_sort_order = RwSignal::new(SortOrder::Desc);
    let kind = RwSignal::new(AssetKind::Stock);
    let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
    let csv_store =
        AssetBalanceCsvStore::new(session, balances, funds, dividends, lookup, data_ops);
    let csv_slot = csv_store.csv;
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            lookup.update(|store| store.clear());
            balances.set(None);
            funds.set(None);
            dividends.set(DividendMaps::default());
            data_ops.update(DataOps::reset);
            reset_search.set(AssetSearch::default());
            search_count.set((0, 0));
            return;
        }
        lookup.update(|store| store.clear_if_stale(generation));
        if !has_current_balances(&balances.get_untracked(), generation) {
            load_asset_balances(
                session, generation, balances, dividends, lookup, csv_slot, data_ops,
            );
        }
        if !matches!(funds.get_untracked(), Some((cached, _)) if cached == generation) {
            load_fund_balances(session, generation, funds, data_ops);
        }
    });
    AssetBalanceState {
        balances,
        funds,
        dividends,
        lookup,
        search,
        search_count,
        sort_key,
        sort_order,
        fund_sort_key,
        fund_sort_order,
        kind,
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
    let funds = state.funds;
    let dividends = state.dividends;
    let lookup = state.lookup;
    let search = state.search;
    let data_ops = state.data_ops;
    let busy_ops = data_ops;
    let csv_slot = state.csv.csv;
    let resolved_view = move || {
        let generation = rail_session.generation.get();
        balances.with(|slot| resolve_asset_balance(generation, slot))
    };
    let resolved_fund_view = move || {
        let generation = rail_session.generation.get();
        funds.with(|slot| resolve_fund_balance(generation, slot))
    };
    let search_count = state.search_count;
    let sort_state = state;
    let action_bar = view! {
        {move || {
            let (stock_rows, fund_rows) = (
                resolved_view().map(|view| view.rows.len()).unwrap_or(0),
                resolved_fund_view().map(|view| view.rows.len()).unwrap_or(0),
            );
            let has_rows = match sort_state.kind.get() {
                AssetKind::Stock => stock_rows > 0,
                AssetKind::Fund => fund_rows > 0,
                AssetKind::All => stock_rows + fund_rows > 0,
            };
            has_rows.then(|| {
                match sort_state.kind.get() {
                    AssetKind::Stock => view! {
                        <AssetMobileSortBar
                            options=&SortKey::ALL
                            key_label=|key: SortKey| key.label()
                            key_value=|key: SortKey| key.value()
                            sort_key=sort_state.sort_key
                            sort_order=sort_state.sort_order
                        />
                    }
                    .into_any(),
                    AssetKind::Fund => view! {
                        <AssetMobileSortBar
                            options=&FundSortKey::ALL
                            key_label=|key: FundSortKey| key.label()
                            key_value=|key: FundSortKey| key.value()
                            sort_key=sort_state.fund_sort_key
                            sort_order=sort_state.fund_sort_order
                        />
                    }
                    .into_any(),
                    AssetKind::All => view! {
                        <AssetMobileSortBar
                            options=&FundSortKey::MERGED
                            key_label=|key: FundSortKey| key.label()
                            key_value=|key: FundSortKey| key.value()
                            sort_key=sort_state.fund_sort_key
                            sort_order=sort_state.fund_sort_order
                        />
                    }
                    .into_any(),
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
        load_fund_balances(session, session.generation.get_untracked(), funds, data_ops);
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
        funds.set(None);
        load_fund_balances(session, session.generation.get_untracked(), funds, data_ops);
    };
    let pc_subtitle = move || match sort_state.kind.get() {
        AssetKind::Stock => resolved_view().map(|view| summary::list_subtitle(&view)),
        AssetKind::Fund => resolved_fund_view().map(|view| summary::fund_subtitle(&view)),
        AssetKind::All => {
            summary::merged_subtitle(resolved_view().as_ref(), resolved_fund_view().as_ref())
        }
    };
    let title_row: Children = Box::new(move || {
        view! {
            <span
                class="hidden self-center text-sm font-normal text-text-muted lg:inline"
                data-testid="asset-subtitle"
            >
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
                <crate::features::csv_sync::CsvSyncButton />
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
                    <div class="no-print px-4 lg:px-0">
                        <crate::features::csv_sync::CsvSyncStatus />
                    </div>
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
                        resolved_fund_view().and_then(|view| view.warning).map(|message| {
                            view! {
                                <div class="no-print px-4 lg:px-0">
                                    <Alert variant=AlertVariant::Warning>
                                        <strong>"注意:"</strong>
                                        " "
                                        <span data-testid="fund-truncated-list-warning">{message}</span>
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
                        let resolved_fund = resolved_fund_view();
                        match (resolved, resolved_fund, list_error) {
                            (None, _, _) | (_, None, _) => {
                                view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
                                    .into_any()
                            }
                            (Some(_), _, Some(message)) => {
                                view! {
                                    <ListLoadError message=message on_retry=reload />
                                }
                                    .into_any()
                            }
                            (Some(resolved), Some(resolved_fund), None) => {
                                let stock_note = summary::stock_note(&resolved);
                                view! {
                                    <AssetBalanceMainContent
                                        stock_rows=resolved.rows
                                        fallback_totals=resolved.fallback_totals
                                        stock_note=stock_note
                                        fund_resolved=resolved_fund
                                        search=search
                                        search_count=search_count
                                        sort_key=state.sort_key
                                        sort_order=state.sort_order
                                        fund_sort_key=state.fund_sort_key
                                        fund_sort_order=state.fund_sort_order
                                        kind=state.kind
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
