use super::TAB_IDS;
use crate::features::receipts::view::ReceiptsMainContent;
use crate::features::receipts::{ReceiptsStore, ReceiptsTab};
use crate::ui::button::TabButton;
use crate::ui::labels;
use crate::ui::state::{ListSkeleton, ListSkeletonVariant};
use crate::ui::underline_tabs;
use crate::ui::underline_tabs::SwipeMotion;
use leptos::prelude::*;

pub fn next_tab_index(current: usize, key: &str) -> Option<usize> {
    underline_tabs::next_tab_index(TAB_IDS.len(), current, key)
}

fn focus_tab(index: usize, suffix: &str) {
    underline_tabs::focus_tab_by_id_after_paint(format!("tab-{}{}", TAB_IDS[index], suffix));
}

#[component]
pub fn ReceiptsTabButton(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    motion: SwipeMotion,
    #[prop(optional)] suffix: Option<&'static str>,
) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let suffix = suffix.unwrap_or("");
    let id = format!("tab-{slug}{suffix}");
    let label = tab.label();
    let selected = store;
    let keyed = store;
    let clicked = store;
    let guard = motion;
    view! {
        <TabButton
            id=id
            controls=format!("tabpanel-{slug}")
            selected=move || selected.active_tab.get() == tab
            on_click=move |event| {
                if event.detail() != 0
                    && underline_tabs::swipe_should_ignore_input(
                        guard.settling.get_untracked(),
                        underline_tabs::prefers_reduced_motion(),
                    )
                {
                    return;
                }
                clicked.select_tab(tab);
            }
            on_keydown=move |event| {
                let current = ReceiptsTab::ALL
                    .iter()
                    .position(|item| *item == keyed.active_tab.get_untracked())
                    .unwrap_or(0);
                if let Some(next) = next_tab_index(current, &event.key()) {
                    event.prevent_default();
                    keyed.select_tab(ReceiptsTab::ALL[next]);
                    focus_tab(next, suffix);
                }
            }
        >
            {label}
        </TabButton>
    }
}

#[component]
pub fn ReceiptsTabNav(
    store: ReceiptsStore,
    motion: SwipeMotion,
    #[prop(optional)] suffix: Option<&'static str>,
) -> impl IntoView {
    let active_tab = store.active_tab;
    let list = NodeRef::<leptos::html::Div>::new();
    {
        let position = active_tab;
        let follow = motion;
        Effect::new(move || {
            let selected = ReceiptsTab::ALL
                .iter()
                .position(|tab| *tab == position.get())
                .unwrap_or(0);
            let progress = follow.progress.get();
            let _dragging = follow.dragging.get();
            if let Some(list) = list.get() {
                underline_tabs::place_ink(&list, selected, progress);
            }
        });
    }
    view! {
        <nav aria-label=labels::KIND>
            <div
                class="page-tab-list relative"
                role="tablist"
                data-swiping=move || motion.dragging.get().to_string()
                node_ref=list
            >
                {ReceiptsTab::ALL
                    .iter()
                    .copied()
            .map(|tab| {
                        view! {
                            <ReceiptsTabButton
                                store=store
                                tab=tab
                                motion=motion
                                suffix=suffix.unwrap_or("")
                            />
                        }
                    })
                    .collect_view()}
                <span
                    class="tab-swipe-ink"
                    data-on=move || motion.dragging.get().to_string()
                ></span>
            </div>
        </nav>
    }
}

#[component]
pub fn TabPanel(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    #[prop(optional)] loading: bool,
    motion: SwipeMotion,
) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let index = ReceiptsTab::ALL
        .iter()
        .position(|item| *item == tab)
        .unwrap_or(0);
    let shown = Memo::new(move |_| {
        let current = store.active_tab.get();
        let position = ReceiptsTab::ALL
            .iter()
            .position(|item| *item == current)
            .unwrap_or(0);
        current == tab || (motion.dragging.get() && position.abs_diff(index) == 1)
    });
    view! {
        <div
            id={format!("tabpanel-{slug}")}
            role="tabpanel"
            aria-labelledby={format!("tab-{slug}")}
            class="tab-swipe-slide"
            data-index=index.to_string()
            data-active=move || (store.active_tab.get() == tab).to_string()
            hidden=move || !shown.get()
        >
            {move || {
                if !shown.get() {
                    return ().into_any();
                }
                let store = store;
                if loading {
                    return view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any();
                }
                if !store.is_authenticated() {
                    return view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any();
                }
                view! { <ReceiptsMainContent store=store tab=tab /> }.into_any()
            }}
        </div>
    }
}
