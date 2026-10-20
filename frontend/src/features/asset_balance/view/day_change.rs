use crate::features::asset_balance::format::{
    format_price_date, format_signed_row_valuation_amount, is_negative_valuation,
    is_positive_valuation,
};
use leptos::prelude::*;

/// 「前日比」列の見出しの説明。ホバーと支援技術から同じ内容を読み取れる
pub(crate) const DAY_CHANGE_COLUMN_NOTE: &str =
    "最新の基準価額の日の前日比がわかる行だけ出します。更新待ちの行は CSV を取り込むと出ます";

/// 前日比の列セル。単位あたり(株は1株・投信は基準価額)の円マーク付き金額で、
/// 前日終値がない行は「—」。基準日が他の行と違うときだけ横に日付を小さく出す
pub(crate) fn day_change_cell(change: Option<f64>, date: Option<String>) -> impl IntoView {
    let class = if is_negative_valuation(change) {
        "text-negative"
    } else if is_positive_valuation(change) {
        "text-positive"
    } else {
        "text-text-soft"
    };
    let text = format_signed_row_valuation_amount(change);
    let date_text = date.map(|date| format_price_date(&date));
    view! {
        <span class=format!("font-bold tabular-nums whitespace-nowrap {class}")>{text}</span>
        {date_text.map(|date| {
            view! {
                <span class="ml-1 whitespace-nowrap text-xs text-text-muted tabular-nums">
                    {format!("({date})")}
                </span>
            }
        })}
    }
}

/// 「更新待ち」の前日比セル。最新の基準価額の日の変動でない値を見せないため、
/// 「—」と小さく理由を出す(理由の詳細はセルの title)
pub(crate) fn pending_day_change_cell() -> impl IntoView {
    view! {
        <span class="font-bold tabular-nums whitespace-nowrap text-text-soft">"—"</span>
        <span class="ml-1 whitespace-nowrap text-xs text-text-muted tabular-nums">"更新待ち"</span>
    }
}
