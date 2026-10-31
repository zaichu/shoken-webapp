//! 「タグ入力 + 条件を足す」式の検索欄に共通する部品。
//! 取引明細と資産管理はこの部品を共有し、条件の中身(期間の入力や
//! 口座・分配金コースの選択肢)だけを各ページが渡す

use crate::support::list_search::SearchOption;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::focus::trap_focus;
use crate::ui::labels;
use crate::ui::state::{Spinner, SpinnerSize};
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

#[component]
pub fn SortColumnButton(
    label: &'static str,
    #[prop(into)] active: Signal<bool>,
    #[prop(into)] descending: Signal<bool>,
    on_click: impl Fn() + Copy + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <Button variant=ButtonVariant::AssetSortColumn class="max-sm:min-h-11" aria_pressed=move || active.get().to_string() on_click=move |_| on_click()>
            <span class="asset-sort-arrow" aria_hidden="true">
                {move || if !active.get() { "" } else if descending.get() { "▼" } else { "▲" }}
            </span>
            {label}
        </Button>
    }
}

/// 適用中の条件チップ(「口座: 特定 ✕」)。押すとその条件だけ解除する
#[component]
pub fn SearchTag(
    label: &'static str,
    value: Memo<String>,
    on_clear: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    move || {
        let value = value.get();
        (!value.is_empty()).then(|| {
            let aria_label = format!("{label} {value} を解除");
            view! {
                <Button
                    variant=ButtonVariant::ConditionChip
                    aria_label=aria_label
                    on_click=move |_| on_clear()
                >
                    {format!("{label}: {value}")}
                    <span aria-hidden="true">"✕"</span>
                </Button>
            }
        })
    }
}

/// 検索欄の面。虫眼鏡アイコンとタグ・入力・件数・クリアを1つの箱に収める
#[component]
pub fn SearchBox(
    #[prop(into, optional)] class: Signal<String>,
    /// アイコンサイズ。PC は h-4、スマホは h-5
    #[prop(optional, default = "h-4 w-4")]
    icon_size: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=move || {
            let extra = class.get();
            if extra.is_empty() {
                "search-field".to_string()
            } else {
                format!("search-field {extra}")
            }
        }>
            <svg aria-hidden="true" class=format!("{icon_size} shrink-0 mx-2 text-text-muted") viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="11" cy="11" r="7"></circle>
                <path d="M20 20l-3.5-3.5"></path>
            </svg>
            {children()}
        </div>
    }
}

/// 文字列の部分一致検索の入力欄
#[component]
pub fn SearchTextInput(
    #[prop(into)] placeholder: String,
    #[prop(optional)] testid: Option<&'static str>,
    /// 取得中など、入力を止める場面
    #[prop(into, optional)]
    disabled: Option<Signal<bool>>,
    value: impl Fn() -> String + Copy + 'static + Send + Sync,
    on_input: impl Fn(String) + Copy + 'static + Send + Sync,
) -> impl IntoView {
    view! {
        <input
            type="search"
            class="search-field-input"
            placeholder=placeholder.clone()
            aria-label=placeholder.clone()
            autocomplete="off"
            enterkeyhint="search"
            disabled=move || disabled.is_some_and(|disabled| disabled.get())
            data-testid=move || testid.unwrap_or("")
            prop:value=value
            on:input=move |ev| on_input(event_target_value(&ev))
        />
    }
}

/// 検索欄の実行ボタン。入力が空のあいだは押せない(薄い色)になり、
/// 文字が入ると主の色になる
#[component]
pub fn SearchSubmitButton(
    disabled: impl Fn() -> bool + Copy + 'static + Send + Sync,
    loading: impl Fn() -> bool + Copy + 'static + Send + Sync,
    /// 実行中でないときの読み上げ用の名前
    #[prop(into, optional)]
    label: Option<String>,
    /// 実行中のときの読み上げ用の名前
    #[prop(into, optional)]
    loading_label: Option<String>,
    #[prop(optional)] testid: Option<&'static str>,
) -> impl IntoView {
    let label = label.unwrap_or_else(|| labels::SEARCH.to_string());
    let loading_label = loading_label.unwrap_or_else(|| labels::SEARCH_LOADING.to_string());
    view! {
        <Button
            variant=ButtonVariant::Primary(ButtonSize::Sm)
            class="h-11 shrink-0 lg:h-9"
            submit=true
            disabled=Signal::derive(disabled)
            testid=testid.unwrap_or("")
            aria_label=move || {
                if loading() {
                    loading_label.clone()
                } else {
                    label.clone()
                }
            }
            attr:data-loading=move || loading().then_some("true")
            // 送信はフォームの submit が行う(ボタン自身では検索しない)
            on_click=move |_| {}
        >
            {move || {
                if loading() {
                    view! {
                        <Spinner size=SpinnerSize::Sm class="mr-2" />
                        {labels::SEARCH_LOADING}
                    }
                        .into_any()
                } else {
                    labels::SEARCH.into_any()
                }
            }}
        </Button>
    }
}

/// 絞り込み中の件数と全件数
#[component]
pub fn SearchCount(
    filtered: impl Fn() -> usize + Copy + 'static + Send + Sync,
    total: impl Fn() -> usize + Copy + 'static + Send + Sync,
    #[prop(optional)] testid: Option<&'static str>,
) -> impl IntoView {
    view! {
        <span class="receipts-search-count" data-testid=move || testid.unwrap_or("")>
            <b>{move || labels::count(filtered())}</b>
            {move || format!(" / {}", labels::count(total()))}
        </span>
    }
}

/// 検索条件の一括解除
#[component]
pub fn SearchClearButton(
    disabled: impl Fn() -> bool + Copy + 'static + Send + Sync,
    #[prop(optional)] testid: Option<&'static str>,
    on_clear: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    view! {
        <Button
            variant=ButtonVariant::SearchClear
            disabled=move || disabled()
            testid=testid.unwrap_or("")
            on_click=move |_| on_clear()
        >
            "すべてクリア"
        </Button>
    }
}

/// 「条件を足す:」の行。中身の条件ボタンは各ページが渡す。
/// 開いた小窓(ConditionPanel)もこの中に置く。小窓はこの行の座標を基準に出る
#[component]
pub fn AddConditionRow(
    /// 条件を足すボタンがあるときだけ「条件を足す:」の文言を出す
    #[prop(optional)]
    show_label: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="add-condition-row">
            {show_label.then(|| view! { <span class="add-condition-label">"条件を足す:"</span> })}
            {children()}
        </div>
    }
}

/// 条件の行の右端に置く件数と「すべてクリア」。
/// スマホは条件のボタンと1行に並べない(ボタンが狭くなるため次の行に置く)
#[component]
pub fn ConditionRowEnd(children: Children) -> impl IntoView {
    view! {
        <div class="ml-auto flex w-full items-center justify-end gap-2 lg:w-auto">{children()}</div>
    }
}

/// 「＋ 口座」など条件を足すボタン。開いている小窓は aria-expanded で示し、
/// node_ref は小窓を閉じた後にフォーカスを戻すために使う
#[component]
pub fn AddConditionButton(
    label: &'static str,
    expanded: impl Fn() -> bool + Copy + 'static + Send + Sync,
    node_ref: NodeRef<leptos::html::Button>,
    #[prop(optional)] testid: Option<&'static str>,
    on_click: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    view! {
        <Button
            variant=ButtonVariant::AddCondition
            aria_expanded=move || expanded()
            testid=testid.unwrap_or("")
            node_ref=node_ref
            on_click=move |_| on_click()
        >
            "＋ " {label}
        </Button>
    }
}

/// 「条件を足す」の小窓(PC)・下から出るシート(スマホ)の外枠。
/// 開くたびに作り直され、中身(期間の入力・選択肢など)は各ページが渡す。
/// 背景クリック・Esc で閉じ、面の裏へ Tab で出ないようフォーカスを閉じ込める。
/// PC は AddConditionRow の直下に fixed で出す(位置は親要素=行の座標から
/// 写す)。fixed は祖先の relative や overflow-clip な面に左右されないため、
/// どのページでも同じ位置に出る
#[component]
pub fn ConditionPanel(
    #[prop(into)] title: String,
    on_close: impl Fn() + Copy + 'static + Send + Sync,
    children: Children,
) -> impl IntoView {
    let panel_ref = NodeRef::<leptos::html::Div>::new();
    // PC: 行(親要素)の座標をビューポート座標のまま top/left に写す。
    // スマホは CSS のシートに任せるので、幅をまたいだときは座標を消す
    let sync_position = move || {
        let (Some(panel), Some(window)) = (panel_ref.get(), web_sys::window()) else {
            return;
        };
        let panel: &web_sys::HtmlElement = panel.unchecked_ref();
        let style = panel.style();
        let is_pc = window
            .match_media("(width >= 64rem)")
            .ok()
            .flatten()
            .is_some_and(|query| query.matches());
        if !is_pc {
            let _ = style.remove_property("top");
            let _ = style.remove_property("left");
            return;
        }
        if let Some(row) = panel.parent_element() {
            let rect = row.get_bounding_client_rect();
            let _ = style.set_property("top", &format!("{}px", rect.bottom()));
            let _ = style.set_property("left", &format!("{}px", rect.left()));
        }
    };
    Effect::new(move |_| {
        if let Some(panel) = panel_ref.get() {
            let _ = panel.focus();
        }
        sync_position();
    });
    // scroll はバブルしないため、内側のスクロール領域も拾えるよう
    // document のキャプチャ段で追従させる
    let scroll_sync = web_sys::window()
        .and_then(|window| window.document())
        .map(|document| {
            let listener: Closure<dyn FnMut()> =
                Closure::wrap(Box::new(sync_position) as Box<dyn FnMut()>);
            let _ = document.add_event_listener_with_callback_and_bool(
                "scroll",
                listener.as_ref().unchecked_ref(),
                true,
            );
            send_wrapper::SendWrapper::new((document, listener))
        });
    let on_resize = window_event_listener(ev::resize, move |_| sync_position());
    on_cleanup(move || {
        on_resize.remove();
        if let Some(pair) = scroll_sync {
            let (document, listener) = &*pair;
            let _ = document.remove_event_listener_with_callback_and_bool(
                "scroll",
                listener.as_ref().unchecked_ref(),
                true,
            );
        }
    });
    view! {
        <div class="condition-backdrop" on:click=move |_| on_close() />
        <div
            class="condition-panel"
            role="dialog"
            aria-label=title.clone()
            tabindex="-1"
            node_ref=panel_ref
            on:keydown=move |event| {
                if event.key() == "Escape" {
                    event.prevent_default();
                    on_close();
                    return;
                }
                if event.key() == "Tab" {
                    trap_focus(&event);
                }
            }
        >
            <p class="condition-panel-title">{title.clone()}</p>
            {children()}
        </div>
    }
}

/// 条件小窓の選択肢一覧。選択中は aria-pressed で示す。
/// 単一選択(選んだら閉じる)・複数選択(その場で切り替え)のどちらの条件にも使える
#[component]
pub fn ConditionOptionList(
    options: Memo<Vec<SearchOption>>,
    selected: Memo<Vec<String>>,
    on_toggle: impl Fn(String) + Copy + 'static + Send + Sync,
) -> impl IntoView {
    view! {
        <div class="condition-option-list">
            {move || {
                options
                    .get()
                    .into_iter()
                    .map(|option| {
                        let pressed_value = option.value.clone();
                        let click_value = option.value.clone();
                        view! {
                            <Button
                                variant=ButtonVariant::ConditionOption
                                aria_pressed=move || {
                                    selected.get().contains(&pressed_value).to_string()
                                }
                                on_click=move |_| on_toggle(click_value.clone())
                            >
                                {option.label.clone()}
                            </Button>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
