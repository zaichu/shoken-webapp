use leptos::prelude::*;

/// 金額・率の表示種別。正負の色は明示符号と表示文字列から共通判定する。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AmountKind {
    #[default]
    Plain,
    Delta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AmountColor {
    Neutral,
    Positive,
    Negative,
}

/// 金額・率の表示。書式は呼び出し側の format 関数で作った文字列を
/// 受け取り、この部品が tabular-nums・nowrap と正負の色をまとめる。
/// 正は緑・負は赤( data-negative )・0 は色なし。
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
    let sign_text = text.clone();
    let color = move || amount_color(kind, negative.get(), positive.get(), &sign_text.get());
    let classes = move || amount_classes(color(), &class.get());
    let negative_attr = move || amount_negative_attr(color());
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

fn amount_color(kind: AmountKind, negative: bool, positive: bool, text: &str) -> AmountColor {
    let negative = negative || signed_text_is_nonzero(text, '-');
    let positive = positive || signed_text_is_nonzero(text, '+');
    match kind {
        AmountKind::Plain | AmountKind::Delta if negative => AmountColor::Negative,
        AmountKind::Plain | AmountKind::Delta if positive => AmountColor::Positive,
        AmountKind::Plain | AmountKind::Delta => AmountColor::Neutral,
    }
}

fn signed_text_is_nonzero(text: &str, sign: char) -> bool {
    let Some(value) = text.trim_start().strip_prefix(sign) else {
        return false;
    };
    let value = value.trim_start_matches('¥').replace(',', "");
    let numeric =
        value.trim_end_matches(|character: char| !character.is_ascii_digit() && character != '.');
    numeric.parse::<f64>().map_or(true, |value| value != 0.0)
}

fn amount_classes(color: AmountColor, extra: &str) -> String {
    let mut base = String::from("tabular-nums whitespace-nowrap");
    if !extra.is_empty() {
        base.push(' ');
        base.push_str(extra);
    }
    match color {
        AmountColor::Neutral => base,
        AmountColor::Positive => {
            base.push_str(" text-positive");
            base
        }
        AmountColor::Negative => base,
    }
}

const fn amount_negative_attr(color: AmountColor) -> Option<&'static str> {
    match color {
        AmountColor::Negative => Some("true"),
        AmountColor::Neutral | AmountColor::Positive => None,
    }
}

#[cfg(test)]
mod tests;
