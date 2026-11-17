use crate::session;
use crate::ui::more_link::MoreLink;
use crate::ui::state::{ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

#[component]
pub(super) fn CardHead(
    title: &'static str,
    href: &'static str,
    link_label: &'static str,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 px-4 pt-3.5">
            <h2 class="text-heading font-bold text-text">{title}</h2>
            <MoreLink href=href.to_string() label=link_label.to_string() />
        </div>
    }
}

#[component]
pub(super) fn CardSkeleton() -> impl IntoView {
    view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
}

#[component]
pub(super) fn CardError(#[prop(into)] message: String) -> impl IntoView {
    let session = session::use_session();
    view! {
        <div class="px-4 py-3">
            <ListLoadError
                message=message
                on_retry=move || session.generation.update(|generation| *generation = generation.next())
            />
        </div>
    }
}
