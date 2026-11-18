use leptos::prelude::*;

#[derive(Clone, Copy, Default)]
pub enum SelectVariant {
    #[default]
    Unstyled,
    Setting,
}

/// ドロップダウン選択。features/ では <select> を直書きせずこの部品を使う。
/// 選択肢は children に <option> を並べる
#[component]
pub fn Select(
    #[prop(optional)] variant: SelectVariant,
    /// 現在の値(一致する option が選択状態になる)
    #[prop(into)]
    value: Signal<String>,
    /// 変更時に新しい値を渡す
    on_change: impl Fn(String) + 'static,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(into, optional)] aria_label: Option<Signal<String>>,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <select
            class=move || match variant {
                SelectVariant::Unstyled => class.get(),
                SelectVariant::Setting => "min-w-44 rounded-md border border-border-strong bg-surface px-3 py-2 text-label max-sm:min-h-11".to_string(),
            }
            aria-label=move || aria_label.map(|label| label.get())
            data-testid=testid
            prop:value=move || value.get()
            on:change=move |ev| on_change(event_target_value(&ev))
        >
            {children()}
        </select>
    }
}
