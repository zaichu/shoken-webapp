use crate::features::receipts::{ReceiptRow, ReceiptTabData, ReceiptsTab, select_header_summary};
use crate::settings::format_amount;
use crate::ui::card::StatTone;
use crate::ui::format::date_parts;
use crate::ui::labels::{
    DIVIDEND_AMOUNT, DIVIDEND_BEFORE_TAX, REALIZED_AFTER_TAX, REALIZED_BEFORE_TAX,
    REALIZED_PROFIT_LOSS, RECEIVED_DIVIDEND_AFTER_TAX, SUMMARY, TAX, receipt_period_short,
};
use crate::ui::summary::{SummaryBand, SummaryBreakdownEntry, SummarySubItem, SummaryTone};
use leptos::prelude::*;
use rust_decimal::Decimal;

/// 帯の内訳に出す項目が税額かどうか。税額は出ていくお金として符号を反転する
fn is_band_tax(label: &str) -> bool {
    label == TAX
}

/// 帯の内訳に出す項目が税引前・受取額・損益かどうか。詳細ダイアログ・PCの表と同じ決まりで符号と色を付ける
fn is_band_profit(label: &str) -> bool {
    label == DIVIDEND_AMOUNT || label == REALIZED_PROFIT_LOSS
}

/// 帯の内訳の表示値・正負・PC用の種別。税額は正を「-」・還付(負)を「+」に反転し、0は通常色にする
pub(crate) fn band_value(label: &'static str, value: Decimal) -> (String, bool, bool, SummaryTone) {
    let text = format_amount(value);
    if is_band_tax(label) {
        if value > Decimal::ZERO {
            (format!("-{text}"), true, false, SummaryTone::Loss)
        } else if value < Decimal::ZERO {
            (
                format!("+{}", format_amount(-value)),
                false,
                true,
                SummaryTone::Gain,
            )
        } else {
            (text, false, false, SummaryTone::Neutral)
        }
    } else if is_band_profit(label) {
        if value > Decimal::ZERO {
            (format!("+{text}"), false, true, SummaryTone::Gain)
        } else if value < Decimal::ZERO {
            (text, true, false, SummaryTone::Loss)
        } else {
            (text, false, false, SummaryTone::Neutral)
        }
    } else {
        (text, false, false, SummaryTone::Neutral)
    }
}

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
    let from = rows.iter().map(|row| row.date()).min()?;
    let to = rows.iter().map(|row| row.date()).max()?;
    Some(receipt_period_short(&year_month(from)?, &year_month(to)?))
}

fn year_month(date: &str) -> Option<String> {
    date_parts(date).map(|(year, month, _)| format!("{year}/{month:02}"))
}

/// 内訳のラベルは税の基準を明記する。PCの列見出しは変えない。
pub(crate) fn breakdown_label(label: &'static str) -> &'static str {
    if label == DIVIDEND_AMOUNT {
        DIVIDEND_BEFORE_TAX
    } else if label == REALIZED_PROFIT_LOSS {
        REALIZED_BEFORE_TAX
    } else {
        label
    }
}

#[component]
pub fn SummaryStrip(
    tab: ReceiptsTab,
    items: Vec<(&'static str, Decimal, StatTone)>,
    #[prop(optional, into)] period: String,
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
    let to_breakdown = |(label, value, _): (&'static str, Decimal, StatTone)| {
        let (text, negative, positive, _) = band_value(label, value);
        SummaryBreakdownEntry {
            label: breakdown_label(label).to_string(),
            value: text,
            negative,
            positive,
            rate: String::new(),
        }
    };
    let to_sub = |(label, value, _): (&'static str, Decimal, StatTone)| {
        let (text, _, _, tone) = band_value(label, value);
        SummarySubItem {
            label: label.to_string(),
            value: text,
            note: String::new(),
            tone,
            extra_class: String::new(),
        }
    };
    let pc_items: Vec<SummarySubItem> = secondary.clone().into_iter().map(to_sub).collect();
    let breakdown: Vec<SummaryBreakdownEntry> = secondary.into_iter().map(to_breakdown).collect();
    let (primary_label, primary_value, primary_negative, primary_positive) = primary
        .map(|(label, value, tone)| {
            let display = match tab {
                ReceiptsTab::Dividend => RECEIVED_DIVIDEND_AFTER_TAX,
                _ => REALIZED_AFTER_TAX,
            };
            let _ = label;
            (
                display.to_string(),
                format_signed(value, primary_mark),
                tone == StatTone::Loss,
                primary_mark && value > Decimal::ZERO,
            )
        })
        .unwrap_or_default();
    view! {
        <section
            data-testid="receipt-summary-strip"
            role="region"
            aria-label=SUMMARY
            class="summary-band"
        >
            <SummaryBand
                primary_label=primary_label
                primary_value=primary_value
                primary_negative=primary_negative
                primary_positive=primary_positive
                primary_sub=period
                pc_items=pc_items
                breakdown=breakdown
                testid_band="kpi-grid"
                testid_toggle="receipt-summary-details-toggle"
            />
        </section>
    }
}
