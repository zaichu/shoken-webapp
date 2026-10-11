use crate::ui::site::AccountMenu;
use leptos::prelude::*;

/// 全ページ共通の型(見出し・戻る・本文・下部操作帯)。下部ナビは App が描く。
/// 操作帯はスクロール下端に張り付く sticky で、スクロール末尾では本文の後ろに付く。
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
    /// 画面下部に常設する操作帯(検索欄・適用・保存など)
    #[prop(optional)]
    action_bar: Option<AnyView>,
    /// true のとき本文の余白を各部品に任せる(ワークスペース系ページ)
    #[prop(optional)]
    full_bleed: bool,
    children: Children,
) -> impl IntoView {
    let body = children();
    view! {
        <div class="page-frame">
            <header class="page-frame-head">
                <div class="page-gutter flex flex-wrap items-center justify-between gap-x-4 gap-y-1">
                    <div class="flex min-w-0 flex-wrap items-baseline gap-x-3 gap-y-1">
                        <h1 class="page-frame-title">{title}</h1>
                        {links.map(|links| links())}
                    </div>
                    <AccountMenu />
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
                view! { <div class="flex flex-1 flex-col">{body}</div> }.into_any()
            } else {
                view! { <div class="page-gutter flex flex-1 flex-col py-4">{body}</div> }.into_any()
            }}
            {action_bar.map(|bar| {
                view! {
                    <div class="page-action-bar no-print">
                        <div class="page-gutter">{bar}</div>
                    </div>
                }
            })}
        </div>
    }
}
