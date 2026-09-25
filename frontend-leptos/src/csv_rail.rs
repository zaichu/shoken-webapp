use crate::csv_flow::row_error_text;
use crate::dto::CsvUploadResponse;
use leptos::prelude::*;

const PRIMARY_BUTTON_CLASS: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print border border-slate-950 bg-slate-950 text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.12)] hover:bg-slate-800 active:bg-slate-950 px-3 py-1.5 text-sm max-sm:min-h-[44px] h-11 w-full rounded-md text-sm font-bold";
const DELETE_BUTTON_CLASS: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print border border-danger text-danger hover:bg-danger hover:text-white px-3 py-1.5 text-sm max-sm:min-h-[44px] h-11 w-full rounded-md text-sm font-bold";
const BADGE_CLASS: &str = "inline-flex items-center rounded-full border border-slate-200 bg-white px-2.5 py-1 text-xs font-semibold text-slate-700";
const BADGE_MUTED_CLASS: &str = "inline-flex items-center rounded-full border border-slate-200 bg-white px-2.5 py-1 text-xs font-medium text-slate-600";
const BADGE_WARN_CLASS: &str = "inline-flex items-center rounded-full border border-amber-200 bg-white px-2.5 py-1 text-xs font-medium text-amber-700";

#[component]
pub fn CsvActionRail(
    input_id: &'static str,
    on_file_select: impl Fn(web_sys::File) + Clone + 'static,
    selected_file_name: Memo<String>,
    file_input_disabled: Memo<bool>,
    has_csv_file: Memo<bool>,
    save_label: Memo<String>,
    on_save: impl Fn() + Clone + Send + Sync + 'static,
    save_disabled: Memo<bool>,
    has_db_data: Memo<bool>,
    delete_label: Memo<String>,
    on_delete_request: impl Fn() + Clone + Send + Sync + 'static,
    delete_disabled: Memo<bool>,
    save_result: Memo<Option<CsvUploadResponse>>,
    mode_label: &'static str,
) -> impl IntoView {
    view! {
        <section
            class="space-y-3 bg-slate-50/60 px-5 py-5"
            role="group"
            aria-label="データ操作"
        >
            <div class="space-y-3">
                <CsvFileInput
                    input_id=input_id
                    on_file_select=on_file_select
                    selected_file_name=selected_file_name
                    disabled=file_input_disabled
                />
                {move || {
                    if has_csv_file.get() {
                        let on_save = on_save.clone();
                        view! {
                            <button
                                type="button"
                                class=PRIMARY_BUTTON_CLASS
                                disabled=move || save_disabled.get()
                                aria-disabled=move || save_disabled.get()
                                on:click=move |_| on_save()
                            >
                                {move || save_label.get()}
                            </button>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}
                {move || {
                    if has_db_data.get() {
                        let on_delete_request = on_delete_request.clone();
                        view! {
                            <button
                                type="button"
                                class=DELETE_BUTTON_CLASS
                                disabled=move || delete_disabled.get()
                                aria-disabled=move || delete_disabled.get()
                                on:click=move |_| on_delete_request()
                            >
                                {move || delete_label.get()}
                            </button>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}
                {move || {
                    save_result
                        .get()
                        .map(|result| view! { <CsvSaveResultNotice result=result mode_label=mode_label /> })
                }}
            </div>
        </section>
    }
}

#[component]
fn CsvFileInput(
    input_id: &'static str,
    on_file_select: impl Fn(web_sys::File) + 'static,
    selected_file_name: Memo<String>,
    disabled: Memo<bool>,
) -> impl IntoView {
    let on_change = move |event: web_sys::Event| {
        if disabled.get_untracked() {
            return;
        }
        let input = event_target::<web_sys::HtmlInputElement>(&event);
        if let Some(file) = input.files().and_then(|files| files.get(0)) {
            on_file_select(file);
            // 同じファイルを続けて選択できるよう値をリセットする
            input.set_value("");
        }
    };
    let label_class = move || {
        if disabled.get() {
            "flex min-h-20 items-center justify-between gap-3 rounded-[1.35rem] border border-dashed border-slate-300 bg-slate-50 px-4 py-3 transition-colors pointer-events-none opacity-65"
        } else {
            "flex min-h-20 cursor-pointer items-center justify-between gap-3 rounded-[1.35rem] border border-dashed border-slate-300 bg-slate-50 px-4 py-3 transition-colors hover:border-slate-400 hover:bg-slate-100"
        }
    };
    view! {
        <div class="space-y-2.5">
            <label data-testid="csv-file-trigger" class=label_class for=input_id>
                <div class="flex min-w-0 items-center gap-3">
                    <span class="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-2xl bg-white text-slate-600 shadow-sm">
                        <svg
                            class="h-5 w-5"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            aria-hidden="true"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="1.8"
                                d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M12 12v6m0-6l-2.5 2.5M12 12l2.5 2.5"
                            />
                        </svg>
                    </span>
                    <div class="min-w-0">
                        <p class="text-sm font-semibold text-slate-800">"CSVファイルを選択"</p>
                    </div>
                </div>
                <span class="inline-flex shrink-0 items-center rounded-full border border-slate-200 bg-white px-3 py-1.5 text-sm font-semibold text-slate-700 shadow-sm">
                    "参照"
                </span>
            </label>
            <input
                id=input_id
                data-testid="csv-file-input"
                type="file"
                accept=".csv"
                class="hidden"
                on:change=on_change
                disabled=move || disabled.get()
                aria-label="CSVファイルを選択"
            />
            <input
                type="text"
                class="w-full rounded-[1.15rem] border border-slate-200 bg-white px-3.5 py-2.5 text-sm text-slate-600 shadow-sm disabled:cursor-not-allowed disabled:bg-slate-100"
                readonly
                placeholder="ファイル未選択"
                prop:value=move || selected_file_name.get()
                disabled=move || disabled.get()
                aria-label="選択されたファイル名"
            />
        </div>
    }
}

#[component]
fn CsvSaveResultNotice(result: CsvUploadResponse, mode_label: &'static str) -> impl IntoView {
    let has_errors = !result.errors.is_empty();
    let section_class = if has_errors {
        "rounded-[1.35rem] border px-4 py-3.5 border-amber-200/80 bg-amber-50/80"
    } else {
        "rounded-[1.35rem] border px-4 py-3.5 border-emerald-200/80 bg-emerald-50/80"
    };
    let icon_class = if has_errors {
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-white border-amber-200 text-amber-600"
    } else {
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-white border-emerald-200 text-emerald-600"
    };
    view! {
        <section
            class=section_class
            data-testid="csv-save-result-notice"
            role="status"
            aria-live="polite"
        >
            <div class="flex items-start gap-3">
                <span class=icon_class aria-hidden="true">
                    <svg class="h-4.5 w-4.5" viewBox="0 0 20 20" fill="none" stroke="currentColor">
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="1.8"
                            d="M5.5 10.5l3 3 6-7"
                        />
                    </svg>
                </span>
                <div class="min-w-0 flex-1">
                    <p class="text-sm font-semibold text-slate-800">"保存しました"</p>
                    <div class="mt-2 flex flex-wrap gap-2">
                        <span class=BADGE_CLASS>{result.inserted_text()}</span>
                        <span class=BADGE_MUTED_CLASS>{mode_label}</span>
                        {result
                            .skipped_text()
                            .map(|text| view! { <span class=BADGE_MUTED_CLASS>{text}</span> })}
                        {result
                            .error_count_text()
                            .map(|text| view! { <span class=BADGE_WARN_CLASS>{text}</span> })}
                    </div>
                    {has_errors.then(|| {
                        view! {
                            <details class="mt-3 rounded-[1rem] border border-amber-200/80 bg-white/80 px-3 py-2">
                                <summary class="cursor-pointer text-sm font-medium text-slate-700">
                                    "エラー詳細を表示"
                                </summary>
                                <ul class="mt-2 space-y-1 text-sm text-slate-600">
                                    {result
                                        .errors
                                        .iter()
                                        .map(|error| view! { <li>{row_error_text(error)}</li> })
                                        .collect_view()}
                                </ul>
                            </details>
                        }
                    })}
                </div>
            </div>
        </section>
    }
}
