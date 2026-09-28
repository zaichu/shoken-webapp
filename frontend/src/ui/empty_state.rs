use leptos::prelude::*;

/// データが空の画面の見せ方を1種類にまとめる。
/// icon は見出しの上、children は説明の下(次の行動)に出る。
#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(into)] description: String,
    #[prop(optional)] icon: Option<AnyView>,
    #[prop(optional)] as_h1: bool,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class=move || {
            let extra = class.get();
            if extra.is_empty() {
                "empty-state".to_string()
            } else {
                format!("empty-state {extra}")
            }
        }>
            {icon}
            {if as_h1 {
                view! { <h1 class="text-2xl font-black text-ink">{title.clone()}</h1> }.into_any()
            } else {
                view! { <h3 class="text-base font-black text-ink">{title.clone()}</h3> }.into_any()
            }}
            <p class="mt-1.5 max-w-md text-sm font-medium text-text-muted">{description}</p>
            {children.map(|children| children())}
        </div>
    }
}
