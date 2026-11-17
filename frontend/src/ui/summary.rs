use crate::ui::amount::{Amount, AmountKind};
use crate::ui::labels;
use leptos::prelude::*;

#[cfg(test)]
mod tests;

/// 集計帯の数字の大きさ。主は1つだけ、従はすべて同じ大きさにする
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SummarySize {
    /// 主
    Primary,
    /// 従
    #[default]
    Secondary,
}

/// 前日比の率。前日時点の評価額(当日評価額−前日比)を分母にする。
/// 分母が無い・0以下・欠測のときは None(「—」表示にするため空にしない)。
pub fn daily_rate(market: Option<f64>, daily: Option<f64>) -> Option<f64> {
    match (market, daily) {
        (Some(market), Some(daily)) if market - daily > 0.0 => {
            Some(daily / (market - daily) * 100.0)
        }
        _ => None,
    }
}

/// 集計帯の数字の色
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

const fn tone_of(negative: bool, positive: bool) -> SummaryTone {
    if negative {
        SummaryTone::Loss
    } else if positive {
        SummaryTone::Gain
    } else {
        SummaryTone::Neutral
    }
}

/// 集計帯の1項目。ラベルと数字の組。
/// 全項目でラベルの上端と数字の基準線をそろえるために、帯の中で同じ作りを使う
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Leptos component props are owned by the view"
)]
pub fn SummaryItem(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional)] size: SummarySize,
    /// 数字の右に添える補足(率など)。値より小さいサイズで、値と同じ正負の色で出す
    #[prop(optional)]
    note: String,
    #[prop(optional)] note_negative: bool,
    #[prop(optional)] note_positive: bool,
    /// 数字の横に小さく出す補足(読み込み中など)
    #[prop(optional)]
    status: String,
    /// ラベルの横に小さく出す補足(基準日・更新待ちなど)。空なら出さない
    #[prop(optional, into)]
    label_note: String,
    #[prop(optional)] tone: SummaryTone,
    /// 項目ごとに付けたいクラス
    #[prop(optional, into)]
    extra_class: String,
) -> impl IntoView {
    let (kind, negative, positive) = tone_amount(tone);
    let size_class = match size {
        SummarySize::Primary => "summary-value-primary",
        SummarySize::Secondary => "summary-value-secondary",
    };
    view! {
        <div class=format!("summary-item {extra_class}")>
            <dt class="summary-label">
                {label}
                {(!label_note.is_empty())
                    .then(|| view! { <span class="summary-label-note">{label_note.clone()}</span> })}
            </dt>
            <dd class=format!("summary-value {size_class}")>
                <Amount text=value kind=kind negative=negative positive=positive />
                {(!note.is_empty())
                    .then(|| view! {
                        <span class="summary-note">
                            <Amount text=note.clone() negative=note_negative positive=note_positive />
                        </span>
                    })}
                {(!status.is_empty())
                    .then(|| view! { <span class="summary-status">{status.clone()}</span> })}
            </dd>
        </div>
    }
}

/// PCの帯に残す従項目
#[derive(Clone, Debug, PartialEq)]
pub struct SummarySubItem {
    pub label: String,
    pub value: String,
    pub note: String,
    pub tone: SummaryTone,
    pub note_negative: bool,
    pub note_positive: bool,
    pub extra_class: String,
}

/// スマホの行に出す従項目
#[derive(Clone, Debug, PartialEq)]
pub struct SummaryBreakdownEntry {
    pub label: String,
    pub value: String,
    pub negative: bool,
    pub positive: bool,
    pub rate: String,
    pub rate_negative: bool,
    pub rate_positive: bool,
}

/// 集計帯の共通形。主役1つ・補助1行・従項目。
/// PCは縦線区切りの横並び、スマホはラベル左・数字右端の行を全項目常時表示する。
#[component]
pub fn SummaryBand(
    #[prop(into)] primary_label: String,
    #[prop(into)] primary_value: String,
    #[prop(optional)] primary_negative: bool,
    #[prop(optional)] primary_positive: bool,
    /// 主ラベルの下に小さく出す期間(取引明細の帯用。空なら出さない)
    #[prop(optional, into)]
    primary_sub: String,
    /// 補助1行(ホーム・資産管理の評価損益と率。取引明細はなし)
    #[prop(optional, into)]
    aux_label: String,
    #[prop(optional, into)] aux_value: String,
    #[prop(optional, into)] aux_rate: String,
    #[prop(optional)] aux_rate_negative: bool,
    #[prop(optional)] aux_rate_positive: bool,
    #[prop(optional)] aux_negative: bool,
    #[prop(optional)] aux_positive: bool,
    /// PCの横並びに出す従項目(スマホでは breakdown を出す)
    #[prop(optional, into)]
    pc_items: Vec<SummarySubItem>,
    /// スマホの行に出す従項目
    #[prop(optional, into)]
    breakdown: Vec<SummaryBreakdownEntry>,
    /// 主ラベルの横に小さく出す基準日(「価格基準日 10/20」。空なら出さない)
    #[prop(optional, into)]
    primary_as_of: String,
    /// 基準日が古い行・更新待ちの行があるときの小さな表示
    #[prop(optional)]
    show_pending: bool,
    #[prop(optional)] testid_band: Option<&'static str>,
) -> impl IntoView {
    let has_aux = !aux_label.is_empty();
    let as_of_note = {
        let mut text = primary_as_of;
        if show_pending {
            if !text.is_empty() {
                text.push('・');
            }
            text.push_str(labels::PARTIAL_PENDING_UPDATE);
        }
        text
    };
    let label_note = if !primary_sub.is_empty() && !as_of_note.is_empty() {
        format!("{}{}{}", primary_sub, labels::TAG_SEPARATOR, as_of_note)
    } else if !primary_sub.is_empty() {
        primary_sub
    } else {
        as_of_note
    };
    let primary_tone = tone_of(primary_negative, primary_positive);
    let aux_tone = tone_of(aux_negative, aux_positive);
    view! {
        <div class="summary-band-grid" data-testid=testid_band>
            <div class="summary-band-pc max-lg:hidden">
                <dl class="summary-band-items">
                    <SummaryItem
                        label=primary_label.clone()
                        value=primary_value.clone()
                        size=SummarySize::Primary
                        label_note=label_note.clone()
                        tone=primary_tone
                    />
                    {has_aux.then(|| view! {
                        <SummaryItem
                            label=aux_label.clone()
                            value=aux_value.clone()
                            note=aux_rate.clone()
                            note_negative=aux_rate_negative
                            note_positive=aux_rate_positive
                            tone=aux_tone
                        />
                    })}
                    {pc_items
                        .into_iter()
                        .map(|item| view! {
                            <SummaryItem
                                label=item.label
                                value=item.value
                                size=SummarySize::Secondary
                                note=item.note
                                note_negative=item.note_negative
                                note_positive=item.note_positive
                                tone=item.tone
                                extra_class=item.extra_class
                            />
                        })
                        .collect_view()}
                </dl>
            </div>
            <dl class="summary-band-rows lg:hidden">
                <SummaryItem
                    label=primary_label
                    value=primary_value
                    size=SummarySize::Primary
                    label_note=label_note
                    tone=primary_tone
                />
                {has_aux.then(|| view! {
                    <SummaryItem
                        label=aux_label
                        value=aux_value
                        note=aux_rate
                        note_negative=aux_rate_negative
                        note_positive=aux_rate_positive
                        tone=aux_tone
                    />
                })}
                {breakdown
                    .into_iter()
                    .map(|entry| view! {
                        <SummaryItem
                            label=entry.label
                            value=entry.value
                            size=SummarySize::Secondary
                            note=entry.rate
                            note_negative=entry.rate_negative
                            note_positive=entry.rate_positive
                            tone=tone_of(entry.negative, entry.positive)
                        />
                    })
                    .collect_view()}
            </dl>
        </div>
    }
}
