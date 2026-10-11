mod cards;
mod csv_page;
pub(crate) mod main_content;
mod panel;
mod pickers;
mod search_card;
mod summary;
mod table;
mod tabs;
mod workspace;

pub use csv_page::{ReceiptsCsvDeletePage, ReceiptsCsvPage};

use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::{ReceiptRow, ReceiptsTab, TabState, use_receipts_data};
use crate::session::use_session;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::page_frame::PageFrame;
use crate::ui::workspace_shell::WorkspaceShell;
use leptos::prelude::*;
use panel::ReceiptsPanel;
use tabs::{ReceiptsTabButton, TabPanel, scroll_tab_into_view};
use workspace::utility_rail_id;

pub(crate) const TAB_IDS: [&str; 3] = ["dividend", "domesticstock", "mutualfund"];

#[component]
pub fn ReceiptsPage() -> impl IntoView {
    let store = use_receipts_data(use_session(), ReceiptsTab::Dividend);
    // 再訪ではストアのキャッシュがそのまま出るので、鮮度だけ裏で更新する
    store.revisit();
    let busy = store;
    let panels_store = store;
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

    let csv_href = Signal::derive(move || {
        format!("/receipts/csv?kind={}", store.active_tab.get().kind_param())
    });
    let csv_link: Children = Box::new(move || {
        view! {
            <a class="page-back-link" href=move || csv_href.get()>"CSV管理"</a>
        }
        .into_any()
    });
    view! {
        <PageFrame title="取引明細" full_bleed=true links=csv_link>
        <div
            class="mt-0 flex flex-1 flex-col"
            aria-busy=move || {
                if busy.auth_loading() || busy.any_tab_fetching() {
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
                    panel_label="取り込み・検索"
                    collapsible=true
                    panel_open=Signal::derive(move || panel_store.utility_rail_open.get())
                    on_toggle=Callback::new(move |_| panel_store.toggle_utility_rail())
                    on_close=Callback::new(move |_| {
                        if panel_store.utility_rail_open.get_untracked() {
                            panel_store.toggle_utility_rail();
                        }
                    })
                    panel=view! { <ReceiptsPanel store=store /> }.into_any()
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
        </div>
        </PageFrame>
    }
}

#[cfg(test)]
mod tests;

pub(crate) fn utility_display_rows(
    store: crate::features::receipts::ReceiptsStore,
    tab: ReceiptsTab,
) -> Vec<ReceiptRow> {
    match store.tab_state(tab) {
        TabState::Ready(data) => data.rows,
        _ => Vec::new(),
    }
}
