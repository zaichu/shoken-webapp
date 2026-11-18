use crate::ui::button::{ButtonSize, ButtonVariant, LinkButton};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use crate::ui::labels;
use crate::ui::not_found as not_found_ui;
use crate::ui::page_frame::PageFrame;
use leptos::prelude::*;

const RELATED_LINKS: &[(&str, &str)] = &[
    ("/search", labels::STOCK_SEARCH_TITLE),
    ("/assetbalance", labels::ASSET_BALANCE_TITLE),
    ("/receipts", labels::RECEIPTS_TITLE),
];

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <PageFrame title=labels::NOT_FOUND_TITLE>
        <div class="min-h-[50vh] flex flex-1 items-center justify-center">
            <EmptyState
                title=labels::NOT_FOUND_HEADING
                icon=EmptyStateIcon::Warning
            >
                <not_found_ui::NotFoundActions>
                    <LinkButton variant=ButtonVariant::Primary(ButtonSize::Md) class="no-print" href="/">
                        {labels::NOT_FOUND_HOME_BACK}
                    </LinkButton>
                    <not_found_ui::NotFoundRelatedLinks>
                        {RELATED_LINKS
                            .iter()
                            .map(|(to, label)| {
                                view! {
                                    <a href={*to} class="security-code-link text-body">
                                        {*label}
                                    </a>
                                }
                            })
                            .collect_view()}
                    </not_found_ui::NotFoundRelatedLinks>
                </not_found_ui::NotFoundActions>
            </EmptyState>
        </div>
        </PageFrame>
    }
}
