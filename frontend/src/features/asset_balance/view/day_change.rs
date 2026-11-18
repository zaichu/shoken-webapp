use crate::features::asset_balance::format as balance_format;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::asset_balance::DayChangeNote;
use crate::ui::format;
use crate::ui::labels;
use leptos::prelude::*;

/// 前日比の列セル。単位あたり(株は1株・投信は基準価額)の円マーク付き金額で、
/// 前日終値がない行は「—」。基準日が他の行と違うときだけ横に日付を小さく出す
pub(crate) fn day_change_cell(change: Option<f64>, date: Option<String>) -> impl IntoView {
    let negative = balance_format::is_negative_valuation(change);
    let positive = balance_format::is_positive_valuation(change);
    let text = balance_format::format_signed_row_valuation_amount(change);
    let date_text = date.map(|date| format::format_price_date(&date));
    view! {
        <Amount
            kind=AmountKind::Delta
            text=text
            negative=negative
            positive=positive
            class="font-bold"
        />
        {date_text.map(|date| {
            view! { <DayChangeNote text=format!("({date})") /> }
        })}
    }
}

/// 「更新待ち」の前日比セル。最新の基準価額の日の変動でない値を見せないため、
/// 「—」と小さく理由を出す(理由の詳細はセルの title)
pub(crate) fn pending_day_change_cell() -> impl IntoView {
    view! {
        <span class="font-bold tabular-nums whitespace-nowrap text-text-soft">{labels::MISSING}</span>
        <DayChangeNote text=labels::PENDING_UPDATE.to_string() />
    }
}
