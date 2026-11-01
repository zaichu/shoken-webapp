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

#[cfg(test)]
mod tests;
