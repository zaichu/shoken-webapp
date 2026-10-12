use crate::features::receipts::view::search_card::ReceiptsSearchCard;
use crate::features::receipts::{ReceiptsStore, TabState};
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::collapsible_search_card::is_narrow_viewport;
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use leptos::prelude::*;

#[component]
pub(crate) fn ReceiptsPanel(store: ReceiptsStore) -> impl IntoView {
    let rail_store = store;
    // 狭い帯ではドロワー内の高さを抑えるため畳んで始める(640px未満)
    let search_expanded = RwSignal::new(!is_narrow_viewport());
    let active_tab = Memo::new(move |_| store.active_tab.get());
    let panel_state = Memo::new(move |_| store.tab_state(active_tab.get()));
    // 0件かつ条件未適用のタブでは検索対象がないためカードごと出さない
    let show_search = Memo::new(move |_| match panel_state.get() {
        TabState::Ready(data) => {
            !data.rows.is_empty() || !store.search.with(|search| search.is_default())
        }
        TabState::Failed(_) => true,
        _ => false,
    });
    let show_summary = Memo::new(move |_| match panel_state.get() {
        TabState::Ready(data) => !data.rows.is_empty(),
        _ => false,
    });

    view! {
        <div>
            // スマホでは検索・集計を1行のツールバーから開閉する
            <div
                class="mobile-toolbar"
                role="group"
                aria-label="取引明細の操作"
                data-testid="receipt-mobile-toolbar"
            >
                {move || {
                    show_search.get().then(|| {
                        view! {
                            <DisclosureToggle
                                style=DisclosureStyle::Rail
                                expanded=Signal::derive(move || {
                                    search_expanded.get()
                                })
                                controls="search-options-body".to_string()
                                aria_label=Signal::derive(move || {
                                    if search_expanded.get() {
                                        "検索オプション 閉じる".to_string()
                                    } else if !store.search.with(|s| s.is_default()) {
                                        "検索オプション 開く（絞り込み適用中）".to_string()
                                    } else {
                                        "検索オプション 開く".to_string()
                                    }
                                })
                                testid="receipt-search-toggle"
                                on_toggle=move || {
                                    search_expanded.update(|open| *open = !*open)
                                }
                            >
                                <span class="font-bold">"検索"</span>
                                {move || {
                                    (!store.search.with(|s| s.is_default())).then(|| {
                                        view! {
                                            <Badge variant=BadgeVariant::AccentFlat>
                                                "適用中"
                                            </Badge>
                                        }
                                    })
                                }}
                            </DisclosureToggle>
                        }
                    })
                }}
                {move || {
                    show_summary.get().then(|| {
                        view! {
                            <DisclosureToggle
                                style=DisclosureStyle::Rail
                                expanded=Signal::derive(move || {
                                    store.mobile_summary_expanded.get()
                                })
                                controls="receipt-summary-mobile-body"
                                    .to_string()
                                aria_label="集計情報".to_string()
                                testid="receipt-summary-compact-toggle"
                                on_toggle=move || {
                                    store
                                        .mobile_summary_expanded
                                        .update(|open| *open = !*open)
                                }
                            >
                                <span class="font-bold">"集計"</span>
                            </DisclosureToggle>
                        }
                    })
                }}
            </div>
            {move || {
                let tab = active_tab.get();
                let data = match panel_state.get() {
                    TabState::Ready(data) => data,
                    TabState::Failed(_) => {
                        crate::features::receipts::view::workspace::empty_tab_data()
                    }
                    _ => return ().into_any(),
                };
                show_search
                    .get()
                    .then(|| {
                        view! {
                            <ReceiptsSearchCard
                                store=rail_store
                                tab=tab
                                data=data
                                expanded=search_expanded
                            />
                        }
                    })
                    .into_any()
            }}
        </div>
    }
}
