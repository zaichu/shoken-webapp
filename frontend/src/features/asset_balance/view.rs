mod cards;
mod chart;
mod csv_section;
mod main_content;
mod palette;
mod panel;
mod search_card;
mod summary;

#[cfg(test)]
mod tests;

use crate::features::asset_balance::csv_store::{resolve_asset_balance, AssetBalanceCsvStore};
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::store::{
    has_current_balances, load_asset_balances, BalanceSlot, DataOps,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::{use_session, SessionStore};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::elements::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use crate::ui::workspace_shell::{workspace_panel_default_open, WorkspaceShell};
use leptos::prelude::*;
use main_content::AssetBalanceMainContent;
use panel::AssetBalancePanelContent;
use std::cell::RefCell;

#[derive(Clone, Copy)]
struct AssetBalanceState {
    balances: RwSignal<BalanceSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    search_query: RwSignal<String>,
    // ページ側に持ち上げた show_all はアンマウントで破棄されないため、グラフを描かない分岐では false に戻す
    show_all: RwSignal<bool>,
    data_ops: RwSignal<DataOps>,
    csv: AssetBalanceCsvStore,
    panel_open: RwSignal<bool>,
}

thread_local! {
    // ページ遷移でビューを作り直しても一覧・検索・取得中状態を失わないよう、アプリ寿命のオーナーに作る。
    // Owner::new() は現オーナーの子として登録されページと一緒に破棄されるため、AppOwner の子を使う
    static PAGE_STATE: RefCell<Option<(Owner, AssetBalanceState)>> = const { RefCell::new(None) };
}

fn use_asset_balance_state(session: SessionStore) -> AssetBalanceState {
    let app_owner = use_context::<crate::app::AppOwner>()
        .map(|app| app.0)
        .unwrap_or_default();
    PAGE_STATE.with(|cell| {
        if let Some((_, state)) = cell.borrow().as_ref() {
            return *state;
        }
        let owner = app_owner.child();
        let state = owner.with(|| build_asset_balance_state(session));
        *cell.borrow_mut() = Some((owner, state));
        state
    })
}

fn build_asset_balance_state(session: SessionStore) -> AssetBalanceState {
    let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
    let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
    let lookup = RwSignal::new(AssetBalanceLookupStore::new());
    let search_query = RwSignal::new(String::new());
    let reset_query = search_query;
    let show_all = RwSignal::new(false);
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
            show_all.set(false);
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
        show_all,
        data_ops,
        csv: csv_store,
        panel_open: RwSignal::new(workspace_panel_default_open()),
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
    let data_ops = state.data_ops;
    let csv_store = state.csv;
    let busy_csv = csv_store;
    let view_csv = csv_store;
    let modal_csv = csv_store;
    let rail_csv = csv_store;
    let busy_ops = data_ops;
    let csv_slot = csv_store.csv;
    let disabled_csv = csv_store;
    let panel_open = state.panel_open;
    let csv_input_disabled = Memo::new(move |_| disabled_csv.csv_input_disabled());
    let resolved_view = move || {
        let generation = rail_session.generation.get();
        let state = rail_csv.csv_state();
        balances.with(|slot| resolve_asset_balance(generation, slot, &state))
    };
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
    view! {
        <h1 class="sr-only">"資産管理"</h1>
        <div
            class="mt-2"
            aria-busy=move || {
                if !busy_session.loaded.get()
                    || busy_csv.csv_busy()
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
            <WorkspaceShell
                workspace_testid="assetbalance-workspace"
                rail_testid="assetbalance-utility-rail"
                main_testid="assetbalance-main-stage"
                panel_id=Signal::derive(|| "assetbalance-utility-panel".to_string())
                toggle_testid="asset-utility-toggle"
                panel_open=Signal::derive(move || panel_open.get())
                on_toggle=Callback::new(move |_| panel_open.update(|open| *open = !*open))
                on_close=Callback::new(move |_| panel_open.set(false))
                panel=view! {
                    <AssetBalancePanelContent
                        view_csv=view_csv
                        search_query=search_query
                        rows=move || {
                            resolved_view().map(|r| r.rows).unwrap_or_default()
                        }
                        facets=move || resolved_view().and_then(|r| r.facets)
                        has_csv_file=move || {
                            resolved_view().map(|r| r.has_csv_file).unwrap_or(false)
                        }
                        warning=move || resolved_view().and_then(|r| r.warning)
                    />
                }
                    .into_any()
            >
                    // パネル(ドロワー)が閉じていても見えるよう、CSV/再取得の失敗はメイン列に出す
                    {move || {
                        data_ops
                            .with(|ops| ops.refresh_error.clone())
                            .or_else(|| csv_store.csv_state().error)
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
                        let generation = render_session.generation.get();
                        let state = view_csv.csv_state();
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
                            (Some(resolved), Some(message)) => {
                                let preview = (state.previewing || !resolved.rows.is_empty())
                                    .then(|| {
                                        view! {
                                            <div class="mt-4">
                                                <AssetBalanceMainContent
                                                    state=state
                                                    rows=resolved.rows
                                                    summary=resolved.summary
                                                    has_csv_file=resolved.has_csv_file
                                                    csv_input_disabled=csv_input_disabled
                                                    search_query=search_query
                                                    dividends=dividends
                                                    show_all=show_all
                                                    lookup=lookup
                                                    generation=generation
                                                />
                                            </div>
                                        }
                                    });
                                if preview.is_none() {
                                    show_all.set(false);
                                }
                                view! {
                                    <ListLoadError message=message on_retry=reload />
                                    {preview}
                                }
                                    .into_any()
                            }
                            (Some(resolved), None) => {
                                view! {
                                    <AssetBalanceMainContent
                                        state=state
                                        rows=resolved.rows
                                        summary=resolved.summary
                                        has_csv_file=resolved.has_csv_file
                                        csv_input_disabled=csv_input_disabled
                                        search_query=search_query
                                        dividends=dividends
                                        show_all=show_all
                                        lookup=lookup
                                        generation=generation
                                    />
                                }
                                    .into_any()
                            }
                        }
                    }}
            </WorkspaceShell>
            {move || {
                if !modal_csv.csv_state().show_delete_confirm {
                    return ().into_any();
                }
                let count = modal_csv.db_count();
                let deleting_csv = modal_csv;
                let deleting = Memo::new(move |_| deleting_csv.csv_state().deleting);
                let confirm = modal_csv;
                let cancel = modal_csv;
                view! {
                    <ConfirmDeleteModal
                        title="資産管理データの全件削除".to_string()
                        description="保存された資産管理データをすべて削除します。".to_string()
                        item_count=count
                        confirm_label="削除する"
                        loading=deleting
                        on_confirm=move || confirm.confirm_delete_all()
                        on_cancel=move || cancel.close_delete_confirm()
                    />
                }
                    .into_any()
            }}
        </div>
    }
}
