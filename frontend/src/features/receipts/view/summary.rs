use crate::features::receipts::{ReceiptRow, ReceiptTabData, ReceiptsTab, select_header_summary};
use crate::settings::format_amount;
use crate::ui::card::StatTone;
use crate::ui::format::format_iso_date;
use crate::ui::labels;
use crate::ui::summary::{SummaryBand, SummaryBreakdownEntry, SummarySubItem, SummaryTone};
use leptos::prelude::*;
use rust_decimal::Decimal;

pub fn profit_tone(value: Decimal) -> StatTone {
    if value < Decimal::ZERO {
        StatTone::Loss
    } else {
        StatTone::Neutral
    }
}

pub fn header_summary(
    tab: ReceiptsTab,
    data: &ReceiptTabData,
    query: &str,
) -> Vec<(&'static str, Decimal, StatTone)> {
    let api = data
        .summary
        .as_ref()
        .and_then(|s| tab.api_summary_triple(s));
    let values = select_header_summary(api.as_ref(), query, tab.client_summary_triple(&data.rows));
    tab.header_items()
        .into_iter()
        .enumerate()
        .map(|(index, (label, marks_profit))| {
            let tone = if marks_profit {
                profit_tone(values[index])
            } else {
                StatTone::Neutral
            };
            (label, values[index], tone)
        })
        .collect()
}

/// 対象期間の注記。明細の最初と最後の日を出す
pub fn period_note(rows: &[ReceiptRow]) -> Option<String> {
    let from = rows.iter().map(|row| row.date()).min()?;
    let to = rows.iter().map(|row| row.date()).max()?;
    Some(labels::target_period_note(
        &format_iso_date(from),
        &format_iso_date(to),
    ))
}

#[component]
pub fn SummaryStrip(
    tab: ReceiptsTab,
    items: Vec<(&'static str, Decimal, StatTone)>,
) -> impl IntoView {
    // 主役は末尾(税引後)。補助はなし。内訳は先頭2つ(配当金・税額 / 実現損益・税額)。
    // 売却額・取得価額は API 集計になく、一覧の部分合計では欠けると嘘になるため出さない。
    let mut marks: Vec<bool> = tab
        .header_items()
        .into_iter()
        .map(|(_, profit)| profit)
        .collect();
    let mut secondary = items;
    let primary = secondary.pop();
    let primary_mark = marks.pop().unwrap_or(false);
    let format_signed = |value: Decimal, is_profit: bool| {
        let text = format_amount(value);
        if is_profit && value > Decimal::ZERO {
            format!("+{text}")
        } else {
            text
        }
    };
    let to_breakdown = |(label, value, tone): (&'static str, Decimal, StatTone),
                        is_profit: bool| {
        let negative = tone == StatTone::Loss;
        let positive = is_profit && value > Decimal::ZERO;
        SummaryBreakdownEntry {
            label: label.to_string(),
            value: format_signed(value, is_profit),
            negative,
            positive,
            rate: String::new(),
        }
    };
    let to_sub =
        |(label, value, tone): (&'static str, Decimal, StatTone), is_profit: bool| SummarySubItem {
            label: label.to_string(),
            value: format_signed(value, is_profit),
            note: String::new(),
            tone: match tone {
                StatTone::Loss => SummaryTone::Loss,
                StatTone::Neutral => SummaryTone::Neutral,
            },
            extra_class: String::new(),
        };
    let pc_items: Vec<SummarySubItem> = secondary
        .clone()
        .into_iter()
        .zip(marks.clone())
        .map(|(item, profit)| to_sub(item, profit))
        .collect();
    let breakdown: Vec<SummaryBreakdownEntry> = secondary
        .into_iter()
        .zip(marks)
        .map(|(item, profit)| to_breakdown(item, profit))
        .collect();
    let (primary_label, primary_value, primary_negative) = primary
        .map(|(label, value, tone)| {
            (
                label.to_string(),
                format_signed(value, primary_mark),
                tone == StatTone::Loss,
            )
        })
        .unwrap_or_default();
    view! {
        <section
            data-testid="receipt-summary-strip"
            role="region"
            aria-label="集計"
            class="summary-band"
        >
            <SummaryBand
                primary_label=primary_label
                primary_value=primary_value
                primary_negative=primary_negative
                pc_items=pc_items
                breakdown=breakdown
                testid_band="kpi-grid"
                testid_toggle="receipt-summary-details-toggle"
            />
        </section>
    }
}
