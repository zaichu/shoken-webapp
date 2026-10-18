use crate::api::dto::CsvUploadResponse;
use crate::support::csv_flow::{CsvTabMeta, CsvUploadResponseExt};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::csv_preview::{
    CsvPreviewNotice, CsvRowErrorList, shared_message, should_show_preview_notice,
};
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, LoadingStrip};
use leptos::prelude::*;

// 全件置換ではエラー行を含む CSV を backend が拒否するため、有効行があっても保存を止める
pub(crate) fn save_disabled_by_preview(meta: &CsvTabMeta, errors_block_save: bool) -> bool {
    meta.busy()
        || !meta.has_preview_rows()
        || (errors_block_save
            && meta
                .preview
                .as_ref()
                .is_some_and(|preview| !preview.errors.is_empty()))
}

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
    /// エラー付きプレビューでは保存を止めるか(全件置換など一部保存ができない方式)
    fn errors_block_save(&self) -> bool {
        false
    }
    // 検証前・検証失敗・全行エラー(有効行0)では保存を送らない
    fn save_disabled(&self, meta: &CsvTabMeta) -> bool {
        save_disabled_by_preview(meta, self.errors_block_save())
    }
    /// ファイル選択の下に出す、想定するCSV形式の案内
    fn csv_format_hint(&self) -> &'static str;
    /// 保存済み全件数。削除ページで対象件数を示すために使う
    fn db_count(&self) -> usize;
    /// 保存済み全件数が確定したか(一覧が未取得・取得失敗なら false で削除を止める)
    fn delete_count_ready(&self) -> bool;
    /// 全件数が確定できない理由(一覧取得の失敗表示用)
    fn delete_count_error(&self) -> Option<String> {
        None
    }
    /// 保存済みデータの日付範囲(最小・最大)。取得済み行から算出し、未取得なら None
    fn date_range(&self) -> Option<(String, String)> {
        None
    }
    /// 削除件数の再取得を要求する(件数取得失敗時の再読み込み用)
    fn refresh_for_delete(&self) {}
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
    /// 削除専用ページへの導線(旧 URL 互換用。カード内のボタンでモーダルを開くため未使用)
    #[prop(into)]
    delete_href: String,
    /// 検証済みプレビュー行の描画。対象ごとの一覧部品を呼び出し側で組む
    preview: AnyView,
    /// AI相談文コピーなど対象ごとの追加領域
    #[prop(optional)]
    extra: Option<AnyView>,
) -> impl IntoView {
    let subject_for_modal = subject.clone();
    let _delete_href = delete_href;
    let selected_file_name = Memo::new(move |_| source.csv_meta().file_name.unwrap_or_default());
    let file_input_disabled = Memo::new(move |_| source.input_disabled());
    let previewing = Memo::new(move |_| source.csv_meta().previewing);
    let meta_error = Memo::new(move |_| source.csv_meta().error);
    let save_result = Memo::new(move |_| source.csv_meta().import_result);
    let db_count = Memo::new(move |_| source.db_count());
    let delete_count_ready = Memo::new(move |_| source.delete_count_ready());
    let delete_count_error = Memo::new(move |_| source.delete_count_error());
    let date_range = Memo::new(move |_| source.date_range());
    let preview_notice = Memo::new(move |_| {
        let meta = source.csv_meta();
        if should_show_preview_notice(
            source.is_authenticated(),
            meta.file_name.is_some(),
            meta.previewing,
        ) {
            meta.preview
                .map(|preview| (preview.valid_rows, preview.errors))
        } else {
            None
        }
    });
    // 削除確認モーダルの表示状態
    let show_delete_confirm = RwSignal::new(false);
    let deleting = Memo::new(move |_| source.csv_meta().deleting);
    let delete_error = RwSignal::new(None::<String>);

    let open_delete_confirm = move |_| {
        source.open_delete_confirm();
        show_delete_confirm.set(true);
    };

    // S: Copy なのでクローンしてクロージャに移動できる
    let confirm_source = source;
    let on_confirm_delete = move || {
        confirm_source.confirm_delete_all();
    };

    let on_cancel_delete = move || {
        if !deleting.get() {
            show_delete_confirm.set(false);
            delete_error.set(None);
        }
    };

    // 削除エラーを CSV メタから同期
    Effect::new(move |_| {
        if let Some(err) = meta_error.get() {
            if source.csv_meta().deleting || source.csv_meta().show_delete_confirm {
                delete_error.set(Some(err));
            }
        }
    });

    // 削除完了時にモーダルを閉じる(成功時: deleting が true から false に変化し、errorなし)
    let was_deleting = RwSignal::new(false);
    Effect::new(move |_| {
        let deleting_now = deleting.get();
        if deleting_now {
            was_deleting.set(true);
        } else if was_deleting.get() && show_delete_confirm.get() && delete_error.get().is_none() {
            // 削除中だったのが完了し、エラーもない -> モーダルを閉じる
            was_deleting.set(false);
            show_delete_confirm.set(false);
            delete_error.set(None);
        }
    });

    // スマホは下部の常設操作帯、PC は操作帯を出さず本文の末尾に同じボタンを置く
    let action_bar = view! {
        <div class="flex gap-2 py-2">
            <CsvActions source=source />
        </div>
    }
    .into_any();

    view! {
        <PageFrame
            title="CSV管理"
            back_href=back_href
            back_label="一覧に戻る"
            action_bar=action_bar
            action_bar_mobile_only=true
        >
            <div class="flex flex-col gap-4">
                <p class="text-sm text-text-soft">
                    "対象: " <strong class="text-text">{subject.clone()}</strong>
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
                <p class="text-xs text-text-muted">{source.csv_format_hint()}</p>
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
                    preview_notice.get().map(|(valid_rows, errors)| {
                        view! {
                            <CsvPreviewNotice
                                valid_rows=valid_rows
                                save_action=source.save_action()
                                errors=errors
                                errors_block_save=source.errors_block_save()
                            />
                        }
                    })
                }}
                {preview}
                {move || {
                    save_result.get().map(|result| {
                        view! { <CsvSaveResultNotice result=result mode_label=source.mode_label() /> }
                    })
                }}
                {extra}
                <section class="border-t border-border-subtle pt-4">
                    <div class="bg-surface-sunken rounded-lg border border-border p-4">
                        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                            <div>
                                <p class="text-xs text-text-soft">"保存済みの" {subject.clone()}</p>
                                <div class="flex items-baseline gap-2 mt-0.5">
                                    <span class="text-2xl font-bold text-text" data-testid="saved-count">
                                        {move || {
                                            if delete_count_ready.get() {
                                                format!("{}件", db_count.get())
                                            } else if delete_count_error.get().is_some() {
                                                "取得できませんでした".to_string()
                                            } else {
                                                "取得中...".to_string()
                                            }
                                        }}
                                    </span>
                                    {move || {
                                        date_range.get().map(|(min, max)| {
                                            view! {
                                                <span class="text-sm font-medium text-text-muted ml-2">
                                                    {format!("{} 〜 {}", min, max)}
                                                </span>
                                            }
                                        })
                                    }}
                                </div>
                            </div>
                            <Button
                                variant=ButtonVariant::Secondary(ButtonSize::Md)
                                class="text-negative border-negative border"
                                disabled=move || !delete_count_ready.get() || db_count.get() == 0 || source.csv_meta().busy()
                                on_click=open_delete_confirm
                            >
                                "すべて削除…"
                            </Button>
                        </div>
                        {move || {
                            delete_count_error.get().map(|message| {
                                view! {
                                    <div class="mt-3">
                                        <Alert variant=AlertVariant::Danger>
                                            <strong>"エラー:"</strong>
                                            " "
                                            {message}
                                            <button
                                                class="ml-2 text-xs underline"
                                                on:click=move |_| source.refresh_for_delete()
                                            >
                                                "再読み込み"
                                            </button>
                                        </Alert>
                                    </div>
                                }
                            })
                        }}
                    </div>
                </section>
                <div class="hidden gap-2 lg:flex">
                    <CsvActions source=source />
                </div>
            </div>

            {move || {
                show_delete_confirm.get().then(|| {
                    let count = db_count.get();
                    let range = date_range.get();
                    let range_text = range.map(|(min, max)| format!("{} 〜 {} の", min, max)).unwrap_or_default();
                    let description = format!("{}データが、一覧と集計から消えます。", range_text);
                    let confirm_label = format!("{}件を削除する", count);
                    view! {
                        <ConfirmDeleteModal
                            title=format!("{}を削除しますか?", subject_for_modal)
                            description=description
                            item_count=count
                            confirm_label=confirm_label
                            loading=deleting
                            error=delete_error
                            on_confirm=on_confirm_delete
                            on_cancel=on_cancel_delete
                        />
                    }
                })
            }}
        </PageFrame>
    }
}

/// プレビューの破棄と保存。スマホは下部操作帯、PC は本文末尾に同じボタンを出す
#[component]
fn CsvActions<S: CsvSource>(source: S) -> impl IntoView {
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
    view! {
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
            class="flex-1 lg:flex-none"
            disabled=move || save_disabled.get()
            aria_disabled=move || save_disabled.get()
            on_click=move |_| save.save_csv()
        >
            {move || save_label.get()}
        </Button>
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

// 成功・一部失敗・全失敗をアイコンと色で分ける(文言の違いだけでは区別しにくいため)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SaveResultTone {
    Success,
    Partial,
    Failure,
}

pub(crate) fn save_result_tone(result: &CsvUploadResponse) -> SaveResultTone {
    if result.errors.is_empty() {
        SaveResultTone::Success
    } else if result.inserted == 0 {
        SaveResultTone::Failure
    } else {
        SaveResultTone::Partial
    }
}

const SAVE_RESULT_STYLES: [(&str, &str, &str); 3] = [
    (
        "rounded-note border px-4 py-3.5 border-positive-border/80 bg-positive-soft/80",
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-surface border-positive-border text-positive-strong",
        "M5.5 10.5l3 3 6-7",
    ),
    (
        "rounded-note border px-4 py-3.5 border-accent-border/80 bg-accent-soft/80",
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-surface border-accent-border text-accent-strong",
        "M10 5.5v5m0 3.5h.01",
    ),
    (
        "rounded-note border px-4 py-3.5 border-negative-border/80 bg-negative-soft/80",
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-surface border-negative-border text-negative-strong",
        "M5.5 5.5l9 9m0-9l-9 9",
    ),
];

#[component]
fn CsvSaveResultNotice(result: CsvUploadResponse, mode_label: &'static str) -> impl IntoView {
    let has_errors = !result.errors.is_empty();
    let aggregated = shared_message(&result.errors).is_some();
    let index = match save_result_tone(&result) {
        SaveResultTone::Success => 0,
        SaveResultTone::Partial => 1,
        SaveResultTone::Failure => 2,
    };
    let (section_class, icon_class, icon_path) = SAVE_RESULT_STYLES[index];
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
                            d=icon_path
                        />
                    </svg>
                </span>
                <div class="min-w-0 flex-1">
                    <p class="text-sm font-semibold text-text">{heading}</p>
                    <p class="mt-1 text-sm text-text-soft">{line}</p>
                    {has_errors.then(|| {
                        let all_failed = result.inserted == 0 && result.skipped == 0;
                        let list = view! {
                            <CsvRowErrorList errors=result.errors.clone() all_failed=all_failed />
                        };
                        if aggregated {
                            view! { <div class="mt-3 text-sm text-text-muted">{list}</div> }
                                .into_any()
                        } else {
                            view! {
                                <details class="mt-3 rounded-2xl border border-accent-border/80 bg-surface/80 px-3 py-2">
                                    <summary class="cursor-pointer text-sm font-medium text-text-soft">
                                        "エラー詳細を表示"
                                    </summary>
                                    <div class="text-sm text-text-muted">{list}</div>
                                </details>
                            }
                            .into_any()
                        }
                    })}
                </div>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests;
