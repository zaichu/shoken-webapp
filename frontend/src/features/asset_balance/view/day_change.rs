use crate::features::asset_balance::format::{
    format_signed_row_valuation_amount, is_negative_valuation, is_positive_valuation,
};
use leptos::prelude::*;

/// 単位あたり(株は1株・投信は基準価額)の前日比サブライン。表の価格セルの下に出す。
/// 前日終値がない銘柄は「—」(集計からも除く)
pub(crate) fn day_change_subline(change: Option<f64>) -> impl IntoView {
    let class = if is_negative_valuation(change) {
        "text-negative"
    } else if is_positive_valuation(change) {
        "text-positive"
    } else {
        "text-text-soft"
    };
    let text = format_signed_row_valuation_amount(change);
    view! {
        <div class=format!("asset-price-date {class}")>{format!("前日比 {text}")}</div>
    }
}
