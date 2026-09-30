use super::use_stock_search;
use crate::api::dto::Stock;
use crate::ui::badge::CodeBadge;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::elements::{Alert, AlertVariant, ListLoadError, PageHeader, Spinner, SpinnerSize};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use leptos::prelude::*;
use shared::normalize::normalize_display_name;

const STOCK_LINKS: &[(&str, &str)] = &[
    (
        "楽天証券",
        "https://www.rakuten-sec.co.jp/web/market/search/quote.html?ric={code}.T",
    ),
    (
        "SBI証券",
        "https://site3.sbisec.co.jp/ETGate/?_ControlID=WPLETsiR001Control&_DataStoreID=DSWPLETsiR001Control&_PageID=WPLETsiR001Ilst10&_ActionID=getDetailOfStockPriceJP&s_rkbn=1&i_stock_sec=%94%43%93%56%93%B0&i_dom_flg=1&i_exchange_code=JPN&i_output_type=0&stock_sec_code_mul={code}",
    ),
    ("株探", "https://kabutan.jp/stock/?code={code}"),
    ("Yahoo! Finance", "https://finance.yahoo.co.jp/quote/{code}"),
    ("日経", "https://www.nikkei.com/nkd/company/?scode={code}"),
    (
        "バフェットコード",
        "https://www.buffett-code.com/company/{code}",
    ),
    ("みんかぶ", "https://minkabu.jp/stock/{code}/"),
    ("IR BANK", "https://irbank.net/{code}"),
    (
        "銘柄スカウター",
        "https://monex.ifis.co.jp/index.php?sa=report_index&bcode={code}",
    ),
    ("ザイマニ", "https://zaimani.com/search/?_sf_s={code}"),
    (
        "JPX Explorer",
        "https://jpx-explorer.com/ja-JP/{code}-TSE",
    ),
];

#[component]
pub(crate) fn SearchPage() -> impl IntoView {
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

    view! {
        <div>
            <PageHeader
                title="銘柄検索"
                description="銘柄コードまたは銘柄名を入力して株式情報を検索できます。"
            />
            <SearchForm
                stock_code=stock_search.stock_code
                loading=loading.into()
                on_submit=on_submit
            />
            <Show when=move || has_invalid>
                <Alert variant=AlertVariant::Warning>"不正な銘柄コードが指定されています。"</Alert>
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
                        <EmptySearch title="該当する銘柄が見つかりませんでした" />
                        <SearchHints />
                    }
                        .into_any()
                } else if !is_loading {
                    view! {
                        <EmptySearch title="銘柄を検索" />
                        <SearchHints />
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}

#[component]
fn SearchForm(
    stock_code: RwSignal<String>,
    loading: Signal<bool>,
    on_submit: impl Fn(web_sys::SubmitEvent) + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Panel class="mb-5 overflow-hidden">
            <div class="p-4 sm:p-5">
                <form on:submit=on_submit>
                    <div class="search-input-frame">
                        <div class="flex-1 min-w-0">
                            <div class="w-full">
                                <input
                                    type="text"
                                    class="search-input"
                                    placeholder="銘柄コードまたは銘柄名"
                                    aria-label="銘柄コードまたは銘柄名"
                                    autocomplete="off"
                                    aria-invalid="false"
                                    disabled=move || loading.get()
                                    prop:value=move || stock_code.get()
                                    on:input=move |ev| {
                                        stock_code.set(event_target_value(&ev));
                                    }
                                />
                            </div>
                        </div>
                        <Button
                            variant=ButtonVariant::SearchSubmit
                            class="no-print"
                            submit=true
                            on_click=move |_| {}
                            disabled=Signal::derive(move || {
                                loading.get() || stock_code.get().is_empty()
                            })
                            data_loading=Signal::derive(move || loading.get())
                            aria_label=Signal::derive(move || {
                                if loading.get() { "検索中" } else { "銘柄を検索" }.to_string()
                            })
                        >
                            {move || {
                                if loading.get() {
                                    view! {
                                        <Spinner size=SpinnerSize::Sm class="mr-2" />
                                        "読み込み中..."
                                    }
                                        .into_any()
                                } else {
                                    "検索".into_any()
                                }
                            }}
                        </Button>
                    </div>
                </form>
            </div>
        </Card>
    }
}

#[component]
fn EmptySearch(title: &'static str) -> impl IntoView {
    let description = if title == "銘柄を検索" {
        "銘柄コード（例：7203）または銘柄名を入力して検索してください。"
    } else {
        "銘柄コードまたは銘柄名を確認してください。"
    };
    view! {
        <EmptyState
            class="py-10"
            title=title
            description=description
            icon=EmptyStateIcon::Search
        />
    }
}

#[component]
fn SearchHints() -> impl IntoView {
    view! {
        <Card variant=CardVariant::Hint class="mt-6">
            <h3 class="mb-2 text-sm font-black text-text">"検索のヒント"</h3>
            <ul class="space-y-1 text-sm font-medium text-text-muted">
                <li>"4桁の銘柄コードで検索できます（例：7203, 9984）"</li>
                <li>"会社名の一部でも検索できます（例：トヨタ）"</li>
                <li>"検索結果から各種証券サイトへのリンクを確認できます"</li>
            </ul>
        </Card>
    }
}

#[component]
fn StockInfo(stock: Stock) -> impl IntoView {
    let meta = [
        ("市場", stock.market_category.clone()),
        (
            "33業種",
            stock.industry_category_33.clone().unwrap_or_default(),
        ),
        (
            "17業種",
            stock.industry_category_17.clone().unwrap_or_default(),
        ),
        ("規模", stock.size_category.clone().unwrap_or_default()),
    ];
    let code = stock.code.clone();
    let code_badge = code.clone();
    let name = normalize_display_name(&stock.name);
    view! {
        <div>
            <Card variant=CardVariant::Panel class="mb-4 overflow-hidden">
                <div class="border-b border-ink/10 bg-ink px-5 py-4 text-text-inverse">
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
        <div class="grid grid-cols-1 xs:grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-2">
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
                            aria-label={format!("{name}（新しいタブで開く）")}
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
