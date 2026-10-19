//! 「タグ入力 + 条件を足す」式の検索欄に共通する部品。
//! 取引明細と資産管理はこの部品を共有し、条件の中身(期間の入力や
//! 口座・分配金コースの選択肢)だけを各ページが渡す

use crate::support::list_search::SearchOption;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::focus::trap_focus;
use leptos::prelude::*;

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
                "receipts-search-box".to_string()
            } else {
                format!("receipts-search-box {extra}")
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
    placeholder: &'static str,
    #[prop(optional)] testid: Option<&'static str>,
    value: impl Fn() -> String + Copy + 'static + Send + Sync,
    on_input: impl Fn(String) + Copy + 'static + Send + Sync,
) -> impl IntoView {
    view! {
        <input
            type="search"
            class="receipts-search-input"
            placeholder=placeholder
            aria-label=placeholder
            autocomplete="off"
            enterkeyhint="search"
            data-testid=move || testid.unwrap_or("")
            prop:value=value
            on:input=move |ev| on_input(event_target_value(&ev))
        />
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
            <b>{move || format!("{}件", filtered())}</b>
            {move || format!(" / {}件", total())}
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

/// 「条件を足す:」の行。中身の条件ボタンは各ページが渡す
#[component]
pub fn AddConditionRow(children: Children) -> impl IntoView {
    view! {
        <div class="add-condition-row">
            <span class="add-condition-label">"条件を足す:"</span>
            {children()}
        </div>
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
/// 背景クリック・Esc で閉じ、面の裏へ Tab で出ないようフォーカスを閉じ込める
#[component]
pub fn ConditionPanel(
    title: &'static str,
    on_close: impl Fn() + Copy + 'static + Send + Sync,
    children: Children,
) -> impl IntoView {
    let panel_ref = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        if let Some(panel) = panel_ref.get() {
            let _ = panel.focus();
        }
    });
    view! {
        <div class="condition-backdrop" on:click=move |_| on_close() />
        <div
            class="condition-panel"
            role="dialog"
            aria-label=title
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
            <p class="condition-panel-title">{title}</p>
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
