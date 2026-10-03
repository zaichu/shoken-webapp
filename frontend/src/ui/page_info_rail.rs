use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use leptos::prelude::*;

/// 右レール(ページ情報)。「このページの内容」のページ内リンクと
/// 「適用中の条件」(適用中の絞り込みと解除操作)を描く。
/// 取引明細・資産管理の両ワークスペースで共通の見た目を揃えるための部品。
#[component]
pub fn PageInfoRail(
    /// 「このページの内容」のアンカーリンク (href, ラベル)
    links: Vec<(&'static str, &'static str)>,
    /// 「適用中の条件」ブロック (ラベル, 値)。None ならブロック自体を出さない
    #[prop(optional_no_strip)]
    applied: Option<Vec<(&'static str, Signal<String>)>>,
    /// 解除ボタンの動作。applied があるときだけ描画される
    #[prop(optional_no_strip)]
    on_clear: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <nav class="prail-toc" aria-label="このページの内容">
            <p class="prail-title">"このページの内容"</p>
            {links
                .into_iter()
                .map(|(href, label)| view! { <a href=href>{label}</a> })
                .collect_view()}
        </nav>
        {applied.map(|items| {
            view! {
                <div class="prail-applied">
                    <p class="prail-title">"適用中の条件"</p>
                    {items
                        .into_iter()
                        .map(|(label, value)| {
                            view! {
                                <div class="prail-applied-row">
                                    <span>{label}</span>
                                    <b>{move || value.get()}</b>
                                </div>
                            }
                        })
                        .collect_view()}
                    {on_clear.map(|clear| {
                        view! {
                            <Button
                                variant=ButtonVariant::SecondarySoft(ButtonSize::Xs)
                                class="prail-clear"
                                on_click=move |_| clear.run(())
                            >
                                "解除"
                            </Button>
                        }
                    })}
                </div>
            }
        })}
    }
}
