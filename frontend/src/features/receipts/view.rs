mod cards;
mod groups;
mod main_content;
mod pickers;
mod search_card;
mod summary;
mod table;
mod tabs;
mod workspace;

use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::{use_receipts_data, ReceiptsTab, TabState};
use crate::session::use_session;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use leptos::prelude::*;
use main_content::display_rows_for;
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
    // 他タブの取得進捗で workspace 全体を再生成するとレール開閉などのローカル状態が
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
    let trigger_store = store;
    // 裏再取得の Ready→Ready で作り直さないよう対象タブの有無だけを memo 化する
    let trigger_tab = Memo::new(move |_| {
        let tab = trigger_store.active_tab.get();
        matches!(trigger_store.tab_state(tab), TabState::Ready(_)).then_some(tab)
    });
    // クリックとキー操作の両経路をカバーするため select_tab ではなく active_tab の変化に追従する
    Effect::new(move |_| scroll_tab_into_view(tabs.active_tab.get()));

    view! {
        <h1 class="sr-only">"取引明細"</h1>
        <div class="mb-2 flex items-center justify-between gap-3">
            <nav class="no-print" aria-label="取引明細タブ">
                <div
                    class="receipts-tab-list"
                    role="tablist"
                >
                    {ReceiptsTab::ALL
                        .iter()
                        .copied()
                        .map(|tab| {
                            view! { <ReceiptsTabButton store=store tab=tab /> }
                        })
                        .collect_view()}
                </div>
            </nav>
            {move || {
                trigger_tab.get().map(|tab| view! {
                    <div
                        class="hidden shrink-0 lg:flex no-print"
                        data-testid="receipt-utility-toggle-bar"
                    >
                        <UtilityRailToggle store=store tab=tab />
                    </div>
                })
            }}
        </div>
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

#[component]
fn UtilityRailToggle(
    store: crate::features::receipts::ReceiptsStore,
    tab: ReceiptsTab,
) -> impl IntoView {
    let rail_open = store.utility_rail_open;
    let rows_store = store;
    let display = Memo::new(move |_| match rows_store.tab_state(tab) {
        TabState::Ready(data) => display_rows_for(rows_store, tab, data.rows),
        _ => Vec::new(),
    });
    let search = store.search;
    let filtered = Memo::new(move |_| {
        filter_receipts(
            tab,
            &display.get(),
            &search.with(|state| state.query.clone()),
        )
    });
    let total = Signal::derive(move || display.get().len());
    let badge_store = store;
    view! {
        <DisclosureToggle
            style=DisclosureStyle::Rail
            expanded=Signal::derive(move || rail_open.get())
            controls=utility_rail_id(tab)
            aria_label=Signal::derive(move || {
                if rail_open.get() {
                    "取り込み・検索パネルを閉じる".to_string()
                } else {
                    "取り込み・検索パネルを開く".to_string()
                }
            })
            testid="receipt-utility-toggle"
            hint=true
            on_toggle=move || store.toggle_utility_rail()
        >
            <span class="flex min-w-0 items-center gap-2">
                <span class="text-sm font-bold text-text">"取り込み・検索"</span>
                {move || {
                    badge_store.utility_filter_badge_visible().then(|| {
                        view! {
                            <Badge variant=BadgeVariant::Accent>
                                {format!(
                                    "絞り込み中 {} / {} 件",
                                    filtered.get().len(),
                                    total.get(),
                                )}
                            </Badge>
                        }
                    })
                }}
            </span>
        </DisclosureToggle>
    }
}
