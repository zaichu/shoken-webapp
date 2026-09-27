use leptos::ev;
use leptos::prelude::*;

/// tablist の中の role="tab" ボタン。選択・roving tabindex・キー移動は
/// 呼び出し側が持ち、この部品は見た目と属性を固定する。
const TAB_BUTTON_BASE: &str = "inline-flex items-center gap-2 rounded-lg border px-4 py-2.5 text-sm font-bold max-sm:min-h-11 max-sm:shrink-0 max-sm:px-3";
const TAB_BUTTON_ACTIVE: &str = "border-ink bg-ink text-text-inverse shadow-edge-accent";
const TAB_BUTTON_INACTIVE: &str =
    "border-border-strong bg-surface text-text hover:border-border-xstrong hover:bg-surface hover:text-ink";

#[component]
pub fn TabButton(
    #[prop(into)] id: String,
    #[prop(into)] controls: String,
    #[prop(into)] selected: Signal<bool>,
    on_select: impl Fn() + 'static,
    on_keydown: impl Fn(ev::KeyboardEvent) + 'static,
    children: ChildrenFn,
) -> impl IntoView {
    view! {
        <button
            id=id
            type="button"
            role="tab"
            class=move || {
                format!(
                    "{TAB_BUTTON_BASE} {}",
                    if selected.get() {
                        TAB_BUTTON_ACTIVE
                    } else {
                        TAB_BUTTON_INACTIVE
                    },
                )
            }
            aria-selected=move || selected.get().to_string()
            aria-controls=controls
            tabindex=move || if selected.get() { "0" } else { "-1" }
            on:click=move |_| on_select()
            on:keydown=move |event| on_keydown(event)
        >
            {children()}
        </button>
    }
}
