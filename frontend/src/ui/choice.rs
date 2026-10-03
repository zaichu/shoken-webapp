use crate::support::list_search::SearchOption;
use crate::ui::disclosure::ChevronIcon;
use leptos::ev;
use leptos::prelude::*;

/// 選択の押せる小部品とフォームの選択部品。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipVariant {
    /// 検索オプションの選択チップ(aria-pressed)
    Filter,
    /// 期間の切り替えセグメント(aria-pressed)
    Segment,
    /// 枠付きの解除チップ
    Pill,
}

impl ChipVariant {
    fn selected_class(self) -> &'static str {
        match self {
            Self::Filter => {
                "rounded border border-ink bg-ink px-3 py-1.5 text-sm font-bold text-text-inverse"
            }
            Self::Segment => {
                "flex-1 rounded bg-ink px-2 py-1 text-xs font-semibold text-text-inverse"
            }
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

    fn disabled_class(self) -> &'static str {
        match self {
            Self::Segment => {
                "flex-1 cursor-not-allowed rounded bg-surface-raised px-2 py-1 text-xs font-semibold text-text-faint"
            }
            _ => self.unselected_class(),
        }
    }
}

/// aria-pressed の選択チップ。selected が無いもの(解除チップ)は aria-pressed を付けない
#[component]
pub fn Chip(
    variant: ChipVariant,
    #[prop(into, optional)] selected: Option<Signal<bool>>,
    #[prop(into, optional)] disabled: Option<Signal<bool>>,
    #[prop(into, optional)] aria_label: Option<Signal<String>>,
    #[prop(optional)] testid: Option<&'static str>,
    on_click: impl Fn(ev::MouseEvent) + 'static,
    children: Children,
) -> impl IntoView {
    let classes = move || {
        if disabled.is_some_and(|disabled| disabled.get()) {
            return variant.disabled_class().to_string();
        }
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
            disabled=move || disabled.is_some_and(|disabled| disabled.get())
            aria-pressed=selected.map(|selected| move || selected.get().to_string())
            aria-label=move || aria_label.map(|label| label.get())
            data-testid=testid
            on:click=on_click
        >
            {children()}
        </button>
    }
}

/// ラベル付きのネイティブ select(「全て表示」が先頭)。
/// OS 標準の矢印は消して山形アイコンを置き、値が入るとアクセント色になる
#[component]
pub fn Select(
    id: &'static str,
    label: &'static str,
    #[prop(into)] value: Signal<String>,
    options: Vec<SearchOption>,
    on_change: impl Fn(String) + 'static,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div>
            <label class="mb-1 block text-sm font-bold text-text" for=id>
                {label}
            </label>
            <div class="relative">
                <select
                    id=id
                    class=move || {
                        let base = if value.get().is_empty() {
                            "w-full appearance-none rounded-md border border-border-strong bg-surface py-2 pl-3 pr-9 text-sm max-sm:min-h-11"
                        } else {
                            "w-full appearance-none rounded-md border border-accent-bright bg-accent-soft py-2 pl-3 pr-9 text-sm font-semibold text-accent-text max-sm:min-h-11"
                        };
                        if let Some(c) = class {
                            format!("{} {}", base, c)
                        } else {
                            base.to_string()
                        }
                    }
                    prop:value=move || value.get()
                    on:change=move |event| on_change(event_target_value(&event))
                >
                    <option value="">"全て表示"</option>
                    {options
                        .into_iter()
                        .map(|option| view! { <option value=option.value>{option.label}</option> })
                        .collect_view()}
                </select>
                <ChevronIcon
                    expanded=Signal::derive(|| false)
                    class="pointer-events-none absolute right-3 top-1/2 h-4 w-4 -translate-y-1/2 text-text-subtle"
                />
            </div>
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

#[cfg(test)]
mod tests;
