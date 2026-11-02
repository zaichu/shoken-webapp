use crate::ui::amount::Amount;
use crate::ui::disclosure::{ChevronIcon, DisclosureStyle, DisclosureToggle};
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

/// 集計帯の1項目。ラベルと数字の組。
/// 全項目でラベルの上端と数字の基準線をそろえるために、帯の中で同じ作りを使う
#[component]
pub fn SummaryItem(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(optional)] size: SummarySize,
    /// 数字の右に添える補足(率など)。値より小さいサイズで出す
    #[prop(optional)]
    note: String,
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
    let tone_class = match tone {
        SummaryTone::Neutral => "",
        SummaryTone::Gain => "summary-gain",
        SummaryTone::Loss => "summary-loss",
    };
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
            <dd class=format!("summary-value {size_class} {tone_class}")>
                <Amount text=value />
                {(!note.is_empty())
                    .then(|| view! { <span class="summary-note">{note.clone()}</span> })}
                {(!status.is_empty())
                    .then(|| view! { <span class="summary-status">{status.clone()}</span> })}
            </dd>
        </div>
    }
}

/// PCの帯に残す従項目(スマホの内訳と同じ値・違う並べ方)
#[derive(Clone, Debug, PartialEq)]
pub struct SummarySubItem {
    pub label: String,
    pub value: String,
    pub note: String,
    pub tone: SummaryTone,
    pub extra_class: String,
}

/// スマホの縦リストの1行。ラベル左・金額右・区切り線つき
#[derive(Clone, Debug, PartialEq)]
pub struct SummaryBreakdownEntry {
    pub label: String,
    pub value: String,
    pub negative: bool,
    pub positive: bool,
    pub rate: String,
}

/// 紺帯サマリーの共通形。主役1つ・補助1行・主役行右端の丸いシェブロンで開閉する。
/// 帯全体が開閉ボタンになり、開くと帯の下に縦リストを出す(初期は閉)。
/// PCは従来の横並びを残し、スマホだけ開閉にする(PCの見た目を変えない)。
/// 開閉の状態は画面内だけで持つ。
#[component]
pub fn SummaryBand(
    #[prop(into)] primary_label: String,
    #[prop(into)] primary_value: String,
    #[prop(optional)] primary_negative: bool,
    /// スマホ主値の正色用。PCは変更前の配色(負のみ赤)を保つため使わない
    #[prop(optional)]
    primary_positive: bool,
    /// 主ラベルの下に小さく出す期間(取引明細の帯用。空なら出さない)
    #[prop(optional, into)]
    primary_sub: String,
    /// 補助1行(ホーム・資産管理の評価損益と率。取引明細はなし)
    #[prop(optional, into)]
    aux_label: String,
    #[prop(optional, into)] aux_value: String,
    #[prop(optional, into)] aux_rate: String,
    #[prop(optional)] aux_negative: bool,
    #[prop(optional)] aux_positive: bool,
    /// PCの横並びに残す項目(従来どおり・スマホでは出さない)
    #[prop(optional, into)]
    pc_items: Vec<SummarySubItem>,
    /// スマホの縦リスト(初期は閉)
    #[prop(optional, into)]
    breakdown: Vec<SummaryBreakdownEntry>,
    /// 主ラベルの横に小さく出す基準日(「価格基準日 10/20」。空なら出さない)
    #[prop(optional, into)]
    primary_as_of: String,
    /// 基準日が古い行・更新待ちの行があるときの小さな表示
    #[prop(optional)]
    show_pending: bool,
    #[prop(optional)] testid_band: Option<&'static str>,
    testid_toggle: &'static str,
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let controls_id = format!("{}-breakdown", testid_band.unwrap_or("summary"));
    let as_of_note = {
        let mut text = primary_as_of.clone();
        if show_pending {
            if !text.is_empty() {
                text.push('・');
            }
            text.push_str(crate::ui::labels::PARTIAL_PENDING_UPDATE);
        }
        text
    };
    let has_breakdown = !breakdown.is_empty();
    let has_aux = !aux_label.is_empty();
    let label_note = if !primary_sub.is_empty() && !as_of_note.is_empty() {
        format!("{}・{}", primary_sub, as_of_note)
    } else if !primary_sub.is_empty() {
        primary_sub.clone()
    } else {
        as_of_note.clone()
    };
    let aux_tone_class = if aux_negative {
        "summary-loss"
    } else if aux_positive {
        "summary-gain"
    } else {
        ""
    };
    let primary_tone_class = if primary_negative {
        "summary-loss"
    } else if primary_positive {
        "summary-gain"
    } else {
        ""
    };
    let label_open = "内訳を開く".to_string();
    let label_close = "内訳を閉じる".to_string();
    view! {
        <div
            class="summary-band-grid"
            data-testid=testid_band
        >
            <div class="summary-band-pc max-lg:hidden">
                <dl class="summary-band-group summary-band-group-main">
                    <SummaryItem
                        label=primary_label.clone()
                        value=primary_value.clone()
                        size=SummarySize::Primary
                        label_note=label_note.clone()
                        tone=if primary_negative {
                            SummaryTone::Loss
                        } else {
                            SummaryTone::Neutral
                        }
                    />
                    {has_aux
                        .then(|| view! {
                            <SummaryItem
                                label=aux_label.clone()
                                value=aux_value.clone()
                                note=aux_rate.clone()
                                tone=if aux_negative {
                                    SummaryTone::Loss
                                } else if aux_positive {
                                    SummaryTone::Gain
                                } else {
                                    SummaryTone::Neutral
                                }
                                extra_class="summary-band-item-pnl"
                            />
                        })}
                </dl>
                {(!pc_items.is_empty())
                    .then(|| view! {
                        <div class="summary-band-group summary-band-group-sub">
                            <dl class="summary-band-items">
                                {pc_items
                                    .into_iter()
                                    .map(|item| view! {
                                        <SummaryItem
                                            label=item.label
                                            value=item.value
                                            size=SummarySize::Secondary
                                            note=item.note
                                            tone=item.tone
                                            extra_class=item.extra_class
                                        />
                                    })
                                    .collect_view()}
                            </dl>
                        </div>
                    })}
            </div>
            <div class="summary-band-mobile lg:hidden">
                {has_breakdown
                    .then(|| view! {
                        <DisclosureToggle
                            style=DisclosureStyle::SummaryBand
                            expanded=expanded
                            controls=controls_id.clone()
                            testid=testid_toggle
                            on_toggle=move || expanded.update(|open| *open = !*open)
                        >
                            <span class="sr-only">
                                {move || if expanded.get() {
                                    label_close.clone()
                                } else {
                                    label_open.clone()
                                }}
                            </span>
                            <span class="summary-band-top-row">
                                <span class="summary-band-label">{primary_label.clone()}</span>
                                {(!primary_sub.is_empty() || !as_of_note.is_empty())
                                    .then(|| view! {
                                        <span class="summary-band-date">
                                            {label_note.clone()}
                                        </span>
                                    })}
                            </span>
                            <span class="summary-band-hero-row">
                                <span class=format!(
                                    "summary-band-hero-value {primary_tone_class}",
                                )>
                                    <Amount text=primary_value.clone() />
                                </span>
                                <span class="summary-band-chevron-circle" aria-hidden="true">
                                    <ChevronIcon
                                        expanded=expanded
                                        class="summary-band-toggle-chevron"
                                    />
                                </span>
                            </span>
                            {has_aux
                                .then(|| view! {
                                    <span class=format!(
                                        "summary-band-aux {aux_tone_class}",
                                    )>
                                        <span class="summary-band-aux-label">
                                            {aux_label.clone()}
                                        </span>
                                        <Amount text=aux_value.clone() />
                                        {(!aux_rate.is_empty())
                                            .then(|| view! {
                                                <span class="summary-note">{aux_rate.clone()}</span>
                                            })}
                                    </span>
                                })}
                        </DisclosureToggle>
                        <div id=controls_id.clone() hidden=move || !expanded.get()>
                            <dl class="summary-band-breakdown">
                                {breakdown
                                    .into_iter()
                                    .map(|entry| {
                                        let tone_class = if entry.negative {
                                            "summary-loss"
                                        } else if entry.positive {
                                            "summary-gain"
                                        } else {
                                            ""
                                        };
                                        view! {
                                            <div class="summary-band-breakdown-row">
                                                <dt class="summary-band-breakdown-label">
                                                    {entry.label}
                                                </dt>
                                                <dd class=format!(
                                                    "summary-band-breakdown-value {tone_class}",
                                                )>
                                                    <Amount text=entry.value />
                                                    {(!entry.rate.is_empty())
                                                        .then(|| view! {
                                                            <span class="summary-note">{entry.rate}</span>
                                                        })}
                                                </dd>
                                            </div>
                                        }
                                    })
                                    .collect_view()}
                            </dl>
                        </div>
                    })}
            </div>
        </div>
    }
}
