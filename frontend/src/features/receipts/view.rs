mod action_bar;
mod cards;
mod conditions;
mod csv_page;
pub(crate) mod main_content;
#[allow(dead_code)]
mod pickers;
mod summary;
mod table;
mod tabs;

pub use action_bar::{ReceiptsMobileBottomSection, ReceiptsPcSearchBar};
pub use conditions::ReceiptsConditionsPage;
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
    // 再訪ではストアのキャッシュがそのまま出るので、鮮度だけ裏で更新する
    store.revisit();

    let active_tab = store.active_tab;
    let tabs_loading = Memo::new(move |_| {
        store.auth_loading() || matches!(store.tab_state(active_tab.get()), TabState::Loading)
    });

    let csv_href =
        Signal::derive(move || format!("/receipts/csv?kind={}", active_tab.get().kind_param()));
    let mobile_csv_link: Children = Box::new(move || {
        view! {
            <a class="page-secondary-link-sm" href=move || csv_href.get()>"CSV管理"</a>
        }
        .into_any()
    });

    view! {
        <PageFrame
            title="取引明細"
            mobile_links=mobile_csv_link
            full_bleed=true
        >
            <div class="flex flex-1 flex-col min-h-0" data-testid="receipts-page">
                <div class="flex flex-1 flex-col min-h-0 lg:hidden">
                    <nav class="no-print mx-4 mb-2" aria-label="種別">
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

                    <div class="flex-1 min-h-0 overflow-y-auto pb-28" data-testid="receipts-tabpanels">
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

                    <ReceiptsMobileBottomSection store=store active_tab=active_tab />
                </div>

                <div class="pc-receipts page-gutter hidden lg:flex lg:flex-col">
                    <div class="flex items-center gap-4">
                        <h1 class="text-xl font-bold text-ink">"取引明細"</h1>
                        <nav class="no-print" aria-label="種別">
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
                        <a class="page-secondary-link ml-auto" href=move || csv_href.get()>"CSV管理"</a>
                    </div>

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
