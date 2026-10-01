use super::main_content::ReceiptsMainContent;
use super::search_card::ReceiptsSearchCard;
use super::TAB_IDS;
use crate::features::receipts::csv::CsvPreviewRow;
use crate::features::receipts::{
    truncated_list_warning, ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState,
};
use crate::support::csv_flow::CsvTabState;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::collapsible_search_card::is_narrow_viewport;
use crate::ui::csv_rail::CsvRailLayout;
use crate::ui::csv_section::{CsvSection, CsvSource};
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use crate::ui::elements::{
    Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant, LoadingStrip,
};
use crate::ui::mobile_toolbar::MobileToolbar;
use leptos::prelude::*;

pub(crate) fn empty_tab_data() -> ReceiptTabData {
    ReceiptTabData {
        rows: Vec::new(),
        summary: None,
        truncated: false,
    }
}

/// 開閉トグル(表の上)から aria-controls で参照する aside の id
pub(crate) fn utility_rail_id(tab: ReceiptsTab) -> String {
    format!("receipt-utility-rail-{}", TAB_IDS[tab as usize])
}

pub(crate) fn rail_shown(open: bool, state: &TabState) -> bool {
    open || matches!(state, TabState::Failed(_) | TabState::Loading)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WorkspaceTools {
    pub(crate) search: bool,
    pub(crate) summary: bool,
}

pub(crate) fn initial_search_expanded(narrow_viewport: bool) -> bool {
    !narrow_viewport
}

pub(crate) fn workspace_tools(
    state: &TabState,
    search_default: bool,
    has_preview: bool,
) -> WorkspaceTools {
    match state {
        TabState::Ready(data) => WorkspaceTools {
            search: !data.rows.is_empty() || !search_default || has_preview,
            summary: !data.rows.is_empty() || has_preview,
        },
        TabState::Failed(_) => WorkspaceTools {
            search: true,
            summary: has_preview,
        },
        _ => WorkspaceTools {
            search: false,
            summary: false,
        },
    }
}

#[component]
pub(crate) fn ReceiptWorkspace(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let csv_store = store;
    let loading_store = store;
    let main_store = store;
    let alert_store = store;
    // cache は全タブ共有の1 signal なので、他タブの取得進捗でも評価自体は走る。
    // memo で実際にこのタブの状態が変わった時だけビューを再生成させる
    let panel_state = Memo::new(move |_| store.tab_state(tab));
    let rail_store = store;
    Effect::new(move |_| {
        rail_store.init_utility_rail_from_state(tab, rail_store.tab_state(tab));
    });
    // 取得失敗・読み込み中のタブには開閉トグルを出していないので、畳んでいると
    // CSV 取り込み・検索に届かなくなる。ユーザーの開閉状態自体は変えず、表示時だけ開く
    let rail_visible = move || rail_shown(store.utility_rail_open.get(), &panel_state.get());
    let search_expanded = RwSignal::new(initial_search_expanded(is_narrow_viewport()));
    let csv_expanded = RwSignal::new(store.csv_state(tab).import_result.is_some());
    let tools = Memo::new(move |_| {
        workspace_tools(
            &panel_state.get(),
            store.search.with(|s| s.is_default()),
            store.has_csv_preview(tab),
        )
    });
    let has_search = Memo::new(move |_| tools.get().search);
    let has_summary = Memo::new(move |_| tools.get().summary);
    view! {
        // DOM 順は rail 先(キーボード・読み上げ順のため)、lg 以上は order で見た目を main 先に戻す
        <div
            class=move || {
                if rail_visible() {
                    "workspace-grid print:block"
                } else {
                    "workspace-grid rail-collapsed print:block"
                }
            }
            data-testid="receipt-workspace"
        >
            // 畳むのは PC(lg 以上)だけ。それより狭い帯ではレールは上段に積まれ、別の開閉が担う
            <aside
                id=utility_rail_id(tab)
                class=move || {
                    if rail_visible() {
                        "order-1 lg:order-2 print:hidden"
                    } else {
                        "order-1 lg:order-2 lg:hidden print:hidden"
                    }
                }
                data-testid="receipt-utility-rail"
            >
                // スマホでは帯と別カードの積み上げを維持するため枠は sm 以上だけにする
                // 年ピッカーのドロップダウンを切らないよう overflow は掛けない。
                // backdrop-blur が作る stack context に listbox が閉じ込められるため、1カラム幅でも表より前面に出す
                <div class="workspace-rail">
                    <MobileToolbar aria_label="取引明細の操作" testid="receipt-mobile-toolbar">
                        {move || has_search.get().then(|| view! {
                            <DisclosureToggle
                                style=DisclosureStyle::Toolbar
                                expanded=Signal::derive(move || search_expanded.get())
                                controls="search-options-body"
                                aria_label=Signal::derive(move || {
                                    if search_expanded.get() {
                                        "検索オプション 閉じる"
                                    } else if !store.search.with(|s| s.is_default()) {
                                        "検索オプション 開く（絞り込み適用中）"
                                    } else {
                                        "検索オプション 開く"
                                    }.to_string()
                                })
                                testid="receipt-search-toggle"
                                on_toggle=move || search_expanded.update(|open| *open = !*open)
                            >
                                "検索オプション"
                                {move || (!store.search.with(|s| s.is_default())).then(|| view! {
                                    <Badge variant=BadgeVariant::AccentFlat>"適用中"</Badge>
                                })}
                            </DisclosureToggle>
                        })}
                        {move || has_summary.get().then(|| view! {
                            <DisclosureToggle
                                style=DisclosureStyle::Toolbar
                                expanded=Signal::derive(move || store.mobile_summary_expanded.get())
                                controls="receipt-summary-mobile-body"
                                aria_label=Signal::derive(move || {
                                    if store.has_csv_preview(tab) { "集計情報(プレビュー)" } else { "集計情報" }.to_string()
                                })
                                testid="receipt-summary-compact-toggle"
                                on_toggle=move || store.mobile_summary_expanded.update(|open| *open = !*open)
                            >
                                "集計情報"
                            </DisclosureToggle>
                        })}
                        <DisclosureToggle
                            style=DisclosureStyle::ToolbarMenu
                            expanded=Signal::derive(move || csv_expanded.get())
                            controls=format!("{}-body", tab.csv_input_id())
                            aria_label="CSV取り込み・削除"
                            testid="receipt-csv-toggle"
                            on_toggle=move || csv_expanded.update(|open| *open = !*open)
                        >
                            "CSV取り込み・削除"
                        </DisclosureToggle>
                    </MobileToolbar>
                    <ReceiptsCsvSection store=csv_store tab=tab expanded=csv_expanded />
                    {move || {
                        let auth_loading = loading_store.auth_loading();
                        let fetching = loading_store.any_tab_fetching();
                        if !auth_loading && !fetching {
                            return ().into_any();
                        }
                        view! {
                            <LoadingStrip text=Signal::derive(move || {
                                if loading_store.auth_loading() {
                                    "認証状態を確認しています..."
                                } else {
                                    "データを読み込んでいます..."
                                }
                                .to_string()
                            }) />
                        }
                            .into_any()
                    }}
                    {move || match panel_state.get() {
                        TabState::Ready(data) => {
                            // 絞り込み対象がないのに検索オプションを出さない(適用中なら解除できるよう残す)。
                            // プレビュー中は取り込み行が絞り込み対象になるためカードを残す
                            if data.rows.is_empty()
                                && rail_store.search.with(|search| search.is_default())
                                && !rail_store.has_csv_preview(tab)
                            {
                                ().into_any()
                            } else {
                                view! { <ReceiptsSearchCard store=rail_store tab=tab data=data expanded=search_expanded /> }
                                    .into_any()
                            }
                        }
                        TabState::Failed(_) => {
                            view! {
                                <ReceiptsSearchCard
                                    store=rail_store
                                    tab=tab
                                    data=empty_tab_data()
                                    expanded=search_expanded
                                />
                            }
                                .into_any()
                        }
                        _ => ().into_any(),
                    }}
                </div>
            </aside>
            <div class="min-w-0 order-2 lg:order-1" data-testid="receipt-main-stage">
                // 裏再取得・CSV の失敗は一覧を消さずに知らせる。レールが畳まれても見えるよう表の上に出す
                {move || {
                    let Some(message) = alert_store
                        .csv_state(tab)
                        .error
                        .or_else(|| alert_store.refresh_error(tab))
                    else {
                        return ().into_any();
                    };
                    view! {
                        <div class="mb-4 no-print">
                            <Alert variant=AlertVariant::Danger>
                                <strong>"エラー:"</strong>
                                " "
                                {message}
                            </Alert>
                        </div>
                    }
                        .into_any()
                }}
                // レールを畳んでも見えるよう、件数上限の警告は表の上(レールの外)に出す
                {move || match panel_state.get() {
                    TabState::Ready(data) if data.truncated => {
                        view! {
                            <div class="mb-4">
                                <Alert variant=AlertVariant::Warning>
                                    {truncated_list_warning()}
                                </Alert>
                            </div>
                        }
                            .into_any()
                    }
                    _ => ().into_any(),
                }}
                {move || match panel_state.get() {
                    TabState::Loading => {
                        view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any()
                    }
                    TabState::Ready(data) => {
                        view! {
                            <ReceiptsMainContent
                                store=main_store
                                tab=tab
                                data=data
                            />
                        }
                        .into_any()
                    }
                    TabState::Failed(message) => {
                        let retry_store = main_store;
                        let preview = main_store.has_csv_preview(tab).then(|| {
                            view! {
                                <div class="mt-4">
                                    <ReceiptsMainContent
                                        store=main_store
                                        tab=tab
                                        data=empty_tab_data()
                                    />
                                </div>
                            }
                        });
                        view! {
                            <ListLoadError message=message on_retry=move || retry_store.reload(tab) />
                            {preview}
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

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
    view! { <CsvSection source=ReceiptCsvSource { store, tab } expanded=expanded layout=CsvRailLayout::Toolbar /> }
}
