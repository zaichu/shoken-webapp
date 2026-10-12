use crate::ui::site::{AccountMenu, NAV_ITEMS, NavLink};
use leptos::prelude::*;

/// PC 用ヘッダー: ロゴ・グローバルナビ・アカウントメニュー
#[component]
fn PcHeader() -> impl IntoView {
    view! {
        <header class="pc-header no-print hidden lg:flex">
            <div class="page-gutter flex items-center gap-8 h-14">
                <a href="/" class="flex items-center gap-2 no-underline" aria-label="証券Web トップ">
                    <div class="header-logo" aria-hidden="true">
                        <svg width="26" height="26" viewBox="0 0 26 26" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
                            <rect x="1" y="1" width="24" height="24" rx="6" fill="#0F2A4A"/>
                            <rect x="6" y="14" width="4" height="6" rx="1" fill="#F2B33D"/>
                            <rect x="11" y="10" width="4" height="10" rx="1" fill="#F2B33D"/>
                            <rect x="16" y="6" width="4" height="14" rx="1" fill="#F2B33D"/>
                        </svg>
                    </div>
                    <span class="text-base font-bold text-ink">証券Web</span>
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
    /// ページ名の隣主要内容(取引明細の種別タブなど)。スマホではページ名の下に回り込む
    #[prop(optional)]
    title_row: Option<Children>,
    /// 画面下部に常設する操作帯(検索欄・適用・保存など)
    #[prop(optional)]
    action_bar: Option<AnyView>,
    #[prop(optional)] action_bar_mobile_only: bool,
    /// true のとき本文の余白を各部品に任せる(ワークスペース系ページ)
    #[prop(optional)]
    full_bleed: bool,
    children: Children,
) -> impl IntoView {
    let body = children();

    view! {
        <div class="page-frame">
            <PcHeader />
            <header class="page-frame-head no-print">
                <div class="page-gutter flex flex-wrap items-center gap-x-4 gap-y-1">
                    <h1 class="page-frame-title order-1 shrink-0">{title}</h1>
                    <div class="order-3 w-full min-w-0 lg:order-2 lg:w-auto">
                        {title_row.map(|row| row())}
                    </div>
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
                view! { <div class="flex min-h-0 flex-1 flex-col">{body}</div> }.into_any()
            } else {
                view! { <div class="page-gutter flex flex-1 flex-col py-4">{body}</div> }.into_any()
            }}
            {action_bar.map(|bar| {
                let class = if action_bar_mobile_only {
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
