use crate::features::receipts::{ReceiptRow, ReceiptTabData, ReceiptsTab, select_header_summary};
use crate::settings::format_amount;
use crate::ui::card::StatTone;
use crate::ui::format::format_iso_date;
use crate::ui::labels;
use crate::ui::summary::{SummaryItem, SummarySize, SummaryTone};
use leptos::prelude::*;
use rust_decimal::Decimal;

pub fn profit_tone(value: Decimal) -> StatTone {
    if value < Decimal::ZERO {
        StatTone::Loss
    } else {
        StatTone::Neutral
    }
}

/// 集計帯の数字の色
fn summary_tone(tone: StatTone) -> SummaryTone {
    match tone {
        StatTone::Loss => SummaryTone::Loss,
        StatTone::Neutral => SummaryTone::Neutral,
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
pub fn SummaryStrip(items: Vec<(&'static str, Decimal, StatTone)>) -> impl IntoView {
    // 見本では主値(税引後)を金の見出しで先頭に大きく出す。items の末尾が主値
    let mut secondary = items;
    let primary = secondary.pop();
    view! {
        <section
            data-testid="receipt-summary-strip"
            role="region"
            aria-label="集計"
            class="summary-band"
        >
            <div class="summary-band-grid" data-testid="kpi-grid">
                <dl class="summary-band-group summary-band-group-main">
                    {primary.map(|(label, value, tone)| {
                        view! {
                            <SummaryItem
                                label=label
                                value=format_amount(value)
                                size=SummarySize::Primary
                                tone=summary_tone(tone)
                            />
                        }
                    })}
                </dl>
                <div class="summary-band-group summary-band-group-sub">
                    <dl class="summary-band-items">
                        {secondary
                            .into_iter()
                            .map(|(label, value, tone)| {
                                view! {
                                    <SummaryItem
                                        label=label
                                        value=format_amount(value)
                                        size=SummarySize::Secondary
                                        tone=summary_tone(tone)
                                    />
                                }
                            })
                            .collect_view()}
                    </dl>
                </div>
            </div>
        </section>
    }
}
