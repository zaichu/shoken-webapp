mod action_bar;
mod cards;
mod conditions;
mod csv_page;
pub(crate) mod main_content;
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
    let csv_link: Children = Box::new(move || {
        view! {
            <a class="page-secondary-link-sm" href=move || csv_href.get()>"CSV管理"</a>
        }
        .into_any()
    });
    // スマホはヘッダーの下、PC は見出しの横に置く
    let tabs: Children = Box::new(move || {
        view! {
            <nav class="no-print" aria-label="種別">
                <div class="receipts-tab-list" role="tablist">
                    {ReceiptsTab::ALL
                        .iter()
                        .copied()
                        .map(|tab| {
                            view! { <ReceiptsTabButton store=store tab=tab /> }
                        })
                        .collect_view()}
                </div>
            </nav>
        }
        .into_any()
    });

    view! {
        <PageFrame title="取引明細" links=csv_link title_row=tabs full_bleed=true>
            <div
                class="flex flex-1 flex-col min-h-0 lg:flex-none"
                aria-busy=move || {
                    if store.auth_loading() || store.any_tab_fetching() {
                        "true"
                    } else {
                        "false"
                    }
                }
            >
                <div class="content-gutter no-print hidden shrink-0 py-4 lg:block">
                    <ReceiptsPcSearchBar store=store active_tab=active_tab />
                </div>

                <div
                    class="content-gutter flex-1 min-h-0 overflow-y-auto lg:flex-none lg:overflow-visible"
                    data-testid="receipts-workspace"
                >
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
        </PageFrame>
    }
}

#[cfg(test)]
mod tests;
