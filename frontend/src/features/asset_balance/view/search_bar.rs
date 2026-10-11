use crate::ui::button::IconButton;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub(crate) fn AssetBalanceSearchBar(
    query: RwSignal<String>,
    #[prop(into)] filtered_count: Signal<usize>,
    total_count: usize,
) -> impl IntoView {
    view! {
        <div class="mb-3 no-print" role="search" data-testid="asset-search-bar">
            <div class="search-input-frame">
                <input
                    type="search"
                    id="asset-search-input"
                    data-testid="asset-search-input"
                    class="asset-search-input"
                    placeholder="銘柄名・コードで絞り込み"
                    aria-label="銘柄名・コードで絞り込み"
                    autocomplete="off"
                    enterkeyhint="search"
                    prop:value=move || query.get()
                    // IME 変換中の確定前の文字では絞り込まない。確定は compositionend で受ける
                    on:input=move |ev| {
                        if ev.unchecked_ref::<web_sys::InputEvent>().is_composing() {
                            return;
                        }
                        query.set(event_target_value(&ev));
                    }
                    on:compositionend=move |ev| query.set(event_target_value(&ev))
                />
                {move || {
                    (!query.get().is_empty()).then(|| {
                        view! {
                            <IconButton
                                aria_label="検索を解除".to_string()
                                class="asset-search-clear".to_string()
                                testid="asset-search-clear"
                                on_click=move |_| query.set(String::new())
                            >
                                <span aria-hidden="true">"×"</span>
                            </IconButton>
                        }
                    })
                }}
            </div>
            {move || {
                (!query.get().is_empty()).then(|| {
                    view! {
                        <p class="asset-search-count" data-testid="asset-search-count">
                            {format!(
                                "検索結果 {} / {}件",
                                filtered_count.get(),
                                total_count,
                            )}
                        </p>
                    }
                })
            }}
        </div>
    }
}
