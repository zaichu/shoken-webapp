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
