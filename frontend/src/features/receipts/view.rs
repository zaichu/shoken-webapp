mod cards;
pub(crate) mod main_content;
mod panel;
mod pickers;
mod search_card;
mod summary;
mod table;
mod tabs;
mod workspace;

use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::{use_receipts_data, ReceiptRow, ReceiptsTab, TabState};
use crate::session::use_session;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::page_info_rail::PageInfoRail;
use crate::ui::workspace_shell::WorkspaceShell;
use leptos::prelude::*;
use main_content::display_rows_for;
use panel::ReceiptPanelContent;
use tabs::{scroll_tab_into_view, ReceiptsTabButton, TabPanel};
use workspace::utility_rail_id;

pub(crate) const TAB_IDS: [&str; 3] = ["dividend", "domesticstock", "mutualfund"];

#[component]
pub fn ReceiptsPage() -> impl IntoView {
    let store = use_receipts_data(use_session(), ReceiptsTab::Dividend);
    // 再訪ではストアのキャッシュがそのまま出るので、鮮度だけ裏で更新する
    store.revisit();
    let busy = store;
    let panels_store = store;
    let modal_store = store;
    // 他タブの取得進捗で workspace 全体を再生成するとパネル開閉などのローカル状態が
    // 巻き戻るため、分岐条件だけを memo 化して再生成を実際の切替時に限定する
    let workspace_store = store;
    let panels_loading = Memo::new(move |_| {
        workspace_store.auth_loading()
            || matches!(
                workspace_store.tab_state(workspace_store.active_tab.get()),
                TabState::Loading
            )
    });
    let tabs = store;
    let tabs_store = store;
    let panel_store = store;
    let badge_store = store;
    // 畳んでいるときだけ表の上に件数を出す(開いていればパネル内で確認できる)
    let badge_tab = Memo::new(move |_| badge_store.active_tab.get());
    let display = Memo::new(move |_| utility_display_rows(badge_store, badge_tab.get()));
    let search_state = badge_store.search;
    let filtered = Memo::new(move |_| {
        filter_receipts(
            badge_tab.get(),
            &display.get(),
            &search_state.with(|state| state.query.clone()),
        )
    });
    let total = Signal::derive(move || display.get().len());
    // クリックとキー操作の両経路をカバーするため select_tab ではなく active_tab の変化に追従する
    Effect::new(move |_| scroll_tab_into_view(tabs.active_tab.get()));

    view! {
        <h1 class="sr-only">"取引明細"</h1>
        <div
            class="mt-0"
            aria-busy=move || {
                if busy.auth_loading()
                    || busy.any_tab_fetching()
                    || ReceiptsTab::ALL.iter().any(|tab| {
                        let state = busy.csv_state(*tab);
                        state.saving || state.deleting
                    }) {
                    "true"
                } else {
                    "false"
                }
            }
        >
            <div data-testid="receipts-workspace">
                <WorkspaceShell
                    workspace_testid="receipt-workspace"
                    rail_testid="receipt-utility-rail"
                    main_testid="receipt-main-stage"
                    panel_id=Signal::derive(move || utility_rail_id(panel_store.active_tab.get()))
                    toggle_testid="receipt-utility-toggle"
                    panel_open=Signal::derive(move || panel_store.utility_rail_open.get())
                    on_toggle=Callback::new(move |_| panel_store.toggle_utility_rail())
                    on_close=Callback::new(move |_| {
                        if panel_store.utility_rail_open.get_untracked() {
                            panel_store.toggle_utility_rail();
                        }
                    })
                    panel=view! { <ReceiptPanelContent store=store /> }.into_any()
                    right_rail=move || {
                        let is_filtered = !panel_store.search.get().is_default();
                        let filter_labels = panel_store.filter_labels(panel_store.active_tab.get());
                        view! {
                            <PageInfoRail
                                links=vec![
                                    ("#receipts-summary", "集計情報"),
                                    ("#receipts-list", "一覧"),
                                ]
                                applied=is_filtered.then(|| {
                                    filter_labels
                                        .into_iter()
                                        .map(|(label, value)| {
                                            (label, Signal::derive(move || value.clone()))
                                        })
                                        .collect()
                                })
                                on_clear=Some(Callback::new(move |_| panel_store.clear_search()))
                            />
                        }.into_any()
                    }
                >
                    <div class="flex items-center justify-between gap-3">
                        <nav class="no-print" aria-label="取引明細タブ">
                            <div
                                class="receipts-tab-list"
                                role="tablist"
                            >
                                {ReceiptsTab::ALL
                                    .iter()
                                    .copied()
                                    .map(|tab| {
                                        view! { <ReceiptsTabButton store=tabs_store tab=tab /> }
                                    })
                                    .collect_view()}
                            </div>
                        </nav>
                        {move || {
                            badge_store.utility_filter_badge_visible().then(|| {
                                view! {
                                    <div
                                        class="shrink-0 no-print"
                                        data-testid="receipt-utility-toggle-bar"
                                    >
                                        <Badge variant=BadgeVariant::Accent>
                                            {format!(
                                                "絞り込み中 {} / {} 件",
                                                filtered.get().len(),
                                                total.get(),
                                            )}
                                        </Badge>
                                    </div>
                                }
                            })
                        }}
                    </div>
                    {move || {
                        let workspace = panels_store;
                        // 読み込み中も全タブの tabpanel を出す。非選択タブの aria-controls が
                        // 存在しない要素を指すと tab/tabpanel の関係で axe が critical になる
                        let loading = panels_loading.get();
                        ReceiptsTab::ALL
                            .iter()
                            .copied()
                            .map(|tab| {
                                view! { <TabPanel store=workspace tab=tab loading=loading /> }
                            })
                            .collect_view()
                    }}
                </WorkspaceShell>
            </div>
            {move || {
                let tab = modal_store.active_tab.get();
                if !modal_store.csv_state(tab).show_delete_confirm {
                    return ().into_any();
                }
                let count = modal_store.count(tab);
                let deleting_store = modal_store;
                let deleting = Memo::new(move |_| deleting_store.csv_state(tab).deleting);
                let confirm = modal_store;
                let cancel = modal_store;
                view! {
                    <ConfirmDeleteModal
                        title=format!("{}データの全件削除", tab.label())
                        description=format!("【{}】のデータをすべて削除します。", tab.label())
                        item_count=count
                        confirm_label="削除する"
                        loading=deleting
                        on_confirm=move || confirm.confirm_delete_all(tab)
                        on_cancel=move || cancel.close_delete_confirm(tab)
                    />
                }
                    .into_any()
            }}
        </div>
    }
}

#[cfg(test)]
mod tests;

pub(crate) fn utility_display_rows(
    store: crate::features::receipts::ReceiptsStore,
    tab: ReceiptsTab,
) -> Vec<ReceiptRow> {
    match store.tab_state(tab) {
        TabState::Ready(data) => display_rows_for(store, tab, data.rows),
        _ => Vec::new(),
    }
}
