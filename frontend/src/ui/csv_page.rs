use crate::api::dto::CsvUploadResponse;
use crate::support::csv_flow::{CsvTabMeta, CsvUploadResponseExt};
use crate::ui::button::{Button, ButtonSize, ButtonVariant, LinkButton};
use crate::ui::csv_preview::{
    CsvPreviewNotice, CsvRowErrorList, preview_notice_text, shared_message,
    should_show_preview_notice,
};
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, LoadingStrip};
use leptos::ev::DragEvent;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

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
    /// 検証結果の通知で使う方式込みの動詞("追加で保存" / "全件置換で保存")
    fn save_action(&self) -> &'static str;
    /// 「保存方式」の短い表示名("追加" / "全件置換")
    fn save_mode_label(&self) -> &'static str;
    /// 操作バーの保存ボタンのラベル。プレビューが無い間は件数を言えないので None
    fn save_button_label(&self, valid_rows: Option<usize>) -> String {
        match valid_rows {
            Some(rows) => format!("{rows}件を保存する"),
            None => "保存する".to_string(),
        }
    }
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
    /// 操作バーに出す、想定するCSV形式の案内
    fn csv_format_hint(&self) -> &'static str;
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
    fn open_delete_confirm(&self);
    fn confirm_delete_all(&self);
}

/// CSV の選択→検証→保存までを1ページに置く管理画面。
/// 操作バーは1か所だけで、削除への導線は保存の手前に来ない位置(ページ名の右)に置く
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
    let previewing = Memo::new(move |_| source.csv_meta().previewing);
    let saving = Memo::new(move |_| source.csv_meta().saving);
    let meta_error = Memo::new(move |_| source.csv_meta().error);
    let save_result = Memo::new(move |_| source.csv_meta().import_result);
    let preview_notice = Memo::new(move |_| {
        let meta = source.csv_meta();
        if should_show_preview_notice(
            source.is_authenticated(),
            meta.file_name.is_some(),
            meta.previewing,
        ) {
            meta.preview
                .map(|preview| (preview.valid_rows, preview.errors))
                // 問題のないプレビューは操作バーに案内を出すため、通知は出すべきときだけ出す
                .filter(|(valid_rows, errors)| *valid_rows == 0 || !errors.is_empty())
        } else {
            None
        }
    });

    // 保存の再送待ちは ApiClient の内部で完結するため、フック経由で受けて
    // 操作バーの保存ボタンに「再試行中...」を出す
    let retrying = RwSignal::new(false);
    crate::api::client::set_retry_hook(move || retrying.set(true));
    Effect::new(move |_| {
        if !saving.get() && retrying.get_untracked() {
            retrying.set(false);
        }
    });

    // 選んだ直後のフォーカス先: 保存できるなら保存ボタン(Enter で保存)、
    // そうでなければ選び直せるファイル入力へ戻す(失敗時にフォーカスが迷子にならないように)
    fn focus_element(selector: &'static str) {
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(0).await;
            let element = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.query_selector(selector).ok())
                .flatten();
            if let Some(element) = element {
                let _ = element.unchecked_into::<web_sys::HtmlElement>().focus();
            }
        });
    }
    let prev_previewing = RwSignal::new(false);
    Effect::new(move |_| {
        let previewing = previewing.get();
        let was = prev_previewing.get_untracked();
        prev_previewing.set(previewing);
        if !was || previewing {
            return;
        }
        let meta = source.csv_meta();
        if meta.preview.is_some() && !source.save_disabled(&meta) {
            focus_element("[data-testid='csv-save-button']");
        } else {
            focus_element("[data-testid='csv-file-input']");
        }
    });

    let delete_link: Children = Box::new(move || {
        let href = delete_href.clone();
        view! { <a class="page-secondary-link-sm" href=href>"保存済みデータを削除…"</a> }.into_any()
    });
    let action_bar = view! { <CsvActionBar source=source retrying=retrying /> }.into_any();

    view! {
        <PageFrame
            title="CSV管理"
            back_href=back_href
            back_label="一覧に戻る"
            links=delete_link
            action_bar=action_bar
            action_bar_pc_top=true
        >
            <div class="flex flex-col gap-4">
                <p class="text-sm text-text-soft">
                    "対象: " <strong class="text-text">{subject}</strong>
                    <span class="mx-2" aria-hidden="true">
                        "/"
                    </span>
                    "保存方式: " <strong class="text-text">{source.save_mode_label()}</strong>
                </p>
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
                        view! { <CsvSaveResultNotice result=result list_href=back_href /> }
                    })
                }}
                {extra}
            </div>
        </PageFrame>
    }
}

/// ファイルの選択(ドラッグ&ドロップ含む)から保存までを1つのバーに置く。
/// 選ぶ前後でボタンの位置が変わらないため、マウスをほとんど動かさずに済む
#[component]
fn CsvActionBar<S: CsvSource>(source: S, retrying: RwSignal<bool>) -> impl IntoView {
    let input_id = source.input_id();
    let disabled = Memo::new(move |_| source.input_disabled());
    let has_file = Memo::new(move |_| source.csv_meta().file_name.is_some());
    let file_name = Memo::new(move |_| source.csv_meta().file_name.unwrap_or_default());
    let status_text = Memo::new(move |_| {
        let meta = source.csv_meta();
        if meta.previewing {
            Some("解析中...".to_string())
        } else if meta.saving {
            Some(
                if retrying.get() {
                    "再試行中..."
                } else {
                    "保存中..."
                }
                .to_string(),
            )
        } else {
            meta.preview.as_ref().map(|preview| {
                preview_notice_text(
                    preview.valid_rows,
                    preview.errors.len(),
                    source.save_action(),
                    source.errors_block_save(),
                )
            })
        }
    });
    let save_label = Memo::new(move |_| {
        let meta = source.csv_meta();
        if meta.saving {
            if retrying.get() {
                "再試行中..."
            } else {
                "保存中..."
            }
            .to_string()
        } else if meta.previewing {
            "解析中...".to_string()
        } else {
            source.save_button_label(meta.preview.as_ref().map(|preview| preview.valid_rows))
        }
    });
    let save_disabled = Memo::new(move |_| {
        let meta = source.csv_meta();
        !source.is_authenticated() || meta.file_name.is_none() || source.save_disabled(&meta)
    });
    let on_change = move |event: web_sys::Event| {
        if disabled.get_untracked() {
            return;
        }
        let input = event_target::<web_sys::HtmlInputElement>(&event);
        if let Some(file) = input.files().and_then(|files| files.get(0)) {
            source.select_file(file);
            // 同じファイルを続けて選択できるよう値をリセットする
            input.set_value("");
        }
    };
    let on_drop = move |event: DragEvent| {
        event.prevent_default();
        if disabled.get_untracked() {
            return;
        }
        if let Some(file) = event
            .data_transfer()
            .and_then(|transfer| transfer.files())
            .and_then(|files| files.get(0))
        {
            source.select_file(file);
        }
    };
    let pick_button_class = "inline-flex min-h-11 shrink-0 cursor-pointer items-center justify-center rounded-md border border-border-strong bg-surface px-4 text-sm font-bold text-text transition-colors hover:border-border-bold hover:bg-surface-sunken focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-ink";
    let pick_trigger_class = move || {
        if disabled.get() {
            "flex min-h-20 items-center gap-3 rounded-lg border border-dashed border-border-strong bg-surface-sunken px-4 py-3 transition-colors pointer-events-none opacity-65"
        } else {
            "flex min-h-20 cursor-pointer items-center gap-3 rounded-lg border border-dashed border-border-strong bg-surface-sunken px-4 py-3 transition-colors hover:border-border-xstrong hover:bg-surface-raised focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-ink"
        }
    };
    view! {
        <div
            class="flex gap-2 py-2"
            data-testid="csv-action-bar"
            on:dragover=move |event: DragEvent| event.prevent_default()
            on:drop=on_drop
        >
            {move || {
                if has_file.get() {
                    let file_input = view! {
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
                    };
                    view! {
                        // スマホではボタンが次の行に回り込み、ファイル名と状態は1行で読める幅を保つ
                        <div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-border-strong bg-surface px-4 py-3 shadow-sm">
                            <CsvFileIcon />
                            <div class="min-w-0 flex-1 basis-56">
                                <p id=format!("{input_id}-filename") class="truncate text-sm font-bold text-text">
                                    {file_name}
                                </p>
                                {move || {
                                    status_text.get().map(|text| {
                                        view! {
                                            <p class="text-xs font-medium text-text-soft" role="status" aria-live="polite">
                                                {text}
                                            </p>
                                        }
                                    })
                                }}
                            </div>
                            // ラベル内に入力を置かないと focus-within が発火しない
                            <label class=pick_button_class>
                                {file_input}
                                "別のファイル"
                            </label>
                            <Button
                                variant=ButtonVariant::Primary(ButtonSize::Md)
                                testid="csv-save-button"
                                disabled=move || save_disabled.get()
                                aria_disabled=move || save_disabled.get()
                                // 前回の保存以外で立った再試行フラグを、保存の開始時に必ず落とす
                                on_click=move |_| {
                                    retrying.set(false);
                                    source.save_csv();
                                }
                            >
                                {move || save_label.get()}
                            </Button>
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <label class=pick_trigger_class data-testid="csv-file-trigger">
                            <CsvFileIcon />
                            <div class="min-w-0 flex-1">
                                <p class="text-sm font-bold text-text">
                                    "CSVをドラッグ、またはボタンで選ぶ"
                                </p>
                                <p class="text-xs text-text-muted">{source.csv_format_hint()}</p>
                            </div>
                            <span class="inline-flex min-h-11 shrink-0 items-center justify-center rounded-md border border-brand bg-brand px-4 text-sm font-bold text-text-inverse shadow-edge-lit hover:bg-brand-deep">
                                "CSVを選ぶ"
                            </span>
                            <input
                                id=input_id
                                data-testid="csv-file-input"
                                type="file"
                                accept=".csv"
                                class="sr-only"
                                on:change=on_change
                                disabled=move || disabled.get()
                                aria-label="CSVファイルを選択"
                            />
                        </label>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
fn CsvFileIcon() -> impl IntoView {
    view! {
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

// 成功・追加なし・失敗を見出しで分ける(件数の有無だけでは区別しにくいため)
pub(crate) fn save_result_heading(result: &CsvUploadResponse) -> &'static str {
    match save_result_tone(result) {
        SaveResultTone::Success => "保存しました",
        SaveResultTone::NoAddition => "追加するデータはありませんでした",
        SaveResultTone::Failure => {
            if result.inserted == 0 {
                "保存できませんでした"
            } else {
                "一部の行を保存できませんでした"
            }
        }
    }
}

// 件数はバッジの集合ではなく一文で伝える。方式は画面上部に既出のため連結しない
pub(crate) fn save_result_line(result: &CsvUploadResponse) -> String {
    [
        Some(result.inserted_text()),
        result.skipped_text(),
        result.error_count_text(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("・")
}

// 成功・追加なし・失敗をアイコンと色で分ける(文言の違いだけでは区別しにくいため)。
// スキップだけで終わった保存は成功扱いにしない
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SaveResultTone {
    Success,
    NoAddition,
    Failure,
}

pub(crate) fn save_result_tone(result: &CsvUploadResponse) -> SaveResultTone {
    if !result.errors.is_empty() {
        SaveResultTone::Failure
    } else if result.inserted > 0 {
        SaveResultTone::Success
    } else {
        SaveResultTone::NoAddition
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
        "M5.5 8h9m-9 4h9",
    ),
    (
        "rounded-note border px-4 py-3.5 border-negative-border/80 bg-negative-soft/80",
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-surface border-negative-border text-negative-strong",
        "M5.5 5.5l9 9m0-9l-9 9",
    ),
];

#[component]
fn CsvSaveResultNotice(result: CsvUploadResponse, list_href: &'static str) -> impl IntoView {
    let has_errors = !result.errors.is_empty();
    let aggregated = shared_message(&result.errors).is_some();
    let index = match save_result_tone(&result) {
        SaveResultTone::Success => 0,
        SaveResultTone::NoAddition => 1,
        SaveResultTone::Failure => 2,
    };
    let (section_class, icon_class, icon_path) = SAVE_RESULT_STYLES[index];
    let heading = save_result_heading(&result);
    let line = save_result_line(&result);
    let skipped_note = (save_result_tone(&result) == SaveResultTone::NoAddition
        && result.skipped > 0)
        .then(|| {
            format!(
                "{}件はすべて保存済みのため、スキップしました。データは変わりません。",
                result.skipped
            )
        });
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
                    {skipped_note.map(|note| {
                        view! { <p class="mt-1 text-sm text-text-soft">{note}</p> }
                    })}
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
                    <div class="mt-3">
                        <LinkButton
                            variant=ButtonVariant::Primary(ButtonSize::Md)
                            href=list_href
                        >
                            "一覧で確認する"
                        </LinkButton>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests;
