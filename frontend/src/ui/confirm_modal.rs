use crate::ui::button::{Button, ButtonSize, ButtonVariant, IconButton, IconButtonVariant};
use crate::ui::elements::{Spinner, SpinnerSize};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const FOCUSABLE_SELECTOR: &str =
    "button:not([disabled]), [href], input, select, textarea, [tabindex]:not([tabindex=\"-1\"])";

fn same_element(a: &web_sys::HtmlElement, b: &web_sys::HtmlElement) -> bool {
    a.unchecked_ref::<web_sys::Node>()
        .is_same_node(Some(b.unchecked_ref()))
}

fn trap_focus(event: &web_sys::KeyboardEvent) {
    let Some(container) = event
        .current_target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Ok(list) = container.query_selector_all(FOCUSABLE_SELECTOR) else {
        return;
    };
    let focusables: Vec<web_sys::HtmlElement> = (0..list.length())
        .filter_map(|index| list.item(index))
        .filter_map(|node| node.dyn_into::<web_sys::HtmlElement>().ok())
        .collect();
    if focusables.is_empty() {
        return;
    }
    let first = &focusables[0];
    let last = &focusables[focusables.len() - 1];
    let active = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element())
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
    let in_trap = active
        .as_ref()
        .is_some_and(|active| focusables.iter().any(|item| same_element(item, active)));
    if !in_trap {
        // dialog 自体にフォーカスがある等、トラップ対象外の場合: Shift+Tab は末尾、Tab は先頭へ
        event.prevent_default();
        let _ = if event.shift_key() { last } else { first }.focus();
    } else if let Some(active) = active {
        let edge = if event.shift_key() { first } else { last };
        if same_element(&active, edge) {
            event.prevent_default();
            let _ = if event.shift_key() { last } else { first }.focus();
        }
    }
}

#[component]
pub fn ConfirmDeleteModal(
    title: String,
    description: String,
    #[prop(optional)] item_count: Option<usize>,
    confirm_label: &'static str,
    loading: Memo<bool>,
    #[prop(optional)] error: Option<RwSignal<Option<String>>>,
    on_confirm: impl Fn() + Clone + 'static,
    on_cancel: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let dialog_ref = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.focus();
        }
    });
    // 削除要求が飛んでいる間に閉じると結果を追えなくなるため、閉じ操作は完了まで無効化する
    let try_cancel = move || {
        if !loading.get() {
            on_cancel();
        }
    };
    let overlay_cancel = try_cancel.clone();
    let key_cancel = try_cancel.clone();
    let close_cancel = try_cancel.clone();
    let cancel = try_cancel.clone();
    view! {
        <div
            node_ref=dialog_ref
            class="fixed inset-0 z-50 flex items-center justify-center bg-scrim/50 p-4"
            on:click=move |_| overlay_cancel()
            on:keydown=move |event| {
                if event.key() == "Escape" {
                    key_cancel();
                    return;
                }
                if event.key() == "Tab" {
                    trap_focus(&event);
                }
            }
            role="dialog"
            aria-modal="true"
            aria-labelledby="confirm-delete-title"
            aria-describedby="confirm-delete-desc"
            tabindex="-1"
        >
            <div
                class="w-full max-w-md"
                role="presentation"
                on:click=move |event| event.stop_propagation()
            >
                <div class="rounded-lg bg-surface shadow-lg">
                    <div class="flex items-center justify-between border-b border-border px-4 py-3">
                        <h5 id="confirm-delete-title" class="text-negative font-semibold">
                            {title}
                        </h5>
                        <IconButton
                            variant=IconButtonVariant::Close
                            aria_label="閉じる".to_string()
                            disabled=move || loading.get()
                            on_click=move |_| close_cancel()
                        >
                            <span aria-hidden="true">"✕"</span>
                        </IconButton>
                    </div>
                    <div id="confirm-delete-desc" class="px-4 py-4 text-base text-text-deep">
                        <p>{description}</p>
                        {item_count.map(|count| {
                            view! {
                                <p class="mt-2 text-sm text-text-quiet">
                                    "対象: "
                                    <strong class="text-negative">{count}"件"</strong>
                                    "のデータ"
                                </p>
                            }
                                .into_any()
                        })}
                        <p class="mt-3 rounded-md bg-negative/10 px-3 py-2 text-sm text-negative">
                            <strong>"⚠ この操作は取り消せません。"</strong>
                            "削除されたデータは復元できません。"
                        </p>
                        {move || {
                            error
                                .and_then(|error| error.get())
                                .map(|message| {
                                    view! {
                                        <p
                                            class="mt-3 rounded-md border border-negative-border bg-negative-soft px-3 py-2 text-sm text-negative-vivid"
                                            role="alert"
                                        >
                                            {message}
                                        </p>
                                    }
                                })
                        }}
                    </div>
                    <div class="flex justify-end gap-2 border-t border-border px-4 py-3">
                        <Button
                            variant=ButtonVariant::Secondary(ButtonSize::Md)
                            class="no-print"
                            disabled=move || loading.get()
                            on_click=move |_| cancel()
                        >
                            "キャンセル"
                        </Button>
                        <Button
                            variant=ButtonVariant::Danger(ButtonSize::Sm)
                            class="no-print"
                            disabled=move || loading.get()
                            on_click=move |_| on_confirm()
                        >
                            {move || {
                                loading
                                    .get()
                                    .then(|| view! { <Spinner size=SpinnerSize::Sm class="mr-2" /> })
                            }}
                            {confirm_label}
                        </Button>
                    </div>
                </div>
            </div>
        </div>
    }
}
