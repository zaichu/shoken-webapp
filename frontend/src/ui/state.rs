use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::labels;
use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertVariant {
    Warning,
    Danger,
}

impl AlertVariant {
    const fn class(self) -> &'static str {
        match self {
            Self::Warning => "border-accent-border bg-accent-soft text-accent-text",
            Self::Danger => "border-negative-border bg-negative-soft text-negative-vivid",
        }
    }
}

#[component]
pub fn Alert(variant: AlertVariant, children: Children) -> impl IntoView {
    let variant_class = variant.class();
    view! {
        <div
            class={format!(
                "rounded-lg border px-4 py-3 text-body font-normal shadow-sm {variant_class}"
            )}
            role="alert"
        >
            {children()}
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpinnerSize {
    Sm,
    Lg,
}

impl SpinnerSize {
    const fn class(self) -> &'static str {
        match self {
            Self::Sm => "h-4 w-4",
            Self::Lg => "h-8 w-8",
        }
    }
}

#[component]
pub fn Spinner(size: SpinnerSize, class: &'static str) -> impl IntoView {
    let size_class = size.class();
    view! {
        <span class={format!("inline-flex items-center {class}")}>
            <svg
                class={format!("animate-spin {size_class}")}
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                role="status"
                aria-label=labels::LOADING
            >
                <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                />
                <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                />
            </svg>
        </span>
    }
}

#[component]
pub fn Loading() -> impl IntoView {
    view! { <p role="status">{labels::LOADING}</p> }
}

/// 見本の失敗行に合わせ、文言(句点区切り)の先頭文を1行目、残りを補足(2行目・灰)に分ける。
/// 文言自体は変えず並べるだけにする。
fn split_first_sentence(message: &str) -> (&str, Option<&str>) {
    message.find('。').map_or((message, None), |idx| {
        let (first, rest) = message.split_at(idx + '。'.len_utf8());
        let rest = rest.trim();
        (first, (!rest.is_empty()).then_some(rest))
    })
}

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub fn ListLoadError(
    message: String,
    on_retry: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    // 見本の失敗行の型: 1行目に何が起きたか、2行目に補足(灰)、右に「再試行」の文字ボタン(青)。
    // 文言は labels.rs のまま、並べるだけにする。
    let (first, rest) = split_first_sentence(&message);
    let first = first.to_string();
    let rest = rest.map(str::to_string);
    view! {
        <div
            class="flex items-center justify-between gap-3 px-4 py-3"
            data-testid="list-load-error"
        >
            <div class="min-w-0 flex-1" role="alert">
                <p class="text-body font-bold text-text-deep">
                    <strong>{labels::ERROR_PREFIX}</strong>
                    " "
                    {first}
                </p>
                {rest.map(|rest| {
                    view! { <p class="mt-0.5 text-cell font-normal text-text-muted">{rest}</p> }
                })}
            </div>
            <Button variant=ButtonVariant::Retry on_click=move |_| on_retry()>
                {labels::RETRY}
            </Button>
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListSkeletonVariant {
    Cards,
    Table,
}

#[component]
pub fn ListSkeleton(variant: ListSkeletonVariant) -> impl IntoView {
    let body = match variant {
        ListSkeletonVariant::Cards => view! {
            {(0..3)
                .map(|_| {
                    view! {
                        <div class="grid gap-2 rounded-lg border border-border-subtle p-4">
                            <div class="h-4 w-1/3 rounded bg-fill"></div>
                            <div class="h-6 w-1/2 rounded bg-fill"></div>
                            <div class="h-4 w-2/3 rounded bg-fill"></div>
                        </div>
                    }
                })
                .collect_view()}
        }
        .into_any(),
        ListSkeletonVariant::Table => view! {
            {(0..2)
                .map(|_| {
                    view! {
                        <div class="grid gap-2">
                            <div class="h-4 w-32 rounded bg-fill"></div>
                            <div class="grid gap-2 rounded-lg border border-border-subtle p-4">
                                {(0..4)
                                    .map(|_| {
                                        view! {
                                            <div class="skeleton-table-row">
                                                <div class="h-4 rounded bg-fill"></div>
                                                <div class="h-4 rounded bg-fill max-sm:hidden"></div>
                                                <div class="h-4 rounded bg-fill"></div>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </div>
                    }
                })
                .collect_view()}
        }
        .into_any(),
    };
    view! {
        <Card variant=CardVariant::Panel class="p-4" attr:role="status" testid="list-skeleton">
            <span class="sr-only">{labels::LIST_LOADING_MESSAGE}</span>
            <div class="grid animate-pulse gap-4" aria-hidden="true">
                {body}
            </div>
        </Card>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_first_sentence_keeps_all_text() {
        let (first, rest) =
            split_first_sentence("読み込みに失敗しました。時間をおいて再度お試しください。");
        assert_eq!(first, "読み込みに失敗しました。");
        assert_eq!(rest, Some("時間をおいて再度お試しください。"));
    }

    #[test]
    fn split_first_sentence_without_period_has_no_note() {
        let (first, rest) = split_first_sentence("読み込みに失敗しました");
        assert_eq!(first, "読み込みに失敗しました");
        assert_eq!(rest, None);
    }
}
