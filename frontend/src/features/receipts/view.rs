mod action_bar;
mod cards;
mod csv_page;
pub(crate) mod main_content;
mod summary;
mod table;
mod tabs;

pub use action_bar::{ReceiptsMobileBottomSection, ReceiptsSearchBar};
pub(crate) use csv_page::{ReceiptCsvSource, ReceiptsCsvPreview};
pub use main_content::ReceiptsMainContent;
pub use summary::SummaryStrip;
pub use table::ReceiptTable;
pub use tabs::{ReceiptsTabButton, TabPanel};

use crate::features::receipts::{ReceiptsTab, TabState, use_receipts_data};
use crate::session::use_session;
use crate::ui::labels;
use crate::ui::page_frame::PageFrame;
use crate::ui::page_skeleton::SkeletonTabs;
use leptos::prelude::*;

pub(crate) const TAB_IDS: [&str; 3] = ["dividend", "domesticstock", "mutualfund"];

#[component]
pub fn ReceiptsPage() -> impl IntoView {
    let store = use_receipts_data(
        use_session(),
        crate::settings::try_use_settings().receipts_initial_tab,
    );
    // 再訪ではストアのキャッシュがそのまま出るので、鮮度だけ裏で更新する
    store.revisit();

    let active_tab = store.active_tab;
    let tabs_loading = Memo::new(move |_| {
        store.auth_loading() || matches!(store.tab_state(active_tab.get()), TabState::Loading)
    });

    let csv_href = Signal::derive(move || format!("/csv?kind={}", active_tab.get().kind_param()));
    let csv_link: Children = Box::new(move || {
        view! {
            <a class="page-secondary-link-sm" href=move || csv_href.get()>"CSV管理"</a>
        }
        .into_any()
    });
    // 見出しの横の件数。現在のタブの全件数
    let title_count = Signal::derive(move || match store.tab_state(active_tab.get()) {
        TabState::Ready(data) => labels::count(data.total.unwrap_or(data.rows.len())),
        TabState::Loading | TabState::Failed(_) => String::new(),
    });
    // タブはページ名の下の行に置く(資産管理と同じ順序)
    let tabs = move || {
        view! {
            <nav aria-label="種別">
                <div class="page-tab-list" role="tablist">
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
    };

    view! {
        <PageFrame title="取引明細" title_count=title_count links=csv_link full_bleed=true mobile_inner_scroll=true>
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
                <div class="content-gutter no-print">
                    <SkeletonTabs>{tabs}</SkeletonTabs>
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

                <ReceiptsMobileBottomSection store=store />
            </div>
        </PageFrame>
    }
}

#[cfg(test)]
mod tests;
