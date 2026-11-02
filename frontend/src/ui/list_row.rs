use crate::ui::button::{Button, ButtonVariant};
use leptos::prelude::*;

#[component]
pub fn ListRow(
    name: impl IntoView + 'static,
    primary: impl IntoView + 'static,
    meta: impl IntoView + 'static,
    secondary: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <div class="list-row">
            <div class="list-row-main">
                <span class="list-row-name">{name}</span>
                <span class="list-row-primary">{primary}</span>
            </div>
            <div class="list-row-sub">
                <span class="list-row-meta">{meta}</span>
                <span class="list-row-secondary">{secondary}</span>
            </div>
        </div>
    }
}

#[component]
pub fn ListRowButton(
    name: impl IntoView + 'static,
    primary: impl IntoView + 'static,
    meta: impl IntoView + 'static,
    secondary: impl IntoView + 'static,
    #[prop(optional)] testid: Option<&'static str>,
    on_click: impl Fn(leptos::ev::MouseEvent) + 'static,
) -> impl IntoView {
    view! {
        <Button
            variant=ButtonVariant::RowButton
            testid=testid.unwrap_or("list-row-button")
            aria_haspopup="dialog"
            on_click=on_click
        >
            <span class="list-row-action-body">
                <ListRow name=name primary=primary meta=meta secondary=secondary />
            </span>
        </Button>
    }
}

#[component]
pub fn ListRowLink(
    href: String,
    name: impl IntoView + 'static,
    primary: impl IntoView + 'static,
    meta: impl IntoView + 'static,
    secondary: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <a class="list-row-action" href=href>
            <ListRow name=name primary=primary meta=meta secondary=secondary />
        </a>
    }
}

#[component]
pub fn ListRowStatic(
    name: impl IntoView + 'static,
    primary: impl IntoView + 'static,
    meta: impl IntoView + 'static,
    secondary: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <div class="list-row-action-static">
            <ListRow name=name primary=primary meta=meta secondary=secondary />
        </div>
    }
}

#[component]
pub fn ListGroupHeader(label: String, summary: Option<String>) -> impl IntoView {
    view! {
        <div class="list-group-header">
            <span class="list-group-title">{label}</span>
            {summary.map(|text| view! { <span class="list-group-summary">{text}</span> })}
        </div>
    }
}
