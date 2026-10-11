use crate::api::dto::CsvUploadResponse;
use crate::support::csv_flow::{CsvTabMeta, CsvUploadResponseExt, row_error_text};
use crate::ui::button::{Button, ButtonSize, ButtonVariant, LinkButton};
use crate::ui::csv_preview::{CsvPreviewNotice, preview_notice_text, should_show_preview_notice};
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, LoadingStrip};
use leptos::prelude::*;

/// CSV 管理・削除ページが要求するデータソース。
/// 取引明細(種別付き)と資産残高が同じページ構成を共有するため、
/// 状態取得と操作をトレイトで受け取る。状態は行データを含まない CsvTabMeta
pub trait CsvSource: Copy + Send + Sync + 'static {
    fn input_id(&self) -> &'static str;
    /// 保存ボタンの方式込みラベル("追加で保存" / "全件置換で保存")
    fn save_action(&self) -> &'static str;
    /// 「保存方式」の短い表示名("追加" / "全件置換")
    fn save_mode_label(&self) -> &'static str;
    /// 保存結果通知の方式表示("追加保存" / "全件置換")
    fn mode_label(&self) -> &'static str;
    fn csv_meta(&self) -> CsvTabMeta;
    fn is_authenticated(&self) -> bool;
    // 未認証・取得中・CSV 処理中など、ファイル選択を受け付けられない状態の判定
    fn input_disabled(&self) -> bool;
    fn save_disabled(&self, meta: &CsvTabMeta) -> bool {
        meta.busy()
    }
    /// 保存済み全件数。削除ページで対象件数を示すために使う
    fn db_count(&self) -> usize;
    /// 保存済み全件数が確定したか(一覧が未取得・取得失敗なら false で削除を止める)
    fn delete_count_ready(&self) -> bool;
    /// 全件数が確定できない理由(一覧取得の失敗表示用)
    fn delete_count_error(&self) -> Option<String> {
        None
    }
    fn delete_disabled(&self, meta: &CsvTabMeta) -> bool {
        meta.saving || meta.deleting
    }
    fn select_file(&self, file: web_sys::File);
    fn save_csv(&self);
    fn discard_csv(&self);
    fn open_delete_confirm(&self);
    fn confirm_delete_all(&self);
}

/// CSV の選択→検証→保存までを1ページに置く管理画面。
/// アップロードと削除を同じ領域に並べないため、削除は区切りの先の導線だけにする
#[component]
pub fn CsvManagePage<S: CsvSource>(
    source: S,
    /// 「対象」欄に出す名前(配当金・資産管理 など)
    #[prop(into)]
    subject: String,
    /// 一覧への戻り先
    back_href: &'static str,
    /// 削除専用ページへの導線
    #[prop(into)]
    delete_href: String,
    /// 検証済みプレビュー行の描画。対象ごとの一覧部品を呼び出し側で組む
    preview: AnyView,
    /// AI相談文コピーなど対象ごとの追加領域
    #[prop(optional)]
    extra: Option<AnyView>,
) -> impl IntoView {
    let selected_file_name = Memo::new(move |_| source.csv_meta().file_name.unwrap_or_default());
    let file_input_disabled = Memo::new(move |_| source.input_disabled());
    let previewing = Memo::new(move |_| source.csv_meta().previewing);
    let meta_error = Memo::new(move |_| source.csv_meta().error);
    let save_result = Memo::new(move |_| source.csv_meta().import_result);
    let db_count = Memo::new(move |_| source.db_count());
    let preview_notice = Memo::new(move |_| {
        let meta = source.csv_meta();
        if should_show_preview_notice(
            source.is_authenticated(),
            meta.file_name.is_some(),
            meta.previewing,
        ) {
            meta.preview.map(|preview| {
                (
                    preview_notice_text(preview.valid_rows, source.save_action()),
                    preview.errors,
                )
            })
        } else {
            None
        }
    });
    let save_label = Memo::new(move |_| source.csv_meta().save_label(source.save_action()));
    let save_disabled = Memo::new(move |_| {
        let meta = source.csv_meta();
        !source.is_authenticated() || meta.file_name.is_none() || source.save_disabled(&meta)
    });
    let discard_disabled = Memo::new(move |_| {
        let meta = source.csv_meta();
        meta.file_name.is_none() || meta.busy()
    });
    let save = source;
    let discard = source;
    let action_bar = view! {
        <div class="flex gap-2 py-2">
            <Button
                variant=ButtonVariant::Secondary(ButtonSize::Md)
                disabled=move || discard_disabled.get()
                aria_disabled=move || discard_disabled.get()
                on_click=move |_| discard.discard_csv()
            >
                "プレビューを破棄"
            </Button>
            <Button
                variant=ButtonVariant::Primary(ButtonSize::Md)
                class="flex-1"
                disabled=move || save_disabled.get()
                aria_disabled=move || save_disabled.get()
                on_click=move |_| save.save_csv()
            >
                {move || save_label.get()}
            </Button>
        </div>
    }
    .into_any();

    view! {
        <PageFrame
            title="CSV管理"
            back_href=back_href
            back_label="一覧に戻る"
            action_bar=action_bar
        >
            <div class="flex flex-col gap-4">
                <p class="text-sm text-text-soft">
                    "対象: " <strong class="text-text">{subject}</strong>
                    <span class="mx-2" aria-hidden="true">
                        "/"
                    </span>
                    "保存方式: " <strong class="text-text">{source.save_mode_label()}</strong>
                </p>
                <CsvFileInput
                    input_id=source.input_id()
                    on_file_select=move |file| source.select_file(file)
                    selected_file_name=selected_file_name
                    disabled=file_input_disabled
                />
                {move || {
                    previewing.get().then(|| {
                        view! { <LoadingStrip text="CSVファイルを解析しています...".to_string() /> }
                    })
                }}
                {move || {
                    meta_error.get().map(|message| {
                        view! {
                            <Alert variant=AlertVariant::Danger>
                                <strong>"エラー:"</strong>
                                " "
                                {message}
                            </Alert>
                        }
                    })
                }}
                {move || {
                    preview_notice
                        .get()
                        .map(|(text, errors)| view! { <CsvPreviewNotice text=text errors=errors /> })
                }}
                {preview}
                {move || {
                    save_result.get().map(|result| {
                        view! { <CsvSaveResultNotice result=result mode_label=source.mode_label() /> }
                    })
                }}
                {extra}
                <section class="border-t border-border-subtle pt-4">
                    <h2 class="text-sm font-bold text-text">"データ削除"</h2>
                    <p class="mt-1 text-sm text-text-soft">
                        {move || {
                            // 未取得・取得失敗を 0件 と見せると空と誤認するため、確定まで件数を出さない
                            if source.delete_count_ready() {
                                format!("保存済みデータ: {}件", db_count.get())
                            } else if source.delete_count_error().is_some() {
                                "保存済みデータ: 取得できませんでした".to_string()
                            } else {
                                "保存済みデータ: 取得中...".to_string()
                            }
                        }}
                    </p>
                    <div class="mt-3">
                        <LinkButton
                            variant=ButtonVariant::Secondary(ButtonSize::Md)
                            href=delete_href
                        >
                            "保存済みデータを削除する画面へ"
                        </LinkButton>
                    </div>
                </section>
            </div>
        </PageFrame>
    }
}

/// 全件削除の専用ページ。遷移自体が確認導線で、対象と保存済み全件数を示してから実行する
#[component]
pub fn CsvDeletePage<S: CsvSource>(
    source: S,
    /// 「対象」欄に出す名前(配当金・資産管理 など)
    #[prop(into)]
    subject: String,
    /// CSV管理ページへの戻り先(種別を保つため動的 href)
    #[prop(into)]
    csv_href: String,
    /// 削除後に戻る一覧
    list_href: &'static str,
) -> impl IntoView {
    // 実行を押したか。押下後に deleting が下りて error も無ければ完了とみなす
    let attempted = RwSignal::new(false);
    let count_ready = Memo::new(move |_| source.delete_count_ready());
    let count = Memo::new(move |_| source.db_count());
    let count_error = Memo::new(move |_| source.delete_count_error());
    let deleting = Memo::new(move |_| source.csv_meta().deleting);
    let meta_error = Memo::new(move |_| source.csv_meta().error);
    let done = Memo::new(move |_| attempted.get() && !deleting.get() && meta_error.get().is_none());
    let delete_disabled = Memo::new(move |_| {
        let meta = source.csv_meta();
        !source.delete_count_ready() || source.db_count() == 0 || source.delete_disabled(&meta)
    });
    let confirm = source;
    let on_confirm = move |_| {
        attempted.set(true);
        // 削除開始は「確認表示中のみ」というストア制約に合わせ、専用ページへの遷移を確認として扱う
        confirm.open_delete_confirm();
        confirm.confirm_delete_all();
    };
    let warning = Signal::stored(format!("{subject}の保存済みデータをすべて削除します。"));

    view! {
        <PageFrame title="データ削除" back_href=csv_href back_label="CSV管理に戻る">
            <div class="flex flex-col gap-4" data-testid="csv-delete-page">
                <dl class="flex flex-col gap-1 text-sm">
                    <div class="flex gap-2">
                        <dt class="text-text-soft">"対象:"</dt>
                        <dd class="font-semibold text-text">{subject}</dd>
                    </div>
                    <div class="flex gap-2" data-testid="csv-delete-count">
                        <dt class="text-text-soft">"保存済みデータ:"</dt>
                        <dd class="font-semibold text-text">
                            {move || {
                                if count_ready.get() {
                                    format!("{}件", count.get())
                                } else if count_error.get().is_some() {
                                    "取得できませんでした".to_string()
                                } else {
                                    "取得中...".to_string()
                                }
                            }}
                        </dd>
                    </div>
                </dl>
                {move || {
                    count_error.get().map(|message| {
                        view! {
                            <Alert variant=AlertVariant::Danger>
                                <strong>"エラー:"</strong>
                                " "
                                {message}
                            </Alert>
                        }
                    })
                }}
                {move || {
                    meta_error.get().map(|message| {
                        view! {
                            <Alert variant=AlertVariant::Danger>
                                <strong>"エラー:"</strong>
                                " "
                                {message}
                            </Alert>
                        }
                    })
                }}
                {move || {
                    if done.get() {
                        view! {
                            <div class="flex flex-col gap-4" data-testid="csv-delete-done">
                                <section
                                    class="rounded-note border px-4 py-3.5 border-positive-border/80 bg-positive-soft/80"
                                    role="status"
                                >
                                    <p class="text-sm font-semibold text-text">"削除しました。"</p>
                                </section>
                                <div>
                                    <LinkButton
                                        variant=ButtonVariant::Primary(ButtonSize::Md)
                                        href=list_href
                                    >
                                        "一覧に戻る"
                                    </LinkButton>
                                </div>
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            <Alert variant=AlertVariant::Warning>
                                {warning.get()}
                                "この操作は取り消せません。"
                            </Alert>
                            <div>
                                <Button
                                    variant=ButtonVariant::DangerSolid(ButtonSize::Md)
                                    disabled=move || delete_disabled.get()
                                    aria_disabled=move || delete_disabled.get()
                                    on_click=on_confirm
                                >
                                    {move || source.csv_meta().delete_label()}
                                </Button>
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </PageFrame>
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
            "flex min-h-20 items-center gap-3 rounded-lg border border-dashed border-border-strong bg-surface-sunken px-4 py-3 transition-colors pointer-events-none opacity-65"
        } else {
            "flex min-h-20 cursor-pointer items-center gap-3 rounded-lg border border-dashed border-border-strong bg-surface-sunken px-4 py-3 transition-colors hover:border-border-xstrong hover:bg-surface-raised focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-ink"
        }
    };
    view! {
        <label data-testid="csv-file-trigger" class=label_class for=input_id>
            // ラベル内に置かないと focus-within が発火せず、見えるファイル選択枠にフォーカス枠が出ない
            <input
                id=input_id
                data-testid="csv-file-input"
                type="file"
                accept=".csv"
                class="sr-only"
                on:change=on_change
                disabled=move || disabled.get()
                aria-label="CSVファイルを選択"
                // 入力の値は同じファイルの再選択のために消すので、読み上げは表示中のファイル名へ逃がす
                aria-describedby=format!("{input_id}-filename")
            />
            <div class="flex min-w-0 items-center gap-3">
                <span class="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-surface text-text-muted shadow-sm">
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
                    <p class="whitespace-nowrap text-sm font-semibold text-text">"CSVファイルを選択"</p>
                    <p
                        id=format!("{input_id}-filename")
                        class="truncate text-xs font-medium text-text-muted"
                    >
                        {move || {
                            let name = selected_file_name.get();
                            if name.is_empty() {
                                "ファイル未選択".to_string()
                            } else {
                                name
                            }
                        }}
                    </p>
                </div>
            </div>
        </label>
    }
}

// 成功・一部失敗・全行失敗を見出しで分ける(件数の有無だけでは区別しにくいため)
pub(crate) fn save_result_heading(result: &CsvUploadResponse) -> &'static str {
    if result.errors.is_empty() {
        "保存しました"
    } else if result.inserted == 0 {
        "保存できませんでした"
    } else {
        "一部の行を保存できませんでした"
    }
}

// 件数はバッジの集合ではなく一文で伝える
pub(crate) fn save_result_line(result: &CsvUploadResponse, mode_label: &str) -> String {
    [
        Some(result.inserted_text()),
        Some(mode_label.to_string()),
        result.skipped_text(),
        result.error_count_text(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("・")
}

#[component]
fn CsvSaveResultNotice(result: CsvUploadResponse, mode_label: &'static str) -> impl IntoView {
    let has_errors = !result.errors.is_empty();
    let section_class = if has_errors {
        "rounded-note border px-4 py-3.5 border-accent-border/80 bg-accent-soft/80"
    } else {
        "rounded-note border px-4 py-3.5 border-positive-border/80 bg-positive-soft/80"
    };
    let icon_class = if has_errors {
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-surface border-accent-border text-accent-strong"
    } else {
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-surface border-positive-border text-positive-strong"
    };
    let heading = save_result_heading(&result);
    let line = save_result_line(&result, mode_label);
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
                    <p class="text-sm font-semibold text-text">{heading}</p>
                    <p class="mt-1 text-sm text-text-soft">{line}</p>
                    {has_errors.then(|| {
                        view! {
                            <details class="mt-3 rounded-2xl border border-accent-border/80 bg-surface/80 px-3 py-2">
                                <summary class="cursor-pointer text-sm font-medium text-text-soft">
                                    "エラー詳細を表示"
                                </summary>
                                <ul class="mt-2 space-y-1 text-sm text-text-muted">
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

#[cfg(test)]
mod tests;
