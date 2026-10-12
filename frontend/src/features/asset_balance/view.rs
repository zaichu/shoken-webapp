mod cards;
mod chart;
mod csv_page;
mod main_content;
mod palette;
mod summary;

#[cfg(test)]
#[path = "view/tests/suite.rs"]
mod tests;

pub use csv_page::{AssetBalanceCsvDeletePage, AssetBalanceCsvPage};

use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::store::csv::{AssetBalanceCsvStore, resolve_asset_balance};
use crate::features::asset_balance::store::{
    BalanceSlot, DataOps, has_current_balances, load_asset_balances,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::{SessionStore, use_session};
use crate::ui::list_action_bar::ListActionBar;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;
use main_content::AssetBalanceMainContent;

#[derive(Clone, Copy)]
pub(crate) struct AssetBalanceState {
    balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    search_query: RwSignal<String>,
    // 操作帯の「N / M件」表示用。一覧側で絞り込むたびに書き直す
    search_count: RwSignal<(usize, usize)>,
    // ページ側に持ち上げた show_all はアンマウントで破棄されないため、グラフを描かない分岐では false に戻す
    show_all: RwSignal<bool>,
    // 取得額構成比の開閉もページ側に持ち、一覧の再描画で状態が消えないようにする
    composition_open: RwSignal<bool>,
    data_ops: RwSignal<DataOps>,
    csv: AssetBalanceCsvStore,
}

// ページ遷移でビューを作り直しても一覧・検索・取得中状態を失わないよう、App コンテキストのスロットに保持する
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
    let show_all = RwSignal::new(false);
    let composition_open = RwSignal::new(false);
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
            show_all.set(false);
            composition_open.set(false);
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
        show_all,
        composition_open,
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
    let show_all = state.show_all;
    let composition_open = state.composition_open;
    let data_ops = state.data_ops;
    let busy_ops = data_ops;
    let csv_slot = state.csv.csv;
    let resolved_view = move || {
        let generation = rail_session.generation.get();
        balances.with(|slot| resolve_asset_balance(generation, slot))
    };
    let search_count = state.search_count;
    // 一覧が空のときは絞り込みの対象がないので操作帯自体を出さない
    let action_bar = view! {
        {move || {
            resolved_view()
                .is_some_and(|view| !view.rows.is_empty())
                .then(|| {
                    view! {
                        <ListActionBar
                            query=search_query
                            placeholder="銘柄名・コードで絞り込み"
                            count_text=Signal::derive(move || {
                                if search_query.get().is_empty() {
                                    String::new()
                                } else {
                                    let (filtered, total) = search_count.get();
                                    format!("検索結果 {filtered} / {total}件")
                                }
                            })
                            input_testid="asset-search-input"
                            clear_testid="asset-search-clear"
                            count_testid="asset-search-count"
                        />
                    }
                })
        }}
    }
    .into_any();
    // 再訪では表示済みの一覧を消さず裏で取り直す(初回・未キャッシュは Effect が担う)
    // 前の取得が残っている往復では要求を重ねない
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
    let mobile_csv_link: Children = Box::new(move || {
        view! {
            <a class="page-back-link" href="/assetbalance/csv">"CSV管理"</a>
        }
        .into_any()
    });
    let pc_csv_link: Children = Box::new(move || {
        view! {
            <a class="page-back-link" href="/assetbalance/csv">"CSV管理"</a>
        }
        .into_any()
    });
    view! {
        <PageFrame title="資産管理" full_bleed=true action_bar=action_bar mobile_links=mobile_csv_link pc_links=pc_csv_link>
        <div
            class="mt-2 flex flex-1 flex-col"
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
            <div class="page-gutter flex flex-1 flex-col gap-3" data-testid="assetbalance-main-stage">
                    // 再取得の失敗は一覧を消さずに知らせる
                    {move || {
                        data_ops
                            .with(|ops| ops.refresh_error.clone())
                            .map(|message| {
                                view! {
                                    <div class="no-print">
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
                                <div class="no-print">
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
                                show_all.set(false);
                                view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
                                    .into_any()
                            }
                            (Some(_), Some(message)) => {
                                show_all.set(false);
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
                                        dividends=dividends
                                        show_all=show_all
                                        composition_open=composition_open
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
