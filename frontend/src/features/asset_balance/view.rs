mod cards;
mod chart;
mod csv_section;
mod main_content;
mod rail;
mod search_card;
mod summary;

#[cfg(test)]
mod tests;

use crate::features::asset_balance::csv_store::{resolve_asset_balance, AssetBalanceCsvStore};
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::search::clear_search_query;
use crate::features::asset_balance::store::{load_asset_balances, BalanceSlot, DataOps};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::use_session;
use crate::ui::card::{Card, CardVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::elements::{ListLoadError, ListSkeleton, PageHeader};
use csv_section::AssetBalanceCsvSection;
use leptos::prelude::*;
use main_content::AssetBalanceMainContent;
use rail::AssetBalanceRailExtras;

#[component]
pub fn AssetBalancePage() -> impl IntoView {
    let session = use_session();
    let render_session = session;
    let busy_session = session;
    let rail_session = session;
    let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
    let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
    let lookup = RwSignal::new(AssetBalanceLookupStore::new());
    let search_query = RwSignal::new(String::new());
    let reset_query = search_query;
    // ページ側に持ち上げた show_all はアンマウントで破棄されないため、グラフを描かない分岐では false に戻す
    let show_all = RwSignal::new(false);
    let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
    let csv_store = AssetBalanceCsvStore::new(session, balances, dividends, lookup, data_ops);
    let busy_csv = csv_store;
    let view_csv = csv_store;
    let modal_csv = csv_store;
    let alert_csv = csv_store;
    let rail_csv = csv_store;
    let alert_ops = data_ops;
    let busy_ops = data_ops;
    let csv_slot = csv_store.csv;
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
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            lookup.update(|store| store.clear());
            balances.set(None);
            dividends.set(DividendMaps::default());
            data_ops.update(DataOps::reset);
            reset_query.set(clear_search_query());
            show_all.set(false);
            return;
        }
        lookup.update(|store| store.clear_if_stale(generation));
        if balances
            .get_untracked()
            .is_some_and(|(cached, _)| cached == generation)
        {
            return;
        }
        load_asset_balances(
            session, generation, balances, dividends, lookup, csv_slot, data_ops,
        );
    });
    view! {
        <PageHeader
            title="資産管理"
            eyebrow="Portfolio"
            description="保有している銘柄の一覧と評価額を確認できます。"
        />
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
            <div
                class="workspace-grid gap-4 xl:gap-5"
                data-testid="assetbalance-workspace"
            >
                // DOM 順は rail 先(キーボード・読み上げ順のため)、lg 以上は order で見た目を main 先に戻す
                <aside class="order-1 lg:order-2" data-testid="assetbalance-utility-rail">
                    <Card variant=CardVariant::Rail>
                        <AssetBalanceCsvSection store=view_csv />
                        {move || {
                            alert_ops
                                .with(|ops| ops.refresh_error.clone())
                                .or_else(|| alert_csv.csv_state().error)
                                .map(|message| {
                                    view! {
                                        <div class="px-5 py-4">
                                            <div
                                                class="rounded-lg border border-negative-border bg-negative-soft px-4 py-3 text-sm font-medium text-negative-strong"
                                                role="alert"
                                                aria-live="assertive"
                                            >
                                                <strong>"エラー:"</strong>
                                                " "
                                                {message}
                                            </div>
                                        </div>
                                    }
                                })
                        }}
                        {move || {
                            let generation = rail_session.generation.get();
                            let state = rail_csv.csv_state();
                            match balances
                                .with(|slot| resolve_asset_balance(generation, slot, &state))
                            {
                                None => ().into_any(),
                                Some(resolved) => {
                                    view! {
                                        <AssetBalanceRailExtras
                                            rows=resolved.rows
                                            facets=resolved.facets
                                            has_csv_file=resolved.has_csv_file
                                            warning=resolved.warning
                                            search_query=search_query
                                        />
                                    }
                                        .into_any()
                                }
                            }
                        }}
                    </Card>
                </aside>
                <div class="min-w-0 order-2 lg:order-1" data-testid="assetbalance-main-stage">
                    {move || {
                        let generation = render_session.generation.get();
                        let state = view_csv.csv_state();
                        let list_error = balances.with(|slot| match slot {
                            Some((cached, Err(message))) if *cached == generation => {
                                Some(message.clone())
                            }
                            _ => None,
                        });
                        let resolved = balances
                            .with(|slot| resolve_asset_balance(generation, slot, &state));
                        match (resolved, list_error) {
                            (None, _) => {
                                show_all.set(false);
                                view! { <ListSkeleton /> }.into_any()
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
                </div>
            </div>
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
