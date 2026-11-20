#[cfg(test)]
mod tests;

use crate::ui::labels;
use crate::ui::settings as settings_ui;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

pub(crate) const SECTION_IDS: [&str; 8] = [
    "account",
    "display",
    "data",
    "security",
    "devices",
    "login-history",
    "operation-log",
    "danger",
];

pub(crate) const TOC_ENTRIES: [(&str, &str); 8] = [
    ("account", labels::ACCOUNT_BUTTON),
    ("display", labels::SETTINGS_DISPLAY_SECTION),
    ("data", labels::SETTINGS_DATA_SECTION),
    ("security", labels::SETTINGS_SECURITY_SECTION),
    ("devices", labels::REGISTERED_DEVICES_SECTION),
    ("login-history", labels::LOGIN_HISTORY_SECTION),
    ("operation-log", labels::OPERATION_LOG_SECTION),
    ("danger", labels::SETTINGS_DANGER_SECTION),
];

const UNLOCK_EVENTS: [&str; 3] = ["pointerdown", "wheel", "keydown"];

pub(crate) fn section_for_fragment(fragment: &str) -> String {
    let id = fragment.trim_start_matches('#');
    if SECTION_IDS.contains(&id) {
        id.to_string()
    } else {
        SECTION_IDS[0].to_string()
    }
}

pub(crate) fn initial_section() -> String {
    let fragment = web_sys::window()
        .and_then(|window| window.location().hash().ok())
        .unwrap_or_default();
    section_for_fragment(&fragment)
}

/// 目次の並びを DOM から読む(目次の並びと本文の並びは同じ前提)。
/// 並び替えがあっても追随がずれないようにするため、順序の正本は持たない
fn toc_ids_in_dom_order(document: &web_sys::Document) -> Vec<String> {
    let mut ids = Vec::new();
    let selector = format!(
        "nav[aria-label=\"{}\"] a[href^=\"/settings#\"]",
        labels::SETTINGS_TOC_LABEL
    );
    let Ok(links) = document.query_selector_all(&selector) else {
        return ids;
    };
    for index in 0..links.length() {
        let Some(anchor) = links.item(index) else {
            continue;
        };
        let anchor: &web_sys::Element = anchor.unchecked_ref();
        let Some(href) = anchor.get_attribute("href") else {
            continue;
        };
        if let Some(id) = href.strip_prefix("/settings#")
            && SECTION_IDS.contains(&id)
            && !ids.iter().any(|known| known == id)
        {
            ids.push(id.to_string());
        }
    }
    ids
}

pub(crate) fn pick_current<'a>(
    ids: &'a [String],
    tops: &[f64],
    line: f64,
    fallback: &'a str,
) -> &'a str {
    let mut current = fallback;
    for (id, top) in ids.iter().zip(tops.iter()) {
        if *top <= line {
            current = id;
        }
    }
    current
}

fn add_capture_listener(
    document: &web_sys::Document,
    name: &str,
    callback: &wasm_bindgen::closure::Closure<dyn FnMut()>,
) {
    let _ = document.add_event_listener_with_callback_and_bool(
        name,
        callback.as_ref().unchecked_ref(),
        true,
    );
}

fn remove_capture_listener(
    document: &web_sys::Document,
    name: &str,
    callback: &wasm_bindgen::closure::Closure<dyn FnMut()>,
) {
    let _ = document.remove_event_listener_with_callback_and_bool(
        name,
        callback.as_ref().unchecked_ref(),
        true,
    );
}

/// 目次の強調をスクロールに追随させる。押下直後は押した項目に固定し、
/// 自分の指での操作があってから追随を再開する(末尾の項目は上端まで
/// 届かないため、押した直後のスクロールで上書きするとずれる)。
pub(crate) fn track_toc(active: RwSignal<String>, locked: RwSignal<bool>) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    // scroll はバブルしないため、内側のスクロール領域も拾えるよう
    // document のキャプチャ段で追従させる
    let sync = move || {
        if locked.get_untracked() {
            return;
        }
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        let Some(main) = document.get_element_by_id("main-content") else {
            return;
        };
        let header_bottom = document
            .query_selector(".pc-header")
            .ok()
            .flatten()
            .map_or_default(|header| header.get_bounding_client_rect().height());
        let line = main.get_bounding_client_rect().top() + header_bottom + 8.0;
        let ids = toc_ids_in_dom_order(&document);
        let fallback = ids.first().map_or(SECTION_IDS[0], String::as_str);
        let tops: Vec<f64> = ids
            .iter()
            .map(|id| {
                document
                    .get_element_by_id(id)
                    .map_or(f64::MAX, |section| section.get_bounding_client_rect().top())
            })
            .collect();
        active.set(pick_current(&ids, &tops, line, fallback).to_string());
    };
    let scroll_listener: wasm_bindgen::closure::Closure<dyn FnMut()> =
        wasm_bindgen::closure::Closure::wrap(Box::new(sync) as Box<dyn FnMut()>);
    add_capture_listener(&document, "scroll", &scroll_listener);
    let unlock: wasm_bindgen::closure::Closure<dyn FnMut()> = wasm_bindgen::closure::Closure::wrap(
        Box::new(move || locked.set(false)) as Box<dyn FnMut()>,
    );
    for name in UNLOCK_EVENTS {
        add_capture_listener(&document, name, &unlock);
    }
    let pair = send_wrapper::SendWrapper::new((document, scroll_listener, unlock));
    on_cleanup(move || {
        let (document, scroll_listener, unlock) = &*pair;
        remove_capture_listener(document, "scroll", scroll_listener);
        for name in UNLOCK_EVENTS {
            remove_capture_listener(document, name, unlock);
        }
    });
}

#[component]
pub(crate) fn TocNav(active: RwSignal<String>, locked: RwSignal<bool>) -> impl IntoView {
    view! {
        <settings_ui::SettingsNavigation>
                {TOC_ENTRIES.iter().map(|(id, label)| {
                    let id = *id;
                    let label = *label;
                    let link_id = id.to_string();
                    let click_id = id.to_string();
                    view! {
                        <li>
                            <a
                                href={format!("/settings#{id}")}
                                data-testid={format!("settings-toc-{id}")}
                                aria-current=move || (active.get() == link_id).then_some("true")
                                on:click=move |_| {
                                    active.set(click_id.clone());
                                    locked.set(true);
                                }
                                class="settings-toc-link"
                            >
                                {label}
                            </a>
                        </li>
                    }
                }).collect_view()}
        </settings_ui::SettingsNavigation>
    }
}
