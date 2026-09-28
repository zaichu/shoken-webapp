use leptos::prelude::*;

/// 金額・率などの数値の表示。書式は呼び出し側の format 関数で作った文字列を
/// 受け取り、この部品が tabular-nums とマイナスの色([data-negative])をまとめる。
/// block=true のときこの部品が p を出すので、呼び出し側で p で包まない。
/// block=false(既定)はインラインの span を出し、周囲の要素(dd/td 等)はそのまま残す。
#[component]
pub fn Amount(
    #[prop(into)] text: Signal<String>,
    #[prop(into, optional)] negative: Signal<bool>,
    /// 値を段落(p)で出す。既定はインラインの span
    #[prop(optional)]
    block: bool,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] testid: Option<&'static str>,
    #[prop(into, optional)] title: Option<Signal<String>>,
) -> impl IntoView {
    let classes = move || {
        let extra = class.get();
        if extra.is_empty() {
            "tabular-nums".to_string()
        } else {
            format!("tabular-nums {extra}")
        }
    };
    let negative = move || negative.get().then_some("true");
    let text = move || text.get();
    let title = move || title.map(|title| title.get());
    if block {
        view! {
            <p class=classes data-negative=negative data-testid=testid title=title>
                {text}
            </p>
        }
        .into_any()
    } else {
        view! {
            <span class=classes data-negative=negative data-testid=testid title=title>
                {text}
            </span>
        }
        .into_any()
    }
}
