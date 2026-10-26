use crate::ui::button::{ButtonSize, ButtonVariant, LinkButton};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use crate::ui::page_frame::PageFrame;
use leptos::prelude::*;

const RELATED_LINKS: &[(&str, &str)] = &[
    ("/search", "銘柄検索"),
    ("/assetbalance", "資産管理"),
    ("/receipts", "取引明細"),
];

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <PageFrame title="ページが見つかりません">
        <div class="min-h-[50vh] flex flex-1 items-center justify-center">
            <EmptyState
                title="404 - ページが見つかりません"
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
                                        <a href={*to} class="security-code-link text-base">
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
        </PageFrame>
    }
}
