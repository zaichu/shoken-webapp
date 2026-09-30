use leptos::prelude::*;

#[component]
pub fn MobileToolbar(
    aria_label: &'static str,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="mobile-toolbar" role="group" aria-label=aria_label data-testid=testid>
            {children()}
        </div>
    }
}
