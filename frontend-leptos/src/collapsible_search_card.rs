use leptos::prelude::*;

const BUTTON_BASE: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print";

const FUNNEL_ICON_PATH: &str = "M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z";
const CHEVRON_ICON_PATH: &str = "M19 9l-7 7-7-7";

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
    children: ChildrenFn,
) -> impl IntoView {
    // 展開状態は UI 表示のみの内部 state。initial_expanded は初期値としてのみ使い、
    // ユーザー操作後に親から上書きしない
    let expanded = RwSignal::new(initial_expanded);
    let toggle = move || {
        expanded.update(|open| {
            *open = !*open;
        });
    };

    view! {
        <section class="px-4 py-4" data-testid="search-card-compact">
            <div class="flex items-center justify-between gap-2">
                <button
                    type="button"
                    class="flex min-w-0 items-center gap-2.5 text-left select-none cursor-pointer max-sm:min-h-[44px]"
                    on:click=move |_| toggle()
                    aria-expanded=move || if expanded.get() { "true" } else { "false" }
                    aria-controls="search-options-body"
                    aria-label=move || {
                        if expanded.get() { "検索オプション 閉じる" } else { "検索オプション 開く" }
                    }
                    data-testid="search-card-header"
                >
                    <span class="inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-md bg-slate-950 text-white">
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
                        <p class="text-[10px] font-black uppercase tracking-[0.18em] text-slate-500">
                            "Filter"
                        </p>
                        <h5 class="whitespace-nowrap text-sm font-black text-slate-950">
                            "検索オプション"
                        </h5>
                        {move || {
                            (!expanded.get() && has_active_search.get()).then(|| {
                                view! {
                                    <span class="mt-1 inline-flex whitespace-nowrap rounded-md border border-amber-200 bg-amber-50 px-2 py-0.5 text-[11px] font-bold text-amber-800">
                                        "適用中"
                                    </span>
                                }
                            })
                        }}
                    </div>
                </button>
                <div class="flex shrink-0 items-center gap-1.5">
                    <button
                        type="button"
                        class=move || {
                            format!(
                                "{BUTTON_BASE} border border-slate-300 text-slate-700 hover:border-slate-500 hover:bg-slate-50 whitespace-nowrap bg-white px-2 py-1 text-xs transition-opacity max-sm:min-h-[44px]{}",
                                if is_default_state.get() {
                                    " opacity-0 pointer-events-none"
                                } else {
                                    ""
                                },
                            )
                        }
                        aria-label="検索条件をクリア"
                        aria-hidden=move || if is_default_state.get() { "true" } else { "false" }
                        tabindex=move || if is_default_state.get() { "-1" } else { "0" }
                        data-testid="search-clear-button"
                        on:click={
                            let on_clear = on_clear.clone();
                            move |event| {
                                event.stop_propagation();
                                on_clear()
                            }
                        }
                    >
                        "解除"
                    </button>
                    <button
                        type="button"
                        class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-md border border-slate-300 bg-white text-slate-700"
                        on:click=move |_| toggle()
                        aria-expanded=move || if expanded.get() { "true" } else { "false" }
                        aria-controls="search-options-body"
                        aria-label=move || {
                            if expanded.get() { "検索オプション 閉じる" } else { "検索オプション 開く" }
                        }
                        data-testid="search-card-chevron-toggle"
                    >
                        <svg
                            class=move || {
                                format!(
                                    "h-4 w-4 text-slate-500 transition-transform duration-200{}",
                                    if expanded.get() { " rotate-180" } else { "" },
                                )
                            }
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            aria-hidden="true"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d=CHEVRON_ICON_PATH
                            />
                        </svg>
                    </button>
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
