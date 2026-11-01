use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[cfg(test)]
mod tests;

pub fn next_tab_index(count: usize, current: usize, key: &str) -> Option<usize> {
    if count == 0 {
        return None;
    }
    match key {
        "ArrowRight" => Some((current + 1) % count),
        "ArrowLeft" => Some((current + count - 1) % count),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => None,
    }
}

pub fn focus_tab_by_id(id: &str) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
    else {
        return;
    };
    if let Some(element) = element.dyn_ref::<web_sys::HtmlElement>() {
        let _ = element.focus();
    }
}

#[component]
pub fn UnderlineTabButton(
    #[prop(into)] id: String,
    #[prop(optional, into)] controls: String,
    #[prop(into)] label: String,
    #[prop(into)] count: Signal<String>,
    #[prop(into)] selected: Signal<bool>,
    #[prop(optional, into)] count_testid: String,
    #[prop(optional, into)] testid: String,
    on_select: impl Fn() + Clone + 'static,
    on_keydown: impl Fn(web_sys::KeyboardEvent) + Clone + 'static,
) -> impl IntoView {
    let count_id = count_testid.clone();
    let button_id = testid.clone();
    let controls_attr = if controls.is_empty() {
        None
    } else {
        Some(controls)
    };
    view! {
        <button
            type="button"
            role="tab"
            id=id
            aria-controls=controls_attr
            aria-selected=move || selected.get().to_string()
            tabindex=move || if selected.get() { "0" } else { "-1" }
            class="page-tab"
            data-testid=button_id
            on:click=move |_| on_select.clone()()
            on:keydown=on_keydown.clone()
        >
            {label}
            <span class="page-tab-count" data-testid=count_id>
                {move || count.get()}
            </span>
        </button>
    }
}
