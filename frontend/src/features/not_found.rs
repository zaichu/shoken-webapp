use crate::ui::button::{ButtonVariant, LinkButton};
use crate::ui::empty_state::EmptyState;
use leptos::prelude::*;

const RELATED_LINKS: &[(&str, &str)] = &[
    ("/", "ホーム"),
    ("/search", "銘柄検索"),
    ("/receipts", "取引明細"),
];

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="min-h-[50vh] flex items-center justify-center">
            <EmptyState
                as_h1=true
                title="404 - ページが見つかりません"
                description="お探しのページは存在しないか、移動した可能性があります。"
                icon=view! {
                    <div class="mb-3 text-text-faint" aria-hidden="true">
                        <svg
                            class="h-16 w-16"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke="currentColor"
                            aria-hidden="true"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="1.5"
                                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                            />
                        </svg>
                    </div>
                }
                .into_any()
            >
                <div class="mt-4">
                    <div class="flex flex-col items-center gap-4">
                        <LinkButton variant=ButtonVariant::Primary class="no-print" href="/">
                            "ホームに戻る"
                        </LinkButton>
                        <div class="flex flex-wrap justify-center gap-3">
                            {RELATED_LINKS
                                .iter()
                                .map(|(to, label)| {
                                    view! {
                                        <a href={*to} class="text-base text-text-deep hover:underline">
                                            {*label}
                                        </a>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                </div>
            </EmptyState>
        </div>
    }
}
