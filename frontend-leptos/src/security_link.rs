//! React `components/atoms/SecurityCodeLink.tsx` に対応する。

use crate::asset_balance_domain::normalize_security_code;
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;

/// `formatters.ts` の `SECURITY_CODE_REGEX`（`/^[0-9A-Za-z.]+$/`）に対応する。
pub(crate) fn is_searchable_code(code: &str) -> bool {
    !code.is_empty()
        && code
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '.')
}

/// React `FONT_WEIGHT_CLASS_REGEX` 相当。呼び出し側が font-weight を
/// 指定済みなら `font-bold` を付けない（クラス競合を防ぐため）。
fn has_font_weight_class(class: &str) -> bool {
    const WEIGHTS: [&str; 9] = [
        "font-thin",
        "font-extralight",
        "font-light",
        "font-normal",
        "font-medium",
        "font-semibold",
        "font-bold",
        "font-extrabold",
        "font-black",
    ];
    class.split_whitespace().any(|token| {
        let base = token.rsplit(':').next().unwrap_or(token);
        WEIGHTS.iter().any(|weight| {
            base.strip_prefix(weight).is_some_and(|rest| {
                rest.is_empty()
                    || rest
                        .chars()
                        .next()
                        .is_some_and(|c| !c.is_ascii_alphanumeric() && c != '_')
            })
        })
    })
}

/// 銘柄コードを正規化して検索詳細 `/search?code=` へのリンクにする。
/// 正規化後が空なら "-"、想定外の文字列はリンク化せずテキストのみ（React 同様）。
#[component]
pub(crate) fn SecurityCodeLink(
    #[prop(into)] value: String,
    #[prop(optional)] class: Option<String>,
) -> impl IntoView {
    let code = normalize_security_code(&value);
    if code.is_empty() {
        return view! { <span>"-"</span> }.into_any();
    }
    if !is_searchable_code(&code) {
        return view! { <span>{code}</span> }.into_any();
    }
    let href = format!("/search?code={}", urlencoding::encode(&code));
    let has_weight = class.as_deref().map(has_font_weight_class).unwrap_or(false);
    let classes = format!(
        "security-code-link text-blue-700 underline-offset-2 hover:text-blue-900 hover:underline focus-visible:rounded-sm focus-visible:outline-2 focus-visible:outline-blue-500{}{}",
        if has_weight { "" } else { " font-bold" },
        class.map(|extra| format!(" {extra}")).unwrap_or_default(),
    );
    view! { <a href=href class=classes data-search=code>{code.clone()}</a> }.into_any()
}

/// `toDisplayText` に対応する。空白のみ・空文字は "-" 表示。
fn display_text(value: &str) -> String {
    let text = value.trim();
    if text.is_empty() {
        "-".to_string()
    } else {
        text.to_string()
    }
}

/// `createInstrumentCopyText` に対応する。コピー内容は `銘柄名(コード)`。
fn instrument_copy_text(name: &str, code: Option<&str>) -> String {
    let display = display_text(name);
    let normalized = normalize_security_code(code.unwrap_or_default());
    if normalized.is_empty() {
        display
    } else {
        format!("{display}({normalized})")
    }
}

/// `navigator.clipboard?.writeText` 相当。API が無い環境では何もしない。
fn copy_to_clipboard(text: String) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let navigator = window.navigator();
    let Ok(clipboard) = js_sys::Reflect::get(&navigator, &JsValue::from_str("clipboard")) else {
        return;
    };
    if clipboard.is_null() || clipboard.is_undefined() {
        return;
    }
    let Ok(write_text) = js_sys::Reflect::get(&clipboard, &JsValue::from_str("writeText")) else {
        return;
    };
    let Ok(write_text) = write_text.dyn_into::<js_sys::Function>() else {
        return;
    };
    let Ok(promise) = write_text
        .call1(&clipboard, &JsValue::from_str(&text))
        .and_then(|result| result.dyn_into::<js_sys::Promise>())
    else {
        return;
    };
    leptos::task::spawn_local(async move {
        if let Err(error) = JsFuture::from(promise).await {
            web_sys::console::warn_2(
                &JsValue::from_str("クリップボードへのコピーに失敗しました"),
                &error,
            );
        }
    });
}

/// React `CopyableInstrumentName` に対応する。
/// クリックで `銘柄名(コード)` をクリップボードへコピーするボタン。
#[component]
pub(crate) fn CopyableInstrumentName(
    #[prop(into)] name: String,
    #[prop(optional)] code: Option<String>,
) -> impl IntoView {
    let display = display_text(&name);
    let copy_text = instrument_copy_text(&name, code.as_deref());
    let aria_label = format!("{copy_text} をコピー");
    view! {
        <button
            type="button"
            aria-label=aria_label
            on:click=move |_| copy_to_clipboard(copy_text.clone())
            class="group inline-flex cursor-pointer items-center gap-0.5 border-0 bg-transparent p-0 text-left text-inherit focus-visible:rounded-sm focus-visible:outline-2 focus-visible:outline-blue-500"
        >
            <span>{display}</span>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                aria-hidden="true"
                class="rounded p-0.5 text-gray-400 opacity-100 group-hover:text-gray-600 group-hover:opacity-100 group-focus-visible:opacity-100 sm:opacity-0 sm:group-hover:opacity-100"
            >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
            </svg>
        </button>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn searchable_code_matches_react_regex() {
        assert!(is_searchable_code("7203"));
        assert!(is_searchable_code("BRK.B"));
        assert!(!is_searchable_code(""));
        assert!(!is_searchable_code("7203: トヨタ"));
        assert!(!is_searchable_code("７２０３"));
    }

    #[test]
    fn copy_text_matches_react_format() {
        assert_eq!(
            instrument_copy_text("トヨタ自動車", Some("7203")),
            "トヨタ自動車(7203)"
        );
        assert_eq!(instrument_copy_text("トヨタ自動車", None), "トヨタ自動車");
        assert_eq!(instrument_copy_text("  ", Some("7203")), "-(7203)");
        // ラベル形式・小文字のコードは正規化してから連結する
        assert_eq!(
            instrument_copy_text("名", Some("7203: トヨタ自動車")),
            "名(7203)"
        );
        assert_eq!(instrument_copy_text("名", Some("brk.b")), "名(BRK.B)");
        assert_eq!(instrument_copy_text("名", Some("  ")), "名");
    }

    #[test]
    fn font_weight_detection_matches_react_regex() {
        assert!(has_font_weight_class("font-semibold"));
        assert!(has_font_weight_class("text-xs sm:font-medium"));
        assert!(has_font_weight_class("hover:font-bold"));
        assert!(!has_font_weight_class("text-xs"));
        assert!(!has_font_weight_class("font-boldx"));
        assert!(!has_font_weight_class(""));
    }
}
