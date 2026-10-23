//! ページの骨格(見出し行・タブ行・帯・注記行・検索と条件の行)。
//! 資産管理と取引明細は同じ部品を同じ順序で使い、余白は共通のトークンで決める

use crate::ui::labels;
use leptos::prelude::*;

/// 絞り込み中の件数。注記の行に出す
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FilteredCount {
    /// 絞り込み後の件数
    pub filtered: usize,
    /// 絞り込み前の件数
    pub total: usize,
    /// 絞り込みを使っているか。使うときだけ注記に「絞り込み中」を出す
    pub filtering: bool,
}

/// タブの行。見出し行の直下に置く
#[component]
pub fn SkeletonTabs(children: Children) -> impl IntoView {
    view! {
        <div class="page-skeleton-tabs no-print">{children()}</div>
    }
}

/// 集計の帯。タブ行と注記行の間
#[component]
pub fn SkeletonSummary(children: Children) -> impl IntoView {
    view! {
        <div class="page-skeleton-summary">{children()}</div>
    }
}

/// 注記の行。帯の直下に置く。注記の右端に「計算基準を見る」を置く
#[component]
pub fn SkeletonNote(
    /// 注記の文
    #[prop(into)]
    notes: Vec<String>,
    /// 絞り込み中の表示。条件が無いときは出さない
    #[prop(optional)]
    filtered: Option<Signal<FilteredCount>>,
    /// 「計算基準を見る」で出す説明
    #[prop(into, optional)]
    basis: Vec<String>,
) -> impl IntoView {
    view! {
        <div class="page-skeleton-note">
            <div class="page-skeleton-note-body">
                {notes
                    .into_iter()
                    .filter(|line| !line.is_empty())
                    .map(|line| view! { <p>{line}</p> })
                    .collect_view()}
                {move || {
                    filtered
                        .as_ref()
                        .map(|signal| signal.get())
                        .filter(|count| count.filtering)
                        .map(|count| {
                            let text = labels::filtering_count(count.filtered, count.total);
                            view! { <p data-testid="page-filtering-count">{text}</p> }
                        })
                }}
            </div>
            <details class="page-skeleton-basis">
                <summary>"計算基準を見る"</summary>
                <div class="page-skeleton-basis-body">
                    {basis
                        .into_iter()
                        .map(|line| view! { <p>{line}</p> })
                        .collect_view()}
                </div>
            </details>
        </div>
    }
}

/// 検索欄の行。検索欄と条件の行を1つの段としてまとめ、その下に表を置く
#[component]
pub fn SkeletonSearch(children: Children) -> impl IntoView {
    view! {
        <div class="page-skeleton-search no-print">{children()}</div>
    }
}
