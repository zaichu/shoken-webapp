use crate::api::dto::CsvRowError;
use crate::support::csv_flow::row_error_text;
use leptos::prelude::*;

pub fn preview_notice_text(valid_rows: usize, save_action: &str) -> String {
    format!("{valid_rows}件 {save_action}されます")
}

// 未認証・ファイル未選択・解析中は通知を出さない(画面に残った古いプレビューを誤表示しない)
pub(crate) fn should_show_preview_notice(
    authenticated: bool,
    has_file: bool,
    previewing: bool,
) -> bool {
    authenticated && has_file && !previewing
}

// 未保存データで描画中であることを、行の見た目が登録済みと同じでも分かるように帯で示す。
// 読み上げはレールのプレビュー通知(role="status")が担うため、帯自体は見た目だけにする
#[component]
pub fn CsvPreviewBanner(description: &'static str) -> impl IntoView {
    view! {
        <div
            data-testid="csv-preview-banner"
            class="mb-3 rounded-lg border border-accent-border-strong bg-accent-soft px-4 py-2.5 text-sm text-accent-text"
        >
            <span class="font-bold">"プレビュー中(未保存)"</span>
            <span class="ml-2">{description}</span>
        </div>
    }
}

#[component]
pub fn CsvPreviewNotice(text: String, errors: Vec<CsvRowError>) -> impl IntoView {
    let error_count = errors.len();
    view! {
        <section
            class="rounded-lg border border-accent-border bg-accent-soft px-4 py-3 text-sm font-medium text-accent-text shadow-sm"
            role="status"
            aria-live="polite"
            data-testid="csv-preview-notice"
        >
            <div class="flex items-center gap-2">
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
                        d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                </svg>
                <p>
                    <strong>{text}</strong>
                    {(error_count > 0).then(|| format!(" / {error_count}件エラー"))}
                </p>
            </div>
            {(!errors.is_empty())
                .then(|| {
                    view! {
                        <ul class="mt-2 space-y-1">
                            {errors
                                .iter()
                                .map(|error| view! { <li>{row_error_text(error)}</li> })
                                .collect_view()}
                        </ul>
                    }
                })}
        </section>
    }
}

#[cfg(test)]
mod tests;
