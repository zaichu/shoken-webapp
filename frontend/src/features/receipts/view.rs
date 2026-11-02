mod action_bar;
mod cards;
pub(crate) mod main_content;
mod summary;
mod table;
mod tabs;

pub use action_bar::{ReceiptsMobileSearchEntry, ReceiptsSearchBar};
pub use main_content::ReceiptsMainContent;
pub use summary::SummaryStrip;
pub use table::ReceiptTable;
pub use tabs::{ReceiptsTabButton, TabPanel};

use crate::features::receipts::{ReceiptsTab, TabState, use_receipts_data};
use crate::session::use_session;
use crate::ui::labels::{CSV_MANAGE_TITLE, KIND, RECEIPTS_TITLE};
use crate::ui::page_frame::PageFrame;
use crate::ui::page_skeleton::SkeletonTabs;
use crate::ui::underline_tabs::{SwipeArea, SwipeMotion, tab_ink_offset};
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
    let motion = SwipeMotion::new();
    let current_tab = Signal::derive(move || {
        ReceiptsTab::ALL
            .iter()
            .position(|tab| *tab == active_tab.get())
            .unwrap_or(0)
    });
    let tabs_loading = Memo::new(move |_| {
        store.auth_loading() || matches!(store.tab_state(active_tab.get()), TabState::Loading)
    });

    let csv_link: Children = Box::new(move || {
        view! {
            <a class="page-secondary-link-sm" href="/csv">{CSV_MANAGE_TITLE}</a>
        }
        .into_any()
    });
    // タブはページ名の下の行に置く(資産管理と同じ順序)
    let tabs = move || {
        view! {
            <nav aria-label=KIND>
                <div
                    class="page-tab-list relative"
                    role="tablist"
                    data-swiping=move || motion.dragging.get().to_string()
                >
                    {ReceiptsTab::ALL
                        .iter()
                        .copied()
                        .map(|tab| {
                            view! { <ReceiptsTabButton store=store tab=tab motion=motion /> }
                        })
                        .collect_view()}
                    <span
                        class="tab-swipe-ink"
                        data-on=move || motion.dragging.get().to_string()
                        style:width=format!("calc(100% / {})", ReceiptsTab::ALL.len())
                        style:transform=move || {
                            let position = ReceiptsTab::ALL
                                .iter()
                                .position(|tab| *tab == active_tab.get())
                                .unwrap_or(0);
                            format!(
                                "translateX({}%)",
                                tab_ink_offset(position, motion.progress.get()) * 100.0
                            )
                        }
                    ></span>
                </div>
            </nav>
        }
        .into_any()
    };

    view! {
        <PageFrame title=RECEIPTS_TITLE links=csv_link full_bleed=true>
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
                <SwipeArea
                    testid="receipts-workspace"
                    count=Signal::derive(move || ReceiptsTab::ALL.len())
                    current=current_tab
                    on_select=Callback::new(move |next: usize| {
                        store.select_tab(ReceiptsTab::ALL[next])
                    })
                    motion=motion
                >
                    {move || {
                        let loading = tabs_loading.get();
                        ReceiptsTab::ALL
                            .iter()
                            .copied()
                            .map(|tab| {
                                view! { <TabPanel store=store tab=tab loading=loading motion=motion /> }
                            })
                            .collect_view()
                    }}
                </SwipeArea>
            </div>
        </PageFrame>
    }
}

#[cfg(test)]
mod tests;
