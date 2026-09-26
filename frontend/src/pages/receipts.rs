mod cards;
mod groups;
mod main_content;
mod pickers;
mod search_card;
mod summary;
mod table;
mod tabs;
mod workspace;

use crate::components::confirm_modal::ConfirmDeleteModal;
use crate::components::ui::{ListSkeleton, PageHeader};
use crate::receipts::{use_receipts_data, ReceiptsTab, TabState};
use crate::session::use_session;
use leptos::prelude::*;
use tabs::{scroll_tab_into_view, TabButton, TabPanel};

pub(crate) const TAB_IDS: [&str; 3] = ["dividend", "domesticstock", "mutualfund"];

#[component]
pub fn ReceiptsPage() -> impl IntoView {
    let store = use_receipts_data(use_session(), ReceiptsTab::Dividend);
    let busy = store.clone();
    let panels_store = store.clone();
    let modal_store = store.clone();
    // 他タブの取得進捗で workspace 全体を再生成するとレール開閉などのローカル状態が
    // 巻き戻るため、分岐条件だけを memo 化して再生成を実際の切替時に限定する
    let workspace_store = store.clone();
    let panels_loading = Memo::new(move |_| {
        workspace_store.auth_loading()
            || matches!(
                workspace_store.tab_state(workspace_store.active_tab.get()),
                TabState::Loading
            )
    });
    let tabs = store.clone();
    // クリックとキー操作の両経路をカバーするため select_tab ではなく active_tab の変化に追従する
    Effect::new(move |_| scroll_tab_into_view(tabs.active_tab.get()));

    view! {
        <PageHeader
            title="取引明細"
            eyebrow="Transactions"
            description="配当金・国内株式・投資信託の取引明細を管理します。"
        />
        <nav class="mb-2 no-print" aria-label="取引明細タブ">
            <div
                class="receipts-tab-list"
                role="tablist"
            >
                {ReceiptsTab::ALL
                    .iter()
                    .copied()
                    .map(|tab| {
                        view! { <TabButton store=store.clone() tab=tab /> }
                    })
                    .collect_view()}
            </div>
        </nav>
        <div
            class="mt-0"
            aria-busy=move || {
                busy.auth_loading()
                    || busy.any_tab_fetching()
                    || ReceiptsTab::ALL.iter().any(|tab| {
                        let state = busy.csv_state(*tab);
                        state.saving || state.deleting
                    })
            }
        >
            <div data-testid="receipts-workspace">
                {move || {
                    let workspace = panels_store.clone();
                    if panels_loading.get() {
                        view! { <ListSkeleton /> }.into_any()
                    } else {
                        view! {
                            {ReceiptsTab::ALL
                                .iter()
                                .copied()
                                .map(|tab| {
                                    view! { <TabPanel store=workspace.clone() tab=tab /> }
                                })
                                .collect_view()}
                        }
                            .into_any()
                    }
                }}
            </div>
            {move || {
                let tab = modal_store.active_tab.get();
                if !modal_store.csv_state(tab).show_delete_confirm {
                    return ().into_any();
                }
                let count = modal_store.count(tab);
                let deleting_store = modal_store.clone();
                let deleting = Memo::new(move |_| deleting_store.csv_state(tab).deleting);
                let confirm = modal_store.clone();
                let cancel = modal_store.clone();
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
