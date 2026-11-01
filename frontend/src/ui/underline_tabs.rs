use wasm_bindgen::JsCast;

#[cfg(test)]
mod tests;

pub fn next_tab_index(count: usize, current: usize, key: &str) -> Option<usize> {
    if count == 0 {
        return None;
    }
    match key {
        "ArrowRight" => Some((current + 1) % count),
        "ArrowLeft" => Some((current + count - 1) % count),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => None,
    }
}

pub fn focus_tab_by_id(id: &str) {
    let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(id))
    else {
        return;
    };
    if let Some(element) = element.dyn_ref::<web_sys::HtmlElement>() {
        let _ = element.focus();
    }
}
