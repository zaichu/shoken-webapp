use super::main_content::ReceiptsMainContent;
use super::search_card::ReceiptsSearchCard;
use crate::components::csv_rail::CsvActionRail;
use crate::components::ui::{Loading, Spinner};
use crate::csv_flow::row_error_text;
use crate::receipts::{
    truncated_list_warning, ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState,
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
    let csv_store = store.clone();
    let preview_store = store.clone();
    let loading_store = store.clone();
    let rail_store = store.clone();
    let main_store = store.clone();
    let alert_store = store.clone();
    // cache は全タブ共有の1 signal なので、他タブの取得進捗でも評価自体は走る。
    // memo で実際にこのタブの状態が変わった時だけビューを再生成させる
    let panel_state = Memo::new(move |_| store.tab_state(tab));
    view! {
        // DOM 順は rail 先(キーボード・読み上げ順のため)、lg 以上は order で見た目を main 先に戻す
        <div
            class="grid gap-3 sm:gap-4 lg:grid-cols-[minmax(0,1fr)_19rem] lg:items-start xl:gap-5 xl:grid-cols-[minmax(0,1fr)_20rem]"
            data-testid="receipt-workspace"
        >
            <aside class="order-1 lg:order-2" data-testid="receipt-utility-rail">
                // スマホでは帯と別カードの積み上げを維持するため枠は sm 以上だけにする
                // 年ピッカーのドロップダウンを切らないよう overflow は掛けない。
                // backdrop-blur が作る stack context に listbox が閉じ込められるため、1カラム幅でも表より前面に出す
                <div class="workspace-rail">
                    <ReceiptsCsvSection store=csv_store.clone() tab=tab />
                    {move || {
                        let Some(message) = alert_store.rail_error(tab)
                        else {
                            return ().into_any();
                        };
                        view! {
                            <section class="px-5 py-4">
                                <div
                                    class="rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm font-medium text-red-800"
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
                        match panel_state.get() {
                            TabState::Ready(data) if data.truncated => {
                                view! {
                                    <section class="px-5 py-4" role="status" aria-live="polite">
                                        <div class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm font-medium text-amber-900">
                                            {truncated_list_warning()}
                                        </div>
                                    </section>
                                }
                                    .into_any()
                            }
                            _ => ().into_any(),
                        }
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
                            "border-amber-200 bg-amber-50 text-amber-900"
                        } else {
                            "border-blue-200 bg-blue-50 text-blue-800"
                        };
                        view! {
                            <section class="px-5 py-4" role="status" aria-live="polite">
                                <div class=format!(
                                    "rounded-lg border px-4 py-3 text-sm font-medium shadow-sm {alert_class}"
                                )>
                                    <p>
                                        <strong>{format!("{}件 追加で保存されます", preview.valid_rows)}</strong>
                                        {has_errors.then(|| format!(" / {}件エラー", preview.errors.len()))}
                                        <span class="ml-2 text-xs text-secondary">"（保存モード: 追加）"</span>
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
                                    <div class="flex items-center gap-2 text-slate-600">
                                        <Spinner size="sm" class="" />
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
                            view! { <ReceiptsSearchCard store=rail_store.clone() tab=tab data=data /> }
                                .into_any()
                        }
                        TabState::Failed(_) => {
                            view! {
                                <ReceiptsSearchCard
                                    store=rail_store.clone()
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
                    TabState::Loading => view! { <Loading /> }.into_any(),
                    TabState::Ready(data) => {
                        view! { <ReceiptsMainContent store=main_store.clone() tab=tab data=data /> }
                            .into_any()
                    }
                    TabState::Failed(_) => {
                        view! {
                            <ReceiptsMainContent
                                store=main_store.clone()
                                tab=tab
                                data=empty_tab_data()
                            />
                        }
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn ReceiptsCsvSection(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let input_id = match tab {
        ReceiptsTab::Dividend => "csv-file-input-dividend",
        ReceiptsTab::DomesticStock => "csv-file-input-domesticstock",
        ReceiptsTab::MutualFund => "csv-file-input-mutualfund",
    };
    let selected = store.clone();
    let selected_file_name =
        Memo::new(move |_| selected.csv_state(tab).file_name.unwrap_or_default());
    let disabled_store = store.clone();
    let file_input_disabled = Memo::new(move |_| {
        disabled_store.auth_loading()
            || !disabled_store.is_authenticated()
            || disabled_store.csv_busy(tab)
            || disabled_store.any_tab_fetching()
    });
    let has_file = store.clone();
    let has_csv_file = Memo::new(move |_| {
        has_file.is_authenticated() && has_file.csv_state(tab).file_name.is_some()
    });
    let label_store = store.clone();
    let save_label = Memo::new(move |_| label_store.csv_state(tab).save_label("追加で保存"));
    let save_dis = store.clone();
    let save_disabled = Memo::new(move |_| save_dis.csv_busy(tab));
    let has_db = store.clone();
    let has_db_data = Memo::new(move |_| has_db.is_authenticated() && has_db.count(tab) > 0);
    let del_label = store.clone();
    let delete_label =
        Memo::new(move |_| del_label.csv_state(tab).delete_label(del_label.count(tab)));
    let del_dis = store.clone();
    let delete_disabled = Memo::new(move |_| {
        let state = del_dis.csv_state(tab);
        state.saving || state.deleting || del_dis.any_tab_fetching()
    });
    let result_store = store.clone();
    let save_result = Memo::new(move |_| result_store.csv_state(tab).import_result);
    let file_select = store.clone();
    let save = store.clone();
    let delete_request = store.clone();
    view! {
        <CsvActionRail
            input_id=input_id
            on_file_select=move |file| file_select.select_file(tab, file)
            selected_file_name=selected_file_name
            file_input_disabled=file_input_disabled
            has_csv_file=has_csv_file
            save_label=save_label
            on_save=move || save.save_csv(tab)
            save_disabled=save_disabled
            has_db_data=has_db_data
            delete_label=delete_label
            on_delete_request=move || delete_request.open_delete_confirm(tab)
            delete_disabled=delete_disabled
            save_result=save_result
            mode_label="追加保存"
            section_class="sm:rounded-t-xl"
        />
    }
}

pub(crate) fn empty_hint(tab: ReceiptsTab) -> &'static str {
    match tab {
        ReceiptsTab::Dividend => "配当金明細をCSVで追加してください",
        ReceiptsTab::DomesticStock => "国内株式明細をCSVで追加してください",
        ReceiptsTab::MutualFund => "投資信託明細をCSVで追加してください",
    }
}
