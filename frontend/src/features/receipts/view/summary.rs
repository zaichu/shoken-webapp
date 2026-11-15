use crate::features::receipts::{ReceiptRow, ReceiptTabData, ReceiptsTab, select_header_summary};
use crate::settings::format_amount;
use crate::ui::card::StatTone;
use crate::ui::format::date_parts;
use crate::ui::labels::{
    PERIOD, REALIZED_AFTER_TAX, RECEIVED_DIVIDEND_AFTER_TAX, SUMMARY, receipt_period_short,
    receipt_summary_dates,
};
use crate::ui::summary::{SummaryBand, SummaryStat, SummaryTone};
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

/// 帯に出す小さな期間。「2026/01 〜 2026/12」。行の日付は YYYY-MM-DD のため年月だけ切り出す
pub fn period_note_short(rows: &[ReceiptRow]) -> Option<String> {
    let from = rows.iter().map(super::super::ReceiptItem::date).min()?;
    let to = rows.iter().map(super::super::ReceiptItem::date).max()?;
    Some(receipt_period_short(&year_month(from)?, &year_month(to)?))
}

fn year_month(date: &str) -> Option<String> {
    date_parts(date).map(|(year, month, _)| format!("{year}/{month:02}"))
}

#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub fn SummaryStrip(
    tab: ReceiptsTab,
    items: Vec<(&'static str, Decimal, StatTone)>,
    #[prop(optional, into)] period: String,
    count: usize,
) -> impl IntoView {
    // 主役は末尾(税引後)。2項目めは期間。内訳の税引前・税額は帯に出さない
    // (行の詳細ダイアログ・PCの表と同じ決まりで符号と色を付けていた表示はやめた)。
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
    let count_text = crate::ui::labels::count(count);
    let (primary_label, primary_value, primary_tone) = primary.map_or(
        (String::new(), String::new(), SummaryTone::Neutral),
        |(label, value, tone)| {
            let display = match tab {
                ReceiptsTab::Dividend => RECEIVED_DIVIDEND_AFTER_TAX,
                _ => REALIZED_AFTER_TAX,
            };
            let _ = label;
            (
                display.to_string(),
                format_signed(value, primary_mark),
                if tone == StatTone::Loss {
                    SummaryTone::Loss
                } else if primary_mark && value > Decimal::ZERO {
                    SummaryTone::Gain
                } else {
                    SummaryTone::Neutral
                },
            )
        },
    );
    view! {
        <section
            data-testid="receipt-summary-strip"
            role="region"
            aria-label=SUMMARY
            class="summary-band"
        >
            <SummaryBand
                primary=SummaryStat {
                    label: primary_label,
                    value: primary_value,
                    note: String::new(),
                    tone: primary_tone,
                }
                pc_items=vec![
                    SummaryStat {
                        label: PERIOD.to_string(),
                        value: period.clone(),
                        note: String::new(),
                        tone: SummaryTone::Neutral,
                    },
                ]
                mobile_dates=receipt_summary_dates(&period, &count_text)
                pc_dates=vec![count_text]
                testid_band="kpi-grid"
            />
        </section>
    }
}
