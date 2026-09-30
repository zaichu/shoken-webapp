use super::main_content::ReceiptsMainContent;
use super::search_card::ReceiptsSearchCard;
use super::TAB_IDS;
use crate::features::receipts::csv::CsvPreviewRow;
use crate::features::receipts::{
    truncated_list_warning, ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState,
};
use crate::support::csv_flow::{row_error_text, CsvTabState};
use crate::ui::csv_section::{CsvSection, CsvSource};
use crate::ui::elements::{ListLoadError, ListSkeleton, Spinner, SpinnerSize};
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

#[component]
pub(crate) fn ReceiptWorkspace(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let csv_store = store;
    let preview_store = store;
    let loading_store = store;
    let main_store = store;
    let alert_store = store;
    // cache は全タブ共有の1 signal なので、他タブの取得進捗でも評価自体は走る。
    // memo で実際にこのタブの状態が変わった時だけビューを再生成させる
    let panel_state = Memo::new(move |_| store.tab_state(tab));
    // refetch などで workspace が作り直されても、初期開閉は store 側の記憶が担う
    let rail_store = store;
    Effect::new(move |_| {
        if let TabState::Ready(data) = rail_store.tab_state(tab) {
            rail_store.init_utility_rail(tab, !data.rows.is_empty());
        }
    });
    // 取得失敗・読み込み中のタブには開閉トグルを出していないので、畳んでいると
    // CSV 取り込み・検索に届かなくなる。ユーザーの開閉状態自体は変えず、表示時だけ開く
    let rail_visible = move || {
        store.utility_rail_open.get()
            || matches!(panel_state.get(), TabState::Failed(_) | TabState::Loading)
    };
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
                    <ReceiptsCsvSection store=csv_store tab=tab />
                    {move || {
                        let Some(message) = alert_store.csv_state(tab).error else {
                            return ().into_any();
                        };
                        view! {
                            <section class="px-5 py-4">
                                <div
                                    class="rounded-lg border border-negative-border bg-negative-soft px-4 py-3 text-sm font-medium text-negative-strong"
                                    role="alert"
                                    aria-live="assertive"
                                >
                                    <strong>"エラー:"</strong>
                                    " "
                                    {message}
                                </div>
                            </section>
                        }
                            .into_any()
                    }}
                    {move || {
                        let state = preview_store.csv_state(tab);
                        let authenticated = preview_store.is_authenticated();
                        let has_file = state.file_name.is_some();
                        let previewing = state.previewing;
                        let Some(preview) = state
                            .preview
                            .filter(|_| authenticated && has_file && !previewing)
                        else {
                            return ().into_any();
                        };
                        let has_errors = !preview.errors.is_empty();
                        let alert_class = if has_errors {
                            "border-accent-border bg-accent-soft text-accent-text"
                        } else {
                            "border-info-border bg-info-soft text-info-deep"
                        };
                        view! {
                            <section class="px-5 py-4" role="status" aria-live="polite">
                                <div class=format!(
                                    "rounded-lg border px-4 py-3 text-sm font-medium shadow-sm {alert_class}"
                                )>
                                    <p>
                                        <strong>{format!("{}件 追加で保存されます", preview.valid_rows)}</strong>
                                        {has_errors.then(|| format!(" / {}件エラー", preview.errors.len()))}
                                        <span class="ml-2 text-xs">"（保存モード: 追加）"</span>
                                    </p>
                                    {has_errors.then(|| {
                                        view! {
                                            <ul class="mt-2 list-disc list-inside text-sm space-y-1">
                                                {preview
                                                    .errors
                                                    .iter()
                                                    .map(|error| view! { <li>{row_error_text(error)}</li> })
                                                    .collect_view()}
                                            </ul>
                                        }
                                    })}
                                </div>
                            </section>
                        }
                            .into_any()
                    }}
                    {move || {
                        let auth_loading = loading_store.auth_loading();
                        let fetching = loading_store.any_tab_fetching();
                        if !auth_loading && !fetching {
                            return ().into_any();
                        }
                        view! {
                            <div aria-live="polite" aria-atomic="true">
                                <section class="px-5 py-4" role="status">
                                    <div class="flex items-center gap-2 text-text-muted">
                                        <Spinner size=SpinnerSize::Sm class="" />
                                        <p class="text-sm">
                                            {auth_loading.then_some("認証状態を確認しています...")}
                                            {fetching.then_some("データを読み込んでいます...")}
                                        </p>
                                    </div>
                                </section>
                            </div>
                        }
                            .into_any()
                    }}
                    {move || match panel_state.get() {
                        TabState::Ready(data) => {
                            view! { <ReceiptsSearchCard store=rail_store tab=tab data=data /> }
                                .into_any()
                        }
                        TabState::Failed(_) => {
                            view! {
                                <ReceiptsSearchCard
                                    store=rail_store
                                    tab=tab
                                    data=empty_tab_data()
                                />
                            }
                                .into_any()
                        }
                        _ => ().into_any(),
                    }}
                </div>
            </aside>
            <div class="min-w-0 order-2 lg:order-1" data-testid="receipt-main-stage">
                // レールを畳んでも見えるよう、件数上限の警告は表の上(レールの外)に出す
                {move || match panel_state.get() {
                    TabState::Ready(data) if data.truncated => {
                        view! {
                            <div class="mb-4" role="status" aria-live="polite">
                                <div class="rounded-lg border border-accent-border bg-accent-soft px-4 py-3 text-sm font-medium text-accent-text">
                                    {truncated_list_warning()}
                                </div>
                            </div>
                        }
                            .into_any()
                    }
                    _ => ().into_any(),
                }}
                {move || match panel_state.get() {
                    TabState::Loading => view! { <ListSkeleton /> }.into_any(),
                    TabState::Ready(data) => {
                        view! {
                            <ReceiptsMainContent
                                store=main_store
                                tab=tab
                                data=data
                                rail_toggle=true
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
                                        rail_toggle=false
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
        self.store.auth_loading() || self.store.csv_busy(self.tab) || self.store.any_tab_fetching()
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
fn ReceiptsCsvSection(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    view! { <CsvSection source=ReceiptCsvSource { store, tab } /> }
}
