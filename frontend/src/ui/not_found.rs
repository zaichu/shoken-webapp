//! 404 の本文。ホームへの戻りと関連リンクの並びはこの部品の中で決める。

use leptos::prelude::*;

/// 空状態の下に出す操作のまとまり
#[component]
pub fn NotFoundActions(children: Children) -> impl IntoView {
    view! {
        <div class="mt-4">
            <div class="flex flex-col items-center gap-4">{children()}</div>
        </div>
    }
}

/// 関連リンクの横並び
#[component]
pub fn NotFoundRelatedLinks(children: Children) -> impl IntoView {
    view! { <div class="flex flex-wrap justify-center gap-3">{children()}</div> }
}
