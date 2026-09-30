use super::main_content::ReceiptsMainContent;
use super::search_card::ReceiptsSearchCard;
use crate::features::receipts::csv::CsvPreviewRow;
use crate::features::receipts::{
    truncated_list_warning, ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState,
};
use crate::support::csv_flow::CsvTabState;
use crate::ui::csv_section::{CsvSection, CsvSource};
use crate::ui::elements::{
    Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant, LoadingStrip,
};
use leptos::prelude::*;

pub(crate) fn empty_tab_data() -> ReceiptTabData {
    ReceiptTabData {
        rows: Vec::new(),
        summary: None,
        truncated: false,
    }
}

#[component]
pub(crate) fn ReceiptWorkspace(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let csv_store = store;
    let loading_store = store;
    let rail_store = store;
    let main_store = store;
    let alert_store = store;
    // cache は全タブ共有の1 signal なので、他タブの取得進捗でも評価自体は走る。
    // memo で実際にこのタブの状態が変わった時だけビューを再生成させる
    let panel_state = Memo::new(move |_| store.tab_state(tab));
    view! {
        // DOM 順は rail 先(キーボード・読み上げ順のため)、lg 以上は order で見た目を main 先に戻す
        <div
            class="workspace-grid print:block"
            data-testid="receipt-workspace"
        >
            <aside class="order-1 lg:order-2 print:hidden" data-testid="receipt-utility-rail">
                // スマホでは帯と別カードの積み上げを維持するため枠は sm 以上だけにする
                // 年ピッカーのドロップダウンを切らないよう overflow は掛けない。
                // backdrop-blur が作る stack context に listbox が閉じ込められるため、1カラム幅でも表より前面に出す
                <div class="workspace-rail">
                    <ReceiptsCsvSection store=csv_store tab=tab />
                    {move || {
                        // 裏再取得の失敗は一覧を消さず、エラーだけ知らせる
                        let Some(message) = alert_store
                            .csv_state(tab)
                            .error
                            .or_else(|| alert_store.refresh_error(tab))
                        else {
                            return ().into_any();
                        };
                        view! {
                            <section class="px-5 py-4">
                                <Alert variant=AlertVariant::Danger>
                                    <strong>"エラー:"</strong>
                                    " "
                                    {message}
                                </Alert>
                            </section>
                        }
                            .into_any()
                    }}
                    {move || {
                        match panel_state.get() {
                            TabState::Ready(data) if data.truncated => {
                                view! {
                                    <section class="px-5 py-4">
                                        <Alert variant=AlertVariant::Warning>
                                            {truncated_list_warning()}
                                        </Alert>
                                    </section>
                                }
                                    .into_any()
                            }
                            _ => ().into_any(),
                        }
                    }}
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
                                view! { <ReceiptsSearchCard store=rail_store tab=tab data=data /> }
                                    .into_any()
                            }
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
                {move || match panel_state.get() {
                    TabState::Loading => {
                        view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any()
                    }
                    TabState::Ready(data) => {
                        view! { <ReceiptsMainContent store=main_store tab=tab data=data /> }
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
fn ReceiptsCsvSection(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    view! { <CsvSection source=ReceiptCsvSource { store, tab } /> }
}
