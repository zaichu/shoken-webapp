use leptos::prelude::*;

/// ラベルを上・値を下にしたタイル。銘柄画面の保有情報と投資信託の詳細で共有し、
/// 並べ方を画面ごとに作り分けない。`<dl>` の中で使う。
#[component]
pub fn HoldingTiles(children: Children) -> impl IntoView {
    view! {
        <dl class="holding-tiles">{children()}</dl>
    }
}

#[component]
pub fn HoldingTile(
    #[prop(into)] label: String,
    /// 主役の項目(評価額・評価損益)は2列分の幅で大きく出す
    #[prop(optional)]
    wide: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=if wide { "holding-tile holding-tile-wide" } else { "holding-tile" }>
            <dt class="holding-tile-label">{label}</dt>
            <dd class="holding-tile-value">{children()}</dd>
        </div>
    }
}

/// 評価額・評価損益の2枚だけの大きなカード並び。銘柄画面の保有情報で使い、
/// ほかの項目は枠なしの HoldingDetails に置く
#[component]
pub fn HoldingSummary(children: Children) -> impl IntoView {
    view! {
        <dl class="holding-summary">{children()}</dl>
    }
}

#[component]
pub fn HoldingSummaryItem(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <div class="holding-summary-card">
            <dt class="holding-summary-label">{label}</dt>
            <dd class="holding-summary-value">{children()}</dd>
        </div>
    }
}

/// 枠なしの「ラベルと値」の並び。区切りは余白と薄い罫線だけにし、
/// 項目数が端数でも崩れないよう列の結合はしない
#[component]
pub fn HoldingDetails(children: Children) -> impl IntoView {
    view! {
        <dl class="holding-details">{children()}</dl>
    }
}

#[component]
pub fn HoldingDetailItem(#[prop(into)] label: String, children: Children) -> impl IntoView {
    view! {
        <div class="holding-detail-item">
            <dt class="holding-detail-label">{label}</dt>
            <dd class="holding-detail-value">{children()}</dd>
        </div>
    }
}
