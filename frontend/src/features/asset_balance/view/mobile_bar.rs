use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::select::Select;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

// 検索語はページ側の状態として画面遷移や他の入力欄をまたいで残るので、
// 絞り込み中は必ず検索欄を開く(閉じたまま一覧だけが減る状態を作らない)
pub(crate) fn mobile_search_open(requested: bool, query: &str) -> bool {
    requested || !query.is_empty()
}

/// スマホ用の下部操作帯。絞り込みは押すと検索欄をその場で開き、
/// 並べ替えは項目の選択と昇順/降順の切り替えを同じ段に置く
#[component]
pub fn AssetMobileActionBar(
    query: RwSignal<String>,
    count_text: Signal<String>,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let search_requested = RwSignal::new(false);
    let search_open =
        Signal::derive(move || mobile_search_open(search_requested.get(), &query.get()));
    let input_ref = NodeRef::<html::Input>::new();
    // フォーカスは利用者が開いたときだけ。検索語による自動表示では奪わない
    Effect::new(move |_| {
        if search_requested.get()
            && let Some(input) = input_ref.get()
        {
            let _ = input.focus();
        }
    });

    view! {
        <div class="asset-mobile-bar">
            {move || {
                search_open.get().then(|| {
                    view! {
                        <div class="asset-mobile-search" id="asset-mobile-search">
                            <div class="search-input-frame">
                                <svg
                                    class="mx-2 h-4 w-4 shrink-0 self-center text-text-muted"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    aria-hidden="true"
                                >
                                    <circle cx="11" cy="11" r="7" />
                                    <path d="m21 21-4.35-4.35" />
                                </svg>
                                <input
                                    type="search"
                                    class="asset-search-input"
                                    placeholder="銘柄名・コードで絞り込み"
                                    aria-label="銘柄名・コードで絞り込み"
                                    autocomplete="off"
                                    enterkeyhint="search"
                                    data-testid="asset-search-input"
                                    node_ref=input_ref
                                    prop:value=move || query.get()
                                    // IME 変換中の確定前の文字では絞り込まない。確定は compositionend で受ける
                                    on:input=move |ev| {
                                        if ev.unchecked_ref::<web_sys::InputEvent>().is_composing() {
                                            return;
                                        }
                                        query.set(event_target_value(&ev));
                                    }
                                    on:compositionend=move |ev| {
                                        query.set(event_target_value(&ev));
                                    }
                                />
                                {move || {
                                    (!query.get().is_empty()).then(|| {
                                        view! {
                                            <Button
                                                variant=ButtonVariant::SecondarySoft(ButtonSize::Xs)
                                                class="shrink-0 self-center".to_string()
                                                aria_label="検索を解除".to_string()
                                                testid="asset-search-clear"
                                                on_click=move |_| {
                                                    // 消しても欄は開いたままにし、続けて次の語を入れられるようにする
                                                    search_requested.set(true);
                                                    query.set(String::new())
                                                }
                                            >
                                                "消去"
                                            </Button>
                                        }
                                    })
                                }}
                            </div>
                            {move || {
                                let text = count_text.get();
                                (!text.is_empty()).then(|| {
                                    view! {
                                        <p
                                            class="asset-search-count"
                                            data-testid="asset-search-count"
                                        >
                                            {text}
                                        </p>
                                    }
                                })
                            }}
                        </div>
                    }
                })
            }}
            <div class="asset-mobile-bar-row" role="group" aria-label="絞り込みと並べ替え">
                <Button
                    variant=ButtonVariant::AssetMobileTool
                    aria_expanded=Signal::derive(move || search_open.get())
                    aria_controls="asset-mobile-search"
                    testid="asset-filter-toggle"
                    on_click=move |_| {
                        if search_open.get() {
                            // 閉じる=絞り込み解除。検索語を残すと絞り込み中なのに
                            // 検索欄も件数も見えない状態になるため消す
                            search_requested.set(false);
                            query.set(String::new());
                        } else {
                            search_requested.set(true);
                        }
                    }
                >
                    <svg
                        class="h-4 w-4"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        aria-hidden="true"
                    >
                        <circle cx="11" cy="11" r="7" />
                        <path d="m21 21-4.35-4.35" />
                    </svg>
                    "絞り込み"
                </Button>
                <label class="asset-mobile-sort">
                    <span class="shrink-0">"並べ替え:"</span>
                    <Select
                        class="asset-mobile-sort-select".to_string()
                        aria_label="並べ替える項目".to_string()
                        testid="asset-sort-select"
                        value=Signal::derive(move || sort_key.get().value().to_string())
                        on_change=move |value| {
                            if let Some(key) = SortKey::from_value(&value) {
                                sort_key.set(key);
                            }
                        }
                    >
                        {SortKey::ALL
                            .iter()
                            .map(|key| {
                                view! { <option value=key.value()>{key.label()}</option> }
                            })
                            .collect_view()}
                    </Select>
                    <span aria-hidden="true">"▾"</span>
                </label>
                <Button
                    variant=ButtonVariant::AssetMobileOrder
                    testid="asset-sort-order-toggle"
                    on_click=move |_| {
                        sort_order.update(|order| {
                            *order = match *order {
                                SortOrder::Desc => SortOrder::Asc,
                                SortOrder::Asc => SortOrder::Desc,
                            }
                        });
                    }
                >
                    {move || sort_order.get().label()}
                </Button>
            </div>
        </div>
    }
}
