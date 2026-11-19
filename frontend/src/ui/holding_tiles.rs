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

#[component]
pub fn HoldingDetails(children: Children) -> impl IntoView {
    view! {
        <dl class="holding-details">{children()}</dl>
    }
}

#[component]
pub fn HoldingDetailItem(
    #[prop(into)] label: String,
    /// 2列(スマホ)で末尾が1つ残るとき全幅で出す
    #[prop(optional)]
    wide: bool,
    /// 4列(PC)でも全幅で出す(2列と別に要るときだけ wide と合わせて付ける)
    #[prop(optional)]
    wide_lg: bool,
    /// 算出条件などの補足(ホバー・読み上げの説明用。空なら出さない)
    #[prop(into, optional)]
    title: Option<String>,
    children: Children,
) -> impl IntoView {
    let class = if wide_lg {
        "holding-detail-item holding-detail-item-wide"
    } else if wide {
        "holding-detail-item holding-detail-item-wide-sm"
    } else {
        "holding-detail-item"
    };
    view! {
        <div class=class title=title>
            <dt class="holding-detail-label">{label}</dt>
            <dd class="holding-detail-value">{children()}</dd>
        </div>
    }
}
