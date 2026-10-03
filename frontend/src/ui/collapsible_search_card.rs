use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::disclosure::{ChevronIcon, DisclosureStyle, DisclosureToggle};
use leptos::prelude::*;

const FUNNEL_ICON_PATH: &str = "M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z";

// matchMedia 未対応環境では展開側に倒す
pub fn is_narrow_viewport() -> bool {
    web_sys::window()
        .and_then(|window| window.match_media("(max-width: 639px)").ok().flatten())
        .is_some_and(|query| query.matches())
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum SearchCardLayout {
    #[default]
    Card,
    Toolbar,
}

#[component]
pub fn CollapsibleSearchCard(
    #[prop(optional)] expanded: Option<RwSignal<bool>>,
    #[prop(optional)] layout: SearchCardLayout,
    #[prop(into)] has_active_search: Signal<bool>,
    #[prop(into)] is_default_state: Signal<bool>,
    on_clear: impl Fn() + Clone + 'static,
    #[prop(optional)] on_expanded_change: Option<Callback<(bool,)>>,
    children: ChildrenFn,
) -> impl IntoView {
    let expanded = expanded.unwrap_or_else(|| RwSignal::new(true));
    // 初期値の通知も兼ねるためマウント時にも発火する
    Effect::new(move |_| {
        let open = expanded.get();
        if let Some(on_expanded_change) = on_expanded_change {
            on_expanded_change.run((open,));
        }
    });
    let toggle = move || expanded.update(|open| *open = !*open);
    let section_class = match layout {
        SearchCardLayout::Card => "px-4 py-4",
        SearchCardLayout::Toolbar => "toolbar-search-card",
    };
    let header_class = match layout {
        // 解除ボタンは absolute でトリガー端に重ねるため relative が必須
        SearchCardLayout::Card => "relative flex items-center justify-between gap-2",
        SearchCardLayout::Toolbar => "relative hidden sm:flex items-center justify-between gap-2",
    };

    view! {
        <section class=section_class data-expanded=move || expanded.get().to_string() data-testid="search-card-compact">
            <div class=header_class>
                <DisclosureToggle
                    style=DisclosureStyle::SearchCard
                    expanded=Signal::derive(move || expanded.get())
                    controls="search-options-body".to_string()
                    aria_label=Signal::derive(move || {
                        if expanded.get() {
                            "検索オプション 閉じる".to_string()
                        } else if has_active_search.get() {
                            "検索オプション 開く（絞り込み適用中）".to_string()
                        } else {
                            "検索オプション 開く".to_string()
                        }
                    })
                    testid="search-card-header"
                    on_toggle=move || toggle()
                >
                    <span class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-ink text-text-inverse">
                        <svg
                            class="h-4 w-4"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            aria-hidden="true"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d=FUNNEL_ICON_PATH
                            />
                        </svg>
                    </span>
                    <div class="min-w-0">
                        <h5 class="whitespace-nowrap text-sm font-black text-ink">
                            "検索オプション"
                        </h5>
                        {move || {
                            (!expanded.get() && has_active_search.get()).then(|| {
                                view! {
                                    <Badge variant=BadgeVariant::Accent>
                                        "適用中"
                                    </Badge>
                                }
                            })
                        }}
                    </div>
                    <ChevronIcon
                        expanded=Signal::derive(move || expanded.get())
                        class="ml-auto h-4 w-4 shrink-0 self-center text-text-subtle transition-transform duration-200"
                    />
                </DisclosureToggle>
                // 行全体をトリガーにするため、解除だけトリガーの手前に重ねて置く
                <Button
                    variant=ButtonVariant::SecondarySoft(ButtonSize::Xs)
                    class=Signal::derive(move || {
                        format!(
                            "absolute right-9 top-1/2 -translate-y-1/2 no-print whitespace-nowrap transition-opacity{}",
                            if is_default_state.get() {
                                " opacity-0 pointer-events-none"
                            } else {
                                ""
                            },
                        )
                    })
                    aria_label="検索条件をクリア".to_string()
                    aria_hidden=move || is_default_state.get()
                    tabindex=move || {
                        if is_default_state.get() { "-1" } else { "0" }
                    }
                    testid="search-clear-button"
                    on_click={
                        let on_clear = on_clear.clone();
                        move |event| {
                            event.stop_propagation();
                            on_clear()
                        }
                    }
                >
                    "解除"
                </Button>
            </div>
            <div id="search-options-body" hidden=move || !expanded.get() class="pt-4">
                {move || expanded.get().then(|| children())}
                {(layout == SearchCardLayout::Toolbar).then(|| view! {
                    <div class="mt-3 sm:hidden">
                        <Button
                            variant=ButtonVariant::SecondarySoft(ButtonSize::Xs)
                            aria_label="検索条件をクリア"
                            testid="receipt-search-clear-button"
                            disabled=move || is_default_state.get()
                            on_click=move |_| on_clear()
                        >
                            "解除"
                        </Button>
                    </div>
                })}
            </div>
        </section>
    }
}
