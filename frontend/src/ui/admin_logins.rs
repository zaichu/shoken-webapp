//! 管理者のログイン一覧の配置。余白はこの部品の中で決め、features/ 側には置かない。

use crate::ui::labels;
use crate::ui::state::ListLoadError;
use leptos::prelude::*;

/// スマホの検索ダイアログの中身の縦並び
#[component]
pub fn AdminDialogStack(children: Children) -> impl IntoView {
    view! { <div class="flex flex-col gap-4">{children()}</div> }
}

/// ダイアログ内の項目名と入力欄の組み。文言と入力欄はこの順で出す
#[component]
pub fn AdminDialogField(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <label class="mb-1 block text-label font-bold text-text-deep">
            {label}
            {children()}
        </label>
    }
}

/// PC の検索欄の置き場所(小窓の座標の基準にする)
#[component]
pub fn AdminPcSearchArea(children: Children) -> impl IntoView {
    view! {
        <div class="hidden lg:block">
            <div class="pc-search-area mb-4" role="search" aria-label=labels::ADMIN_LOGIN_SEARCH>
                {children()}
            </div>
        </div>
    }
}

/// 一覧の続きの読み込みに失敗したときの行
#[component]
pub fn AdminListMoreError(
    #[prop(into)] message: String,
    on_retry: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="mt-4">
            <ListLoadError message=message on_retry=on_retry />
        </div>
    }
}

/// 表の上の凡例の1行。設定の SettingsNote と同じ文字の見た目にする
/// (管理者は設定の部品を使わず、この画面の部品に寄せる)
#[component]
pub fn AdminTableNote(children: Children) -> impl IntoView {
    view! { <p class="pb-3 text-cell text-text-muted">{children()}</p> }
}
