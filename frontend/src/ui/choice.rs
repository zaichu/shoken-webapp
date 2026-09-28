use crate::support::list_search::SearchOption;
use leptos::ev;
use leptos::prelude::*;

/// 選択の押せる小部品とフォームの選択部品。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipVariant {
    /// 検索オプションの選択チップ(aria-pressed)
    Filter,
    /// 期間の切り替えセグメント(aria-pressed)
    Segment,
    /// 枠付きの解除チップ(.filter-chip)
    Pill,
}

impl ChipVariant {
    fn selected_class(self) -> &'static str {
        match self {
            Self::Filter => {
                "rounded border border-accent-bright bg-accent-soft px-3 py-1.5 text-sm font-bold text-accent-text"
            }
            Self::Segment => "flex-1 rounded bg-ink px-2 py-1 text-xs font-semibold text-text-inverse",
            Self::Pill => "filter-chip",
        }
    }

    fn unselected_class(self) -> &'static str {
        match self {
            Self::Filter => {
                "rounded border border-border-strong bg-surface px-3 py-1.5 text-sm text-text-soft"
            }
            Self::Segment => {
                "flex-1 rounded bg-surface-raised px-2 py-1 text-xs font-semibold text-text-muted"
            }
            Self::Pill => "filter-chip",
        }
    }
}

/// aria-pressed の選択チップ。selected が無いもの(解除チップ)は aria-pressed を付けない
#[component]
pub fn Chip(
    variant: ChipVariant,
    #[prop(into, optional)] selected: Option<Signal<bool>>,
    #[prop(into, optional)] aria_label: Option<Signal<String>>,
    #[prop(optional)] testid: Option<&'static str>,
    on_click: impl Fn(ev::MouseEvent) + 'static,
    children: Children,
) -> impl IntoView {
    let classes = move || {
        selected.map_or_else(
            || variant.selected_class().to_string(),
            |selected| {
                if selected.get() {
                    variant.selected_class().to_string()
                } else {
                    variant.unselected_class().to_string()
                }
            },
        )
    };
    view! {
        <button
            type="button"
            class=classes
            aria-pressed=selected.map(|selected| move || selected.get().to_string())
            aria-label=move || aria_label.map(|label| label.get())
            data-testid=testid
            on:click=on_click
        >
            {children()}
        </button>
    }
}

/// ラベル付きのネイティブ select(「全て表示」が先頭)
#[component]
pub fn Select(
    id: &'static str,
    label: &'static str,
    #[prop(into)] value: Signal<String>,
    options: Vec<SearchOption>,
    on_change: impl Fn(String) + 'static,
) -> impl IntoView {
    view! {
        <div>
            <label class="mb-1 block text-sm font-bold text-text" for=id>
                {label}
            </label>
            <select
                id=id
                class="w-full rounded-md border border-border-strong bg-surface px-3 py-2 text-sm max-sm:min-h-11"
                prop:value=move || value.get()
                on:change=move |event| on_change(event_target_value(&event))
            >
                <option value="">"全て表示"</option>
                {options
                    .into_iter()
                    .map(|option| view! { <option value=option.value>{option.label}</option> })
                    .collect_view()}
            </select>
        </div>
    }
}

/// 値が入るとアクセント色になる入力風トリガー(日付・年のピッカー)
#[component]
pub fn FieldTrigger(
    #[prop(into)] active: Signal<bool>,
    #[prop(into, optional)] aria_label: Option<Signal<String>>,
    #[prop(optional)] aria_haspopup: Option<&'static str>,
    #[prop(into, optional)] aria_expanded: Option<Signal<bool>>,
    #[prop(optional)] node_ref: Option<NodeRef<leptos::html::Button>>,
    on_click: impl Fn(ev::MouseEvent) + 'static,
    #[prop(optional)] on_keydown: Option<Box<dyn Fn(ev::KeyboardEvent)>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            node_ref=node_ref.unwrap_or_default()
            type="button"
            aria-label=move || aria_label.map(|label| label.get())
            aria-haspopup=aria_haspopup
            aria-expanded=move || aria_expanded.map(|expanded| expanded.get().to_string())
            class=move || {
                if active.get() {
                    "w-full rounded-md border border-accent-bright bg-accent-soft px-3 py-2 text-left text-sm font-semibold text-accent-text transition-colors"
                } else {
                    "w-full rounded-md border border-border-strong bg-surface px-3 py-2 text-left text-sm text-text-subtle transition-colors hover:border-border-xstrong"
                }
            }
            on:click=on_click
            on:keydown=move |event| {
                if let Some(on_keydown) = on_keydown.as_ref() {
                    on_keydown(event)
                }
            }
        >
            {children()}
        </button>
    }
}

/// listbox の選択肢(role="option")
#[component]
pub fn OptionButton(
    id: String,
    #[prop(into)] selected: Signal<bool>,
    on_click: impl Fn(ev::MouseEvent) + 'static,
    on_keydown: impl Fn(ev::KeyboardEvent) + 'static,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            id=id
            type="button"
            role="option"
            aria-selected=move || selected.get().to_string()
            class=move || {
                if selected.get() {
                    "rounded bg-accent-soft px-1 py-1.5 text-center text-sm font-semibold text-accent-text ring-1 ring-inset ring-accent-ring"
                } else {
                    "rounded px-1 py-1.5 text-center text-sm text-text-muted hover:bg-surface-raised hover:text-text-strong"
                }
            }
            on:click=on_click
            on:keydown=on_keydown
        >
            {children()}
        </button>
    }
}
