use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertVariant {
    Warning,
    Danger,
}

impl AlertVariant {
    fn class(self) -> &'static str {
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
                "rounded-lg border px-4 py-3 text-sm font-medium shadow-sm {variant_class}"
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
    fn class(self) -> &'static str {
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
                aria-label="読み込み中..."
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
    view! { <p role="status">"読み込み中..."</p> }
}

#[component]
pub fn LoadingStrip(#[prop(into)] text: Signal<String>) -> impl IntoView {
    view! {
        <section class="px-5 py-4" role="status" aria-live="polite" aria-atomic="true">
            <div class="flex items-center gap-2 text-text-muted">
                <Spinner size=SpinnerSize::Sm class=""/>
                <p class="text-sm">{move || text.get()}</p>
            </div>
        </section>
    }
}

#[component]
pub fn Skeleton(#[prop(into, optional)] class: Signal<String>) -> impl IntoView {
    view! {
        <div
            class=move || format!("animate-pulse rounded bg-fill {}", class.get())
            aria-hidden="true"
        ></div>
    }
}

#[component]
pub fn ListLoadError(
    message: String,
    on_retry: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Panel class="p-4" testid="list-load-error">
            <div
                class="rounded-lg border border-negative-border bg-negative-soft px-4 py-3 text-sm font-medium text-negative-vivid"
                role="alert"
            >
                <strong>"エラー:"</strong>
                " "
                {message}
            </div>
            <div class="mt-4">
                <Button variant=ButtonVariant::Retry on_click=move |_| on_retry()>
                    "再読み込み"
                </Button>
            </div>
        </Card>
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
            <span class="sr-only">"データを読み込んでいます..."</span>
            <div class="grid animate-pulse gap-4" aria-hidden="true">
                {body}
            </div>
        </Card>
    }
}
