use leptos::prelude::*;
use wasm_bindgen::JsCast;

const FOCUSABLE_SELECTOR: &str =
    "button:not([disabled]), [href], input, select, textarea, [tabindex]:not([tabindex=\"-1\"])";

const SECONDARY_BUTTON_CLASS: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print border border-slate-300 bg-white text-slate-800 hover:border-slate-500 hover:bg-slate-50 px-4 py-2 text-sm max-sm:min-h-[44px]";
const CONFIRM_BUTTON_CLASS: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print border border-danger text-danger hover:bg-danger hover:text-white px-3 py-1.5 text-sm max-sm:min-h-[44px]";

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
        .map(|active| focusables.iter().any(|item| same_element(item, active)))
        .unwrap_or(false);
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
    item_count: usize,
    confirm_label: &'static str,
    loading: Memo<bool>,
    on_confirm: impl Fn() + Clone + 'static,
    on_cancel: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let dialog_ref = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        if let Some(dialog) = dialog_ref.get() {
            let _ = dialog.focus();
        }
    });
    let overlay_cancel = on_cancel.clone();
    let key_cancel = on_cancel.clone();
    let close_cancel = on_cancel.clone();
    let cancel = on_cancel.clone();
    view! {
        <div
            node_ref=dialog_ref
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4"
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
                <div class="rounded-lg bg-white shadow-lg">
                    <div class="flex items-center justify-between border-b border-border px-4 py-3">
                        <h5 id="confirm-delete-title" class="text-danger font-semibold">
                            {title}
                        </h5>
                        <button
                            type="button"
                            class="text-gray-500 hover:text-gray-700 focus:outline-none focus:ring-2 focus:ring-primary/50 rounded"
                            aria-label="閉じる"
                            on:click=move |_| close_cancel()
                        >
                            <span aria-hidden="true">"✕"</span>
                        </button>
                    </div>
                    <div id="confirm-delete-desc" class="px-4 py-4 text-base text-dark">
                        <p>{description}</p>
                        <p class="mt-2 text-sm text-secondary">
                            "対象: "
                            <strong class="text-danger">{item_count}"件"</strong>
                            "のデータ"
                        </p>
                        <p class="mt-3 rounded-md bg-danger/10 px-3 py-2 text-sm text-danger">
                            <strong>"⚠ この操作は取り消せません。"</strong>
                            "削除されたデータは復元できません。"
                        </p>
                    </div>
                    <div class="flex justify-end gap-2 border-t border-border px-4 py-3">
                        <button
                            type="button"
                            class=SECONDARY_BUTTON_CLASS
                            on:click=move |_| cancel()
                        >
                            "キャンセル"
                        </button>
                        <button
                            type="button"
                            class=CONFIRM_BUTTON_CLASS
                            disabled=move || loading.get()
                            on:click=move |_| on_confirm()
                        >
                            {move || {
                                loading.get().then(|| {
                                    view! {
                                        <span class="inline-flex items-center mr-2">
                                            <svg
                                                class="animate-spin h-4 w-4"
                                                xmlns="http://www.w3.org/2000/svg"
                                                fill="none"
                                                viewBox="0 0 24 24"
                                                role="status"
                                                aria-label="読み込み中..."
                                            >
                                                <circle
                                                    class="opacity-25"
                                                    cx="12"
                                                    cy="12"
                                                    r="10"
                                                    stroke="currentColor"
                                                    stroke-width="4"
                                                />
                                                <path
                                                    class="opacity-75"
                                                    fill="currentColor"
                                                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                                />
                                            </svg>
                                        </span>
                                    }
                                })
                            }}
                            {confirm_label}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
