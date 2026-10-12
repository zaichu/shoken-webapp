mod action_bar;
mod cards;
mod csv_page;
pub(crate) mod main_content;
#[allow(dead_code)]
mod pickers;
mod summary;
mod table;
mod tabs;

pub use action_bar::{ReceiptsActionBar, ReceiptsPcSearchBar};
pub use csv_page::{ReceiptsCsvDeletePage, ReceiptsCsvPage};
pub use main_content::ReceiptsMainContent;
pub use summary::SummaryStrip;
pub use table::ReceiptTable;
pub use tabs::{ReceiptsTabButton, TabPanel};

#[cfg(test)]
pub use main_content::utility_display_rows;

#[cfg(test)]
pub use pickers::next_year_option_index;

use crate::features::receipts::{ReceiptsTab, TabState, use_receipts_data};
use crate::session::use_session;
use crate::ui::page_frame::PageFrame;
use leptos::prelude::*;

pub(crate) const TAB_IDS: [&str; 3] = ["dividend", "domesticstock", "mutualfund"];

#[component]
pub fn ReceiptsPage() -> impl IntoView {
    let store = use_receipts_data(use_session(), ReceiptsTab::Dividend);
    store.revisit();

    let active_tab = store.active_tab;
    let tabs_loading = Memo::new(move |_| {
        store.auth_loading() || matches!(store.tab_state(active_tab.get()), TabState::Loading)
    });

    let csv_href =
        Signal::derive(move || format!("/receipts/csv?kind={}", active_tab.get().kind_param()));
    let csv_link: Children = Box::new(move || {
        view! {
            <a class="page-back-link" href=move || csv_href.get()>"CSV管理"</a>
        }
        .into_any()
    });

    // モバイル用: 下部操作帯(適用中の条件・検索欄・検索条件リンク)
    let action_bar = Box::new(move || {
        let store = store;
        let active_tab = active_tab;
        view! {
            <ReceiptsActionBar store=store active_tab=active_tab />
        }
        .into_any()
    });

    view! {
        <PageFrame title="取引明細" pc_layout=true links=csv_link action_bar=action_bar()>
            <div class="flex flex-1 flex-col min-h-0" data-testid="receipts-page">
                {/* モバイル用コンテンツ */}
                <div class="sm:block hidden">
                    <nav class="no-print mb-3" aria-label="種別">
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

                    <div class="flex-1 min-h-0 overflow-y-auto" data-testid="receipts-tabpanels">
                        {move || {
                            let loading = tabs_loading.get();
                            ReceiptsTab::ALL
                                .iter()
                                .copied()
                                .map(|tab| {
                                    view! { <TabPanel store=store tab=tab loading=loading /> }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>

                {/* PC 用コンテンツ */}
                <div class="hidden sm:block pc-receipts">
                    <nav class="no-print mb-4" aria-label="種別">
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

                    <ReceiptsPcSearchBar store=store active_tab=active_tab />

                    <div class="flex-1 min-h-0 overflow-y-auto" data-testid="receipts-tabpanels-pc">
                        {move || {
                            let loading = tabs_loading.get();
                            ReceiptsTab::ALL
                                .iter()
                                .copied()
                                .map(|tab| {
                                    view! { <TabPanel store=store tab=tab loading=loading /> }
                                })
                                .collect_view()
                        }}
                    </div>
                </div>
            </div>
        </PageFrame>
    }
}

#[cfg(test)]
mod tests;
