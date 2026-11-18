//! 取得・取り込みの確認と案内。種類ごとの進行・結果は CSV 管理の表に出す。

use crate::ui::state::{Alert, AlertVariant};
use leptos::prelude::*;

/// 確認と案内の1件ぶんの枠
#[component]
pub fn CsvSyncNotice(
    variant: AlertVariant,
    #[prop(optional)] testid: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="mt-2" data-testid=testid>
            <Alert variant=variant>{children()}</Alert>
        </div>
    }
}

/// 取り込み確認の操作の並び
#[component]
pub fn CsvSyncConfirmActions(children: Children) -> impl IntoView {
    view! { <div class="mt-2 flex gap-2">{children()}</div> }
}

/// 案内の補足文
#[component]
pub fn CsvSyncNote(children: Children) -> impl IntoView {
    view! { <p class="mt-1 font-normal">{children()}</p> }
}

/// ファイル選択の案内の手順
#[component]
pub fn CsvSyncSteps(children: Children) -> impl IntoView {
    view! { <ol class="mt-1 list-decimal pl-5 font-normal">{children()}</ol> }
}
