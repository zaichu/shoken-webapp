use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels::{LOGIN_HISTORY_SHOW_MORE, SEARCH_AND_SORT, SORT_BY};
use crate::ui::search_controls::{
    ConditionRowEnd, SearchBox, SearchClearButton, SearchCount, SearchTextInput,
};
use leptos::prelude::*;

/// 資産管理・取引明細のスマホ検索欄と並べ替えの入口。
/// 検索の入力欄を常に出し、その下に「並べ替え: …」の1行を出す。
/// 検索欄の指定が無い呼び出し(管理者のログイン履歴)は今の本番のボタンのままにする
#[component]
pub fn MobileSearchEntryShell(
    #[prop(into)] entry_testid: &'static str,
    #[prop(into)] search_label: String,
    #[prop(into, optional)] placeholder: Option<String>,
    #[prop(into, optional)] value: Option<Signal<String>>,
    #[prop(optional)] on_input: Option<Callback<String>>,
    #[prop(optional)] search_input_testid: Option<&'static str>,
    #[prop(optional)] search_hidden_on_mobile: bool,
    sort_summary: Signal<String>,
    open: RwSignal<bool>,
    children: Children,
) -> impl IntoView {
    let search_input = placeholder.zip(value).zip(on_input);
    view! {
        <div class="no-print lg:hidden" role="search" aria-label=search_label>
            {match search_input {
                Some(((placeholder, value), on_input)) => view! {
                    <SearchBox class="mobile-search-box" class:hidden=search_hidden_on_mobile>
                        <SearchTextInput
                            placeholder=placeholder
                            testid=search_input_testid.unwrap_or(entry_testid)
                            value=move || value.get()
                            on_input=move |value| on_input.run(value)
                        />
                    </SearchBox>
                    <Button
                        variant=ButtonVariant::Secondary(ButtonSize::Md)
                        class="mobile-sort-entry".to_string()
                        aria_expanded=Signal::derive(move || open.get())
                        testid=entry_testid
                        on_click=move |_| open.set(true)
                    >
                        <span>{SORT_BY}</span>
                        <span class="mobile-sort-entry-value">{move || sort_summary.get()}</span>
                        <svg aria-hidden="true" class="mobile-sort-entry-chevron" viewBox="0 0 20 20" fill="none" stroke="currentColor" stroke-width="1.8">
                            <path d="m5 7 5 5 5-5"></path>
                        </svg>
                    </Button>
                }
                    .into_any(),
                None => view! {
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
                }
                    .into_any(),
            }}
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
