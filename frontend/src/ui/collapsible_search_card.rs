use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant, IconButton, IconButtonVariant};
use crate::ui::disclosure::{ChevronIcon, DisclosureStyle, DisclosureToggle};
use leptos::prelude::*;

const FUNNEL_ICON_PATH: &str = "M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z";

// matchMedia 未対応環境では展開側に倒す
pub fn is_narrow_viewport() -> bool {
    web_sys::window()
        .and_then(|window| window.match_media("(max-width: 639px)").ok().flatten())
        .is_some_and(|query| query.matches())
}

#[component]
pub fn CollapsibleSearchCard(
    #[prop(default = true)] initial_expanded: bool,
    #[prop(into)] has_active_search: Signal<bool>,
    #[prop(into)] is_default_state: Signal<bool>,
    on_clear: impl Fn() + Clone + 'static,
    #[prop(optional)] on_expand_toggle: Option<Callback<(bool,)>>,
    children: ChildrenFn,
) -> impl IntoView {
    // 展開状態は UI 表示のみの内部 state。initial_expanded は初期値としてのみ使い、
    // ユーザー操作後に親から上書きしない
    let expanded = RwSignal::new(initial_expanded);
    let toggle = move || {
        expanded.update(|open| {
            *open = !*open;
            if let Some(on_expand_toggle) = on_expand_toggle {
                on_expand_toggle.run((*open,));
            }
        });
    };

    view! {
        <section class="px-4 py-4" data-testid="search-card-compact">
            <div class="flex items-center justify-between gap-2">
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
                </DisclosureToggle>
                <div class="flex shrink-0 items-center gap-1.5">
                    <Button
                        variant=ButtonVariant::SecondarySoft(ButtonSize::Xs)
                        class=Signal::derive(move || {
                            format!(
                                "no-print whitespace-nowrap transition-opacity{}",
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
                    <IconButton
                        variant=IconButtonVariant::Boxed
                        on_click=move |_| toggle()
                        aria_hidden=true
                        tabindex="-1"
                        testid="search-card-chevron-toggle"
                    >
                        <ChevronIcon
                            expanded=Signal::derive(move || expanded.get())
                            class="h-4 w-4 text-text-subtle transition-transform duration-200"
                        />
                    </IconButton>
                </div>
            </div>
            {move || {
                expanded
                    .get()
                    .then(|| {
                        view! {
                            <div id="search-options-body" class="pt-4">
                                {children()}
                            </div>
                        }
                    })
            }}
        </section>
    }
}
