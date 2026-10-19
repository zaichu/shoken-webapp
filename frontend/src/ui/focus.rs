use wasm_bindgen::JsCast;

const FOCUSABLE_SELECTOR: &str =
    "button:not([disabled]), [href], input, select, textarea, [tabindex]:not([tabindex=\"-1\"])";

fn same_element(a: &web_sys::HtmlElement, b: &web_sys::HtmlElement) -> bool {
    a.unchecked_ref::<web_sys::Node>()
        .is_same_node(Some(b.unchecked_ref()))
}

/// 開いた面の外へ Tab・Shift+Tab でフォーカスが出ないようにする。
/// 背景の面で操作を塞いでいるあいだ、下の操作要素をキーボードで触れなくするため
pub(crate) fn trap_focus(event: &web_sys::KeyboardEvent) {
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
