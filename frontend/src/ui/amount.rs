use leptos::prelude::*;

/// 金額・率などの数値の表示。書式は呼び出し側の format 関数で作った文字列を
/// 受け取り、この部品が tabular-nums とマイナスの色([data-negative])をまとめる。
/// 周囲の要素(p/dd/td)はそのまま残し、この部品は値の span として使う。
#[component]
pub fn Amount(
    #[prop(into)] text: Signal<String>,
    #[prop(into, optional)] negative: Signal<bool>,
    /// 置き換え前が p などのブロック要素だった場合に付ける
    #[prop(optional)]
    block: bool,
    #[prop(into, optional)] class: Signal<String>,
    #[prop(optional)] testid: Option<&'static str>,
    #[prop(into, optional)] title: Option<Signal<String>>,
) -> impl IntoView {
    view! {
        <span
            class=move || {
                let mut classes = "tabular-nums".to_string();
                if block {
                    classes.push_str(" block");
                }
                let extra = class.get();
                if !extra.is_empty() {
                    classes.push(' ');
                    classes.push_str(&extra);
                }
                classes
            }
            data-negative=move || negative.get().then_some("true")
            data-testid=testid
            title=move || title.map(|title| title.get())
        >
            {move || text.get()}
        </span>
    }
}
