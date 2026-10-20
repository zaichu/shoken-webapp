use crate::features::asset_balance::format::{
    format_price_date, format_signed_row_valuation_amount, is_negative_valuation,
    is_positive_valuation,
};
use leptos::prelude::*;

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
