use crate::features::receipts::view::search_card::ReceiptsSearchCard;
use crate::features::receipts::{csv::CsvPreviewRow, ReceiptsStore, ReceiptsTab, TabState};
use crate::support::csv_flow::CsvTabState;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::collapsible_search_card::is_narrow_viewport;
use crate::ui::csv_section::{CsvSection, CsvSource};
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct ReceiptCsvSource {
    store: ReceiptsStore,
    tab: ReceiptsTab,
}

impl CsvSource for ReceiptCsvSource {
    type Row = CsvPreviewRow;

    fn input_id(&self) -> &'static str {
        self.tab.csv_input_id()
    }

    fn save_action(&self) -> &'static str {
        "追加で保存"
    }

    fn mode_label(&self) -> &'static str {
        "追加保存"
    }

    fn toggle_testid(&self) -> &'static str {
        "receipt-csv-toggle"
    }

    fn section_class(&self) -> &'static str {
        "sm:rounded-t-xl"
    }

    fn csv_state(&self) -> CsvTabState<CsvPreviewRow> {
        self.store.csv_state(self.tab)
    }

    fn is_authenticated(&self) -> bool {
        self.store.is_authenticated()
    }

    fn input_disabled(&self) -> bool {
        self.store.csv_input_disabled(self.tab)
    }

    fn db_count(&self) -> usize {
        self.store.count(self.tab)
    }

    fn delete_disabled(&self, state: &CsvTabState<CsvPreviewRow>) -> bool {
        state.saving || state.deleting || self.store.any_tab_fetching()
    }

    fn select_file(&self, file: web_sys::File) {
        self.store.select_file(self.tab, file)
    }

    fn save_csv(&self) {
        self.store.save_csv(self.tab)
    }

    fn open_delete_confirm(&self) {
        self.store.open_delete_confirm(self.tab)
    }
}

#[component]
fn ReceiptsCsvSection(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    expanded: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <CsvSection
            source=ReceiptCsvSource { store, tab }
            expanded=expanded
            external_toggle=true
        />
    }
}

#[component]
pub(crate) fn ReceiptPanelContent(store: ReceiptsStore) -> impl IntoView {
    let csv_store = store;
    let rail_store = store;
    Effect::new(move |_| {
        let tab = rail_store.active_tab.get();
        rail_store.init_utility_rail_from_state(tab, rail_store.tab_state(tab));
    });
    // 狭い帯ではドロワー内の高さを抑えるため畳んで始める(640px未満)
    let search_expanded = RwSignal::new(!is_narrow_viewport());
    let active_tab = Memo::new(move |_| store.active_tab.get());
    let panel_state = Memo::new(move |_| store.tab_state(active_tab.get()));
    let has_preview = Memo::new(move |_| store.has_csv_preview(active_tab.get()));
    // 0件かつ条件未適用のタブでは検索対象がないためカードごと出さない
    let show_search = Memo::new(move |_| match panel_state.get() {
        TabState::Ready(data) => {
            !data.rows.is_empty()
                || !store.search.with(|search| search.is_default())
                || has_preview.get()
        }
        TabState::Failed(_) => true,
        _ => false,
    });
    let show_summary = Memo::new(move |_| match panel_state.get() {
        TabState::Ready(data) => !data.rows.is_empty() || has_preview.get(),
        TabState::Failed(_) => has_preview.get(),
        _ => false,
    });

    view! {
        <div>
            {move || {
                let tab = active_tab.get();
                // タブごとのCSV状態に合わせ、保存結果があれば開いて始める
                let csv_expanded =
                    RwSignal::new(csv_store.csv_state(tab).import_result.is_some());
                let csv_body_id = format!("{}-body", tab.csv_input_id());
                view! {
                    // スマホでは検索・集計・CSVを1行のツールバーから開閉する
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
                                        aria_label=Signal::derive(move || {
                                            if has_preview.get() {
                                                "集計情報(プレビュー)".to_string()
                                            } else {
                                                "集計情報".to_string()
                                            }
                                        })
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
                        <DisclosureToggle
                            style=DisclosureStyle::Rail
                            expanded=Signal::derive(move || csv_expanded.get())
                            controls=csv_body_id
                            aria_label="CSV取り込み・削除".to_string()
                            testid="receipt-csv-toggle"
                            on_toggle=move || csv_expanded.update(|open| *open = !*open)
                        >
                            <span class="font-bold">"CSV"</span>
                        </DisclosureToggle>
                    </div>
                    <ReceiptsCsvSection
                        store=csv_store
                        tab=tab
                        expanded=csv_expanded
                    />
                }
            }}
            {move || {
                let tab = active_tab.get();
                match panel_state.get() {
                    TabState::Ready(data) => {
                        if data.rows.is_empty()
                            && rail_store.search.with(|search| search.is_default())
                            && !has_preview.get()
                        {
                            ().into_any()
                        } else {
                            view! {
                                <ReceiptsSearchCard
                                    store=rail_store
                                    tab=tab
                                    data=data
                                    expanded=search_expanded
                                />
                            }
                                .into_any()
                        }
                    }
                    TabState::Failed(_) => {
                        view! {
                            <ReceiptsSearchCard
                                store=rail_store
                                tab=tab
                                data=crate::features::receipts::view::workspace::empty_tab_data()
                                expanded=search_expanded
                            />
                        }
                            .into_any()
                    }
                    _ => ().into_any(),
                }
            }}
        </div>
    }
}
