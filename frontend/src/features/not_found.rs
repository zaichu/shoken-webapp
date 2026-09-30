use crate::ui::button::{ButtonSize, ButtonVariant, LinkButton};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
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
                icon=EmptyStateIcon::Warning
            >
                <div class="mt-4">
                    <div class="flex flex-col items-center gap-4">
                        <LinkButton variant=ButtonVariant::Primary(ButtonSize::Md) class="no-print" href="/">
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
