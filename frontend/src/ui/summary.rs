use crate::ui::amount::Amount;
use leptos::prelude::*;

/// 集計帯の数字の大きさ。主は1つだけ、従はすべて同じ大きさにする
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SummarySize {
    /// 主
    Primary,
    /// 従
    #[default]
    Secondary,
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
            <dt class="summary-label">{label}</dt>
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
