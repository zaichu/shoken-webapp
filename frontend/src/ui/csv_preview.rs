use crate::api::dto::CsvRowError;
use crate::support::csv_flow::{display_message, row_error_text};
use crate::ui::labels::count;
use leptos::prelude::*;

// 同一メッセージのエラーが4件以上並ぶときは1文に集約する(件数が少ない・理由が混在する一覧はそのままの方が読みやすい)
const AGGREGATE_MIN_ERRORS: usize = 4;
const EXAMPLE_ERROR_ROWS: usize = 3;

pub fn preview_notice_text(
    valid_rows: usize,
    error_count: usize,
    save_action: &str,
    errors_block_save: bool,
) -> String {
    let errors = if error_count > 0 {
        format!("({}エラー)", count(error_count))
    } else {
        String::new()
    };
    if valid_rows == 0 {
        format!("取り込める行がありません{errors}")
    } else if errors_block_save && error_count > 0 {
        format!(
            "{}を読み込めますが、{}のエラーを修正するまで{save_action}できません",
            count(valid_rows),
            count(error_count)
        )
    } else {
        format!("{}を{save_action}できます{errors}", count(valid_rows))
    }
}

// 保存できないプレビュー(有効行0・または全件置換でエラー残り)は警告色にする
// 未認証・ファイル未選択・解析中は通知を出さない(画面に残った古いプレビューを誤表示しない)
pub(crate) fn should_show_preview_notice(
    authenticated: bool,
    has_file: bool,
    previewing: bool,
) -> bool {
    authenticated && has_file && !previewing
}

// 全行が同じ理由で落ちたとき、その理由を返す(表示用に空の括弧は外す)
pub(crate) fn shared_message(errors: &[CsvRowError]) -> Option<String> {
    let first = errors.first()?;
    (errors.len() >= AGGREGATE_MIN_ERRORS
        && errors.iter().all(|error| error.message == first.message))
    .then(|| display_message(&first.message).to_string())
}

/// 行エラーの一覧。全行が同じ理由なら1回だけ表示して全行分は折りたたみに入れる
/// (見出し不一致のような典型的な間違いでは全行が同じ文になるため)
/// all_failed が false(一部行が有効・スキップ)なら見出し不一致の案内は出さない。
/// 見出しが違えば全行落ちるため、有効行があるのに案内を出すと誤誘導になる
#[component]
pub fn CsvRowErrorList(errors: Vec<CsvRowError>, all_failed: bool) -> impl IntoView {
    if let Some(message) = shared_message(&errors) {
        let count = errors.len();
        let examples: Vec<_> = errors.iter().take(EXAMPLE_ERROR_ROWS).cloned().collect();
        view! {
            <div class="mt-2">
                <p>
                    <strong>{message}</strong>
                    {format!(" — {count}行が同じエラーです")}
                </p>
                {all_failed.then(|| {
                    view! { <p class="mt-1">"見出しの行が想定と違う可能性があります"</p> }
                })}
                <ul class="mt-2 space-y-1">
                    {examples
                        .iter()
                        .map(|error| view! { <li>{format!("{}行目", error.row)}</li> })
                        .collect_view()}
                </ul>
                <details class="mt-2">
                    <summary class="cursor-pointer">"すべての行を表示"</summary>
                    <ul class="mt-2 space-y-1">
                        {errors
                            .iter()
                            .map(|error| view! { <li>{format!("{}行目", error.row)}</li> })
                            .collect_view()}
                    </ul>
                </details>
            </div>
        }
        .into_any()
    } else {
        view! {
            <ul class="mt-2 space-y-1">
                {errors
                    .iter()
                    .map(|error| view! { <li>{row_error_text(error)}</li> })
                    .collect_view()}
            </ul>
        }
        .into_any()
    }
}

/// 行エラーの帯。保存できる件数の文はファイル欄に出るため、ここには出さない。
/// 「N件エラー ・ N行目: 理由 …」を1つの黄の帯にまとめ、折り返さずに横へ流す(全文は title に)
#[component]
pub fn CsvPreviewNotice(valid_rows: usize, errors: Vec<CsvRowError>) -> impl IntoView {
    let heading = format!("{}エラー", count(errors.len()));
    let full = format!(
        "{} ・ {}",
        heading,
        errors
            .iter()
            .map(row_error_text)
            .collect::<Vec<_>>()
            .join(" ・ ")
    );
    view! {
        <section
            class="overflow-x-auto rounded-lg border border-band-border bg-band px-4 py-3 text-sm font-medium text-warn-ink shadow-sm"
            role="status"
            aria-live="polite"
            data-testid="csv-preview-notice"
            title=full
        >
            <div class="flex items-center gap-2 whitespace-nowrap">
                <svg
                    class="h-5 w-5 shrink-0"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    aria-hidden="true"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="2"
                        d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0zM12 9v4m0 4h.01"
                    />
                </svg>
                <p>
                    <strong>{heading}</strong>
                </p>
            </div>
            <div class="mt-1 whitespace-nowrap">
                <CsvRowErrorList errors=errors all_failed=valid_rows == 0 />
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests;
