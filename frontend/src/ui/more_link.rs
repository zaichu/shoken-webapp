use leptos::prelude::*;

#[component]
pub fn MoreLink(
    #[prop(into)] href: String,
    #[prop(into)] label: String,
    #[prop(into, optional)] testid: String,
) -> impl IntoView {
    let testid = if testid.is_empty() {
        None
    } else {
        Some(testid)
    };
    view! {
        <a class="more-link" href=href data-testid=testid>
            {label}
            <span aria-hidden="true">" ›"</span>
        </a>
    }
}
