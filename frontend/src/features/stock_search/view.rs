use super::use_stock_search;
use crate::api::dto::Stock;
use crate::ui::badge::CodeBadge;
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use crate::ui::labels;
use crate::ui::page_frame::PageFrame;
use crate::ui::search_controls::{SearchBox, SearchSubmitButton, SearchTextInput};
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;
use shared::normalize::normalize_display_name;

const STOCK_LINKS: &[(&str, &str)] = &[
    (
        labels::STOCK_LINK_RAKUTEN,
        "https://www.rakuten-sec.co.jp/web/market/search/quote.html?ric={code}.T",
    ),
    (
        labels::STOCK_LINK_SBI,
        "https://site3.sbisec.co.jp/ETGate/?_ControlID=WPLETsiR001Control&_DataStoreID=DSWPLETsiR001Control&_PageID=WPLETsiR001Ilst10&_ActionID=getDetailOfStockPriceJP&s_rkbn=1&i_stock_sec=%94%43%93%56%93%B0&i_dom_flg=1&i_exchange_code=JPN&i_output_type=0&stock_sec_code_mul={code}",
    ),
    (
        labels::STOCK_LINK_KABUTAN,
        "https://kabutan.jp/stock/?code={code}",
    ),
    ("Yahoo! Finance", "https://finance.yahoo.co.jp/quote/{code}"),
    (
        labels::STOCK_LINK_NIKKEI,
        "https://www.nikkei.com/nkd/company/?scode={code}",
    ),
    (
        labels::STOCK_LINK_BUFFETT,
        "https://www.buffett-code.com/company/{code}",
    ),
    (
        labels::STOCK_LINK_MINKABU,
        "https://minkabu.jp/stock/{code}/",
    ),
    ("IR BANK", "https://irbank.net/{code}"),
    (
        labels::STOCK_LINK_SCOUTER,
        "https://monex.ifis.co.jp/index.php?sa=report_index&bcode={code}",
    ),
    (
        labels::STOCK_LINK_ZAIMANI,
        "https://zaimani.com/search/?_sf_s={code}",
    ),
    ("JPX Explorer", "https://jpx-explorer.com/ja-JP/{code}-TSE"),
];

#[component]
pub(crate) fn StockSearchPage() -> impl IntoView {
    let stock_search = use_stock_search();
    let loading = stock_search.search.pending();
    let has_invalid = stock_search.has_invalid_code_param;

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !loading.get_untracked() && !stock_search.stock_code.get_untracked().is_empty() {
            stock_search
                .search
                .dispatch(stock_search.stock_code.get_untracked());
        }
    };

    // 検索欄はスマホが下部操作帯、PC が本文の直上。どちらも同じ入力を共有する
    let action_bar = view! {
        <div class="py-2" role="search" aria-label=labels::STOCK_SEARCH_TITLE>
            <SearchForm
                stock_code=stock_search.stock_code
                loading=loading.into()
                icon_size="h-5 w-5"
                on_submit=on_submit
            />
        </div>
    }
    .into_any();

    view! {
        <PageFrame title=labels::STOCK_SEARCH_TITLE action_bar=action_bar action_bar_mobile_only=true>
            <div class="hidden lg:block lg:pb-4" role="search" aria-label=labels::STOCK_SEARCH_TITLE>
                <SearchForm
                    stock_code=stock_search.stock_code
                    loading=loading.into()
                    on_submit=on_submit
                />
            </div>
            <Show when=move || has_invalid>
                <Alert variant=AlertVariant::Warning>{labels::STOCK_SEARCH_INVALID}</Alert>
            </Show>
            {move || {
                let data = stock_search.stock_data();
                let error = stock_search.error_message();
                let not_found = stock_search.is_not_found();
                let is_loading = loading.get();
                if let Some(message) = error {
                    view! {
                        <ListLoadError message=message on_retry=move || {
                            let code = stock_search.stock_code.get_untracked();
                            if !code.is_empty() && !stock_search.search.pending().get_untracked() {
                                stock_search.search.dispatch(code);
                            }
                        } />
                    }
                        .into_any()
                } else if let Some(stock) = data {
                    view! { <StockInfo stock=stock /> }.into_any()
                } else if not_found {
                    view! {
                        <EmptySearch title=labels::STOCK_SEARCH_NOT_FOUND_TITLE />
                    }
                        .into_any()
                } else if !is_loading {
                    view! {
                        <EmptySearch title=labels::STOCK_SEARCH_INITIAL_TITLE />
                    }
                        .into_any()
                } else {
                    view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
                        .into_any()
                }
            }}
        </PageFrame>
    }
}

#[component]
fn SearchForm(
    stock_code: RwSignal<String>,
    loading: Signal<bool>,
    /// 虫眼鏡アイコンのサイズ。PC は h-4、スマホは h-5
    #[prop(optional, default = "h-4 w-4")]
    icon_size: &'static str,
    on_submit: impl Fn(web_sys::SubmitEvent) + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <form on:submit=on_submit class="w-full">
            <SearchBox icon_size=icon_size>
                <SearchTextInput
                    placeholder=labels::search_placeholder(labels::STOCK_SEARCH_TARGET)
                    disabled=Signal::derive(move || loading.get())
                    value=move || stock_code.get()
                    on_input=move |value| stock_code.set(value)
                />
                <SearchSubmitButton
                    disabled=move || loading.get() || stock_code.get().is_empty()
                    loading=move || loading.get()
                    label=labels::STOCK_SEARCH_ACTION
                    loading_label=labels::SEARCH_LOADING
                />
            </SearchBox>
        </form>
    }
}

fn empty_search_description(title: &str) -> &'static str {
    if title == labels::STOCK_SEARCH_INITIAL_TITLE {
        labels::STOCK_SEARCH_INITIAL_HINT
    } else {
        labels::STOCK_SEARCH_NO_RESULT_HINT
    }
}

#[component]
fn EmptySearch(title: &'static str) -> impl IntoView {
    let description = empty_search_description(title);
    view! {
        <EmptyState
            class="py-10"
            title=title
            description=description
            icon=EmptyStateIcon::Search
        >
            <ul class="mt-4 space-y-1 text-sm font-medium text-text-muted">
                <li>{labels::STOCK_SEARCH_TIP_CODE}</li>
                <li>{labels::STOCK_SEARCH_TIP_NAME}</li>
                <li>{labels::STOCK_SEARCH_TIP_LINKS}</li>
            </ul>
        </EmptyState>
    }
}

#[component]
fn StockInfo(stock: Stock) -> impl IntoView {
    let meta = [
        (
            labels::STOCK_SEARCH_META_MARKET,
            stock.market_category.clone(),
        ),
        (
            labels::STOCK_SEARCH_META_INDUSTRY_33,
            stock.industry_category_33.clone().unwrap_or_default(),
        ),
        (
            labels::STOCK_SEARCH_META_INDUSTRY_17,
            stock.industry_category_17.clone().unwrap_or_default(),
        ),
        (
            labels::STOCK_SEARCH_META_SIZE,
            stock.size_category.clone().unwrap_or_default(),
        ),
    ];
    let code = stock.code.clone();
    let code_badge = code.clone();
    let name = normalize_display_name(&stock.name);
    view! {
        <div>
            <Card variant=CardVariant::Panel class="mb-4 overflow-hidden">
                <div class="border-b border-ink/10 bg-ink px-5 py-4 text-page">
                    <div class="flex flex-wrap items-center justify-between gap-3">
                        <h2 class="text-xl font-black leading-tight">{name}</h2>
                        <CodeBadge>{code_badge}</CodeBadge>
                    </div>
                </div>
                <div>
                    <dl class="grid grid-cols-1 gap-px bg-fill text-sm sm:grid-cols-2 lg:grid-cols-4">
                        {meta
                            .into_iter()
                            .map(|(label, value)| {
                                let display = if value.is_empty() {
                                    "—".to_string()
                                } else {
                                    value
                                };
                                view! {
                                    <div class="bg-surface px-4 py-3">
                                        <dt class="text-xs font-medium text-text-muted">
                                            {label}
                                        </dt>
                                        <dd class="mt-1 font-bold text-text-deep">{display}</dd>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </dl>
                    <div class="px-4 py-4">
                        <StockInfoLinks code=code />
                    </div>
                </div>
            </Card>
        </div>
    }
}

#[component]
fn StockInfoLinks(code: String) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 gap-2 sm:grid-cols-3 lg:grid-cols-4">
            {STOCK_LINKS
                .iter()
                .map(|(name, template)| {
                    let href = template.replace("{code}", &code);
                    view! {
                        <a
                            class="stock-link-button"
                            href={href}
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label=labels::stock_search_new_tab(name)
                        >
                            {*name}
                            <svg
                                class="h-3.5 w-3.5 shrink-0 text-text-faint"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                aria-hidden="true"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                                />
                            </svg>
                        </a>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[cfg(test)]
mod tests;
