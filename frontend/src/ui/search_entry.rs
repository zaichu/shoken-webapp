use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels::{LOGIN_HISTORY_SHOW_MORE, SEARCH_AND_SORT};
use crate::ui::search_controls::{ConditionRowEnd, SearchClearButton, SearchCount};
use leptos::prelude::*;

/// スマホの「検索・並べ替え」入口の器。ボタンと条件タグ・件数の段を1つにまとめる。
/// 資産管理・取引明細・管理者ログインで同じ見た目にする。
/// 中身のタグは子要素に書き、押した先のダイアログは呼び出し側でこの後に置く
#[component]
pub fn MobileSearchEntryShell(
    #[prop(into)] entry_testid: &'static str,
    #[prop(into)] search_label: String,
    sort_summary: Signal<String>,
    open: RwSignal<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="no-print lg:hidden" role="search" aria-label=search_label>
            <Button
                variant=ButtonVariant::Secondary(ButtonSize::Md)
                class="search-sort-entry".to_string()
                aria_expanded=Signal::derive(move || open.get())
                testid=entry_testid
                on_click=move |_| open.set(true)
            >
                <span class="search-sort-entry-title">
                    <svg aria-hidden="true" class="h-4 w-4 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="11" cy="11" r="7"></circle>
                        <path d="M20 20l-3.5-3.5"></path>
                    </svg>
                    {SEARCH_AND_SORT}
                </span>
                <span class="search-sort-entry-sub">{move || sort_summary.get()}</span>
            </Button>
            <div class="search-sort-entry-tags">{children()}</div>
        </div>
    }
}

/// PC の条件の行の右端。件数と「すべてクリア」を1つの段にまとめる
#[component]
pub fn SearchCountAndClear(
    filtered: impl Fn() -> usize + Copy + 'static + Send + Sync,
    total: impl Fn() -> usize + Copy + 'static + Send + Sync,
    clear_disabled: impl Fn() -> bool + Copy + 'static + Send + Sync,
    on_clear: impl Fn() + Copy + 'static + Send + Sync,
    #[prop(optional)] testid: Option<&'static str>,
    #[prop(optional)] clear_testid: Option<&'static str>,
) -> impl IntoView {
    let testid = testid.unwrap_or("");
    let clear_testid = clear_testid.unwrap_or("");
    view! {
        <ConditionRowEnd>
            <SearchCount filtered=filtered total=total testid=testid />
            <SearchClearButton disabled=clear_disabled testid=clear_testid on_clear=on_clear />
        </ConditionRowEnd>
    }
}

/// 一覧の末尾の「もっと見る」。設定のログイン履歴と管理者の一覧で同じ文言・同じ配置にする
#[component]
pub fn ShowMoreButton(
    #[prop(into)] testid: &'static str,
    disabled: Signal<bool>,
    on_click: impl Fn() + Copy + 'static + Send + Sync,
) -> impl IntoView {
    view! {
        <div class="mt-4 flex justify-center">
            <Button
                variant=ButtonVariant::Secondary(ButtonSize::Sm)
                testid=testid
                disabled=disabled
                on_click=move |_| on_click()
            >
                {LOGIN_HISTORY_SHOW_MORE}
            </Button>
        </div>
    }
}
