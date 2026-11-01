use leptos::prelude::*;

/// 金額・率の値の種類。増減・損益だけ色と符号で正負を示し、
/// それ以外の金額(配当・売却額・税額など)は通常の文字色で出す。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AmountKind {
    #[default]
    Plain,
    Delta,
}

/// 金額・率の表示。書式は呼び出し側の format 関数で作った文字列を
/// 受け取り、この部品が tabular-nums・nowrap と正負の色をまとめる。
/// Delta は正が緑・負が赤( data-negative )・0 が灰、Plain は通常色。
/// block=true のときこの部品が p を出すので、呼び出し側で p で包まない。
/// block=false(既定)はインラインの span を出し、周囲の要素(dd/td 等)はそのまま残す。
#[component]
pub fn Amount(
    #[prop(into)] text: Signal<String>,
    #[prop(into, optional)] negative: Signal<bool>,
    #[prop(into, optional)] positive: Signal<bool>,
    #[prop(optional)] kind: AmountKind,
    /// 値を段落(p)で出す。既定はインラインの span
    #[prop(optional)]
    block: bool,
    #[prop(into, optional)] class: Signal<String>,
) -> impl IntoView {
    let classes = move || amount_classes(kind, negative.get(), positive.get(), &class.get());
    let negative_attr = move || amount_negative_attr(kind, negative.get());
    let text = move || text.get();
    if block {
        view! {
            <p class=classes data-negative=negative_attr>
                {text}
            </p>
        }
        .into_any()
    } else {
        view! {
            <span class=classes data-negative=negative_attr>
                {text}
            </span>
        }
        .into_any()
    }
}

fn amount_classes(kind: AmountKind, negative: bool, positive: bool, extra: &str) -> String {
    let mut base = String::from("tabular-nums whitespace-nowrap");
    if !extra.is_empty() {
        base.push(' ');
        base.push_str(extra);
    }
    match kind {
        AmountKind::Plain => base,
        AmountKind::Delta => {
            if negative {
                base
            } else if positive {
                base.push_str(" text-positive");
                base
            } else {
                base.push_str(" text-text-soft");
                base
            }
        }
    }
}

fn amount_negative_attr(kind: AmountKind, negative: bool) -> Option<&'static str> {
    match kind {
        AmountKind::Delta if negative => Some("true"),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
