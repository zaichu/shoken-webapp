use crate::ui::amount::{Amount, AmountKind};
use crate::ui::labels;
use leptos::prelude::*;

#[cfg(test)]
mod tests;

/// 集計の1項目の数字の色
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SummaryTone {
    /// 中立
    #[default]
    Neutral,
    /// プラス
    Gain,
    /// マイナス
    Loss,
}

const fn tone_amount(tone: SummaryTone) -> (AmountKind, bool, bool) {
    match tone {
        SummaryTone::Gain => (AmountKind::Delta, false, true),
        SummaryTone::Loss => (AmountKind::Delta, true, false),
        SummaryTone::Neutral => (AmountKind::Plain, false, false),
    }
}

/// 帯の基準日。更新待ちの行があるときは小さな表示を添える
pub fn with_pending_note(as_of: &str, show_pending: bool) -> String {
    if !show_pending {
        return as_of.to_string();
    }
    if as_of.is_empty() {
        labels::PARTIAL_PENDING_UPDATE.to_string()
    } else {
        format!(
            "{}{}{}",
            as_of,
            labels::TAG_SEPARATOR,
            labels::PARTIAL_PENDING_UPDATE
        )
    }
}

/// 集計の1項目。ラベルと数字の組。スマホの主役の次の行とPCの2項目め以降で同じ値を使う
#[derive(Clone, Debug, PartialEq)]
pub struct SummaryStat {
    pub label: String,
    pub value: String,
    pub note: String,
    pub tone: SummaryTone,
}

/// 数字と補足(率)の組。補足は数字と同じ大きさ・同じ色で出す
fn stat_value(value: String, note: String, tone: SummaryTone) -> impl IntoView {
    let (kind, negative, positive) = tone_amount(tone);
    view! {
        <Amount text=value kind=kind negative=negative positive=positive />
        {(!note.is_empty())
            .then(|| view! {
                <Amount text=note kind=kind negative=negative positive=positive />
            })}
    }
}

/// 集計の1項目。ラベルを上、数字を下に置く
#[component]
pub fn SummaryItem(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional, into)] note: String,
    #[prop(optional)] tone: SummaryTone,
) -> impl IntoView {
    view! {
        <div class="summary-stat">
            <dt class="summary-stat-label">{label}</dt>
            <dd class="summary-stat-value">{stat_value(value, note, tone)}</dd>
        </div>
    }
}

/// 画面上部の集計。面の塗りはなく、地の上に置き、色は文字だけにする
/// (紺=主役の数字、緑/赤=損益、灰=ラベル・補足)。
/// スマホは主役と次の1項目と基準日だけ出し、3項目め以降はPCだけに出す。
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub fn SummaryBand(
    primary: SummaryStat,
    /// スマホの主役の次の行とPCの2項目め。ラベルはPCだけに出す
    #[prop(optional)]
    secondary: Option<SummaryStat>,
    /// PCだけに出す3項目め以降
    #[prop(optional)]
    pc_items: Vec<SummaryStat>,
    /// スマホの主役の下に出す基準日
    #[prop(into)]
    mobile_dates: String,
    /// PCの右端に出す基準日・取り込み日
    #[prop(optional)]
    pc_dates: Vec<String>,
    #[prop(optional)] testid_band: Option<&'static str>,
) -> impl IntoView {
    view! {
        <div class="summary-band-grid" data-testid=testid_band>
            <div class="summary-hero">
                <p class="summary-hero-label">{primary.label.clone()}</p>
                <p class="summary-hero-primary">
                    {stat_value(primary.value.clone(), String::new(), primary.tone)}
                </p>
                {secondary
                    .clone()
                    .map(|stat| view! {
                        <p class="summary-hero-secondary">
                            {stat_value(stat.value, stat.note, stat.tone)}
                        </p>
                    })}
                {(!mobile_dates.is_empty())
                    .then(|| view! {
                        <p class="summary-hero-dates">{mobile_dates.clone()}</p>
                    })}
            </div>
            <div class="summary-stats">
                <dl class="summary-stats-group">
                    <SummaryItem
                        label=primary.label
                        value=primary.value
                        tone=primary.tone
                    />
                    {secondary
                        .map(|stat| view! {
                            <SummaryItem
                                label=stat.label
                                value=stat.value
                                note=stat.note
                                tone=stat.tone
                            />
                        })}
                    {pc_items
                        .into_iter()
                        .map(|item| view! {
                            <SummaryItem
                                label=item.label
                                value=item.value
                                note=item.note
                                tone=item.tone
                            />
                        })
                        .collect_view()}
                </dl>
                {(!pc_dates.is_empty())
                    .then(|| view! {
                        <div class="summary-dates">
                            {pc_dates
                                .into_iter()
                                .map(|line| view! { <p>{line}</p> })
                                .collect_view()}
                        </div>
                    })}
            </div>
        </div>
    }
}
