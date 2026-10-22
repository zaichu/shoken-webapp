use crate::ui::site::{AccountMenu, NAV_ITEMS, NavLink};
use leptos::prelude::*;

/// PC 用ヘッダー: ロゴ・グローバルナビ・アカウントメニュー
#[component]
fn PcHeader() -> impl IntoView {
    view! {
        <header class="pc-header no-print hidden lg:flex">
            <div class="page-gutter flex items-center gap-8 h-14">
                <a href="/" class="flex items-center gap-2.5 no-underline" aria-label="証券Web トップ">
                    <div class="header-logo" aria-hidden="true">
                        <svg width="28" height="28" viewBox="0 0 28 28" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
                            <rect width="28" height="28" rx="7" fill="#FFFFFF"/>
                            <rect x="7" y="17" width="4" height="6" rx="1" fill="#F2B33D"/>
                            <rect x="12" y="12" width="4" height="11" rx="1" fill="#F2B33D"/>
                            <rect x="17" y="7" width="4" height="16" rx="1" fill="#F2B33D"/>
                        </svg>
                    </div>
                    <span class="text-base font-bold text-text-inverse">証券Web</span>
                </a>
                <nav class="flex gap-1 h-14" aria-label="メインナビゲーション">
                    {NAV_ITEMS.iter().map(|(to, label)| {
                        view! { <NavLink to=*to label=*label class="pc-nav-link" /> }
                    }).collect_view()}
                </nav>
                <div class="ml-auto flex items-center gap-2">
                    <AccountMenu />
                </div>
            </div>
        </header>
    }
}

/// 全ページ共通の型(ページ名・戻る・本文・下部操作帯)。下部ナビは App が描く。
/// ページ名は幅によらず 1 つだけ出し、PC はヘッダーの下・モバイルは各ページの内容の直前に出る
#[component]
pub fn PageFrame(
    /// ページ名(h1)
    #[prop(into)]
    title: String,
    /// 戻る文字リンク("[資産管理に戻る]" など)。href とラベルはセットで指定する
    #[prop(into, optional)]
    back_href: Option<String>,
    #[prop(into, optional)] back_label: Option<String>,
    /// ページ名の右に置く低頻度操作への文字リンク([CSV管理] など)
    #[prop(optional)]
    links: Option<Children>,
    /// ページ名の横に出す件数(「15件」)
    #[prop(optional)]
    title_count: Option<Signal<String>>,
    /// 画面下部に常設する操作帯(検索欄・適用・保存など)
    #[prop(optional)]
    action_bar: Option<AnyView>,
    #[prop(optional)] action_bar_mobile_only: bool,
    /// true のとき PC は操作帯を本文の先頭に置いて上部に張り付かせる
    /// (スマホは下のまま。CSV 管理のように保存ボタンを常に届かせる頁用)
    #[prop(optional)]
    action_bar_pc_top: bool,
    /// true のとき本文の余白を各部品に任せる(ワークスペース系ページ)
    #[prop(optional)]
    full_bleed: bool,
    /// true のときスマホでも頁枠を画面高まで縮める。本文が内側スクロールを
    /// 持つ頁専用(縮めないとスクロール領域が確定しない)
    #[prop(optional)]
    mobile_inner_scroll: bool,
    children: Children,
) -> impl IntoView {
    let body = children();
    let has_action_bar = action_bar.is_some();

    view! {
        <div
            class="page-frame"
            class:has-action-bar=has_action_bar
            class:inner-scroll=mobile_inner_scroll
        >
            <PcHeader />
            <header class="page-frame-head no-print">
                <div class="page-gutter flex flex-wrap items-center gap-x-4 gap-y-2">
                    <h1 class="page-frame-title order-1 shrink-0">{title}</h1>
                    {move || {
                        title_count.map(|count| {
                            view! {
                                <span class="page-frame-count order-1" data-testid="page-title-count">
                                    {count}
                                </span>
                            }
                        })
                    }}
                    <div class="order-2 ml-auto flex shrink-0 items-center gap-3 lg:order-3">
                        {links.map(|links| links())}
                        <div class="lg:hidden">
                            <AccountMenu hide_name=true />
                        </div>
                    </div>
                </div>
            </header>
            {back_href
                .zip(back_label)
                .map(|(href, label)| {
                    view! {
                        <div class="page-gutter">
                            <a class="page-back-link" href=href>
                                {label}
                            </a>
                        </div>
                    }
                })}
            {if full_bleed {
                // lg 以上では高さを内容に任せる。basis 0 のままだとページ枠が画面高で
                // 打ち切られ、sticky な上部ヘッダー・表見出しの固定範囲が失われる
                view! { <div class="flex min-h-0 flex-1 flex-col lg:flex-none">{body}</div> }.into_any()
            } else {
                let body_class = if action_bar_pc_top {
                    // 操作帯は DOM の後ろにあるため、order を付けて PC だけ本文の前に出す
                    "page-gutter flex flex-1 flex-col py-4 lg:order-2"
                } else {
                    "page-gutter flex flex-1 flex-col py-4"
                };
                view! { <div class=body_class>{body}</div> }.into_any()
            }}
            {action_bar.map(|bar| {
                let class = if action_bar_pc_top {
                    // 上部ヘッダー(h-14)の下に張り付かせる。mt-auto・下端張り付きは PC では解除する
                    "page-action-bar no-print lg:order-1 lg:top-14 lg:bottom-auto lg:z-30 lg:mt-0 lg:border-b lg:border-t-0"
                } else if action_bar_mobile_only {
                    "page-action-bar no-print lg:hidden"
                } else {
                    "page-action-bar no-print"
                };
                view! {
                    <div class=class>
                        <div class="page-gutter">{bar}</div>
                    </div>
                }
            })}
        </div>
    }
}
