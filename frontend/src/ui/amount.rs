use crate::ui::labels;
use leptos::prelude::*;

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
/// 表示文字列の符号で色を決める。負は種類を問わず赤・正は Delta のとき緑・
/// 0 と欠損は色なし。残高・価格などの Plain は正でも通常色のままにする。
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
    let sign_text = text;
    let color = move || amount_color(kind, negative.get(), positive.get(), &sign_text.get());
    let classes = move || amount_classes(color(), &class.get());
    let negative_attr = move || amount_negative_attr(color());
    let text = move || text.get();
    if block {
        view! {
            <p class=classes data-amount="" data-negative=negative_attr>
                {text}
            </p>
        }
        .into_any()
    } else {
        view! {
            <span class=classes data-amount="" data-negative=negative_attr>
                {text}
            </span>
        }
        .into_any()
    }
}

fn amount_color(kind: AmountKind, negative: bool, positive: bool, text: &str) -> AmountColor {
    let text = text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();
    let mut numeric = text.replace(['¥', ','], "");
    for suffix in [
        labels::THOUSAND_YEN,
        labels::TEN_THOUSAND_YEN,
        labels::YEN,
        "%",
    ] {
        if let Some(value) = numeric.strip_suffix(suffix) {
            numeric = value.to_string();
            break;
        }
    }
    let Ok(value) = numeric.parse::<f64>() else {
        return AmountColor::Neutral;
    };
    // マイナス表示は種類・明示の符号を問わず赤にする。プラス表示は
    // 損益・出入りの Delta(または明示の正)のときだけ緑にし、残高・価格などの
    // Plain は通常色のままにする。0・欠損は常に色なし
    if value < 0.0 {
        AmountColor::Negative
    } else if value > 0.0 && (kind == AmountKind::Delta || negative || positive) {
        AmountColor::Positive
    } else {
        AmountColor::Neutral
    }
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
