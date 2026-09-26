use crate::asset_balance_page::AssetBalancePage;
use crate::dto::Stock;
use crate::home_page::HomePage;
use crate::login_page::LoginPage;
use crate::not_found_page::NotFoundPage;
use crate::receipts_page::ReceiptsPage;
use crate::search::use_stock_search;
use crate::session::{provide_session, SessionStore};
use crate::ui::{current_path, Alert, Loading, PageHeader, SiteFooter, SiteHeader, Spinner};
use leptos::prelude::*;

const BASE_TITLE: &str = "証券Web";

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

#[derive(Clone, Copy, Debug, PartialEq)]
enum Route {
    Home,
    Search,
    Receipts,
    AssetBalance,
    Login,
    NotFound,
}

impl Route {
    fn title(&self) -> &'static str {
        match self {
            Route::Home => "ホーム",
            Route::Search => "銘柄検索",
            Route::Receipts => "取引明細",
            Route::AssetBalance => "資産管理",
            Route::Login => "ログイン",
            Route::NotFound => "ページが見つかりません",
        }
    }

    fn protected(&self) -> bool {
        matches!(self, Route::Receipts | Route::AssetBalance)
    }
}

fn route_for_path(path: &str) -> Route {
    match path {
        "/" => Route::Home,
        "/search" => Route::Search,
        "/receipts" => Route::Receipts,
        "/assetbalance" => Route::AssetBalance,
        "/login" => Route::Login,
        "/404" => Route::NotFound,
        _ => Route::NotFound,
    }
}

fn current_route() -> Route {
    route_for_path(current_path().as_str())
}

fn replace_location(path: &str) {
    if let Some(history) = web_sys::window().and_then(|window| window.history().ok()) {
        let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(path));
    }
}

#[component]
pub fn App() -> impl IntoView {
    let route = current_route();
    Effect::new(move |_| {
        if route == Route::NotFound && current_path() != "/404" {
            replace_location("/404");
        }
    });
    let session = provide_session();
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title(&format!("{} - {BASE_TITLE}", route.title()));
    }
    Effect::new(move |_| {
        if !session.loaded.get() {
            return;
        }
        let authed = session.user.get().is_some();
        if route.protected() && !authed {
            SessionStore::redirect_to("/login");
        } else if route == Route::Login && authed {
            SessionStore::redirect_to("/");
        }
    });
    view! {
        <div class="min-h-screen flex flex-col bg-slate-50 text-slate-950">
            <a
                href="#main-content"
                class="skip-link"
            >
                "メインコンテンツへスキップ"
            </a>
            <SiteHeader />
            <main id="main-content" class="mx-auto w-full max-w-[1680px] px-4 sm:px-6 lg:px-8 flex flex-1 flex-col py-5">
                {move || {
                    if !session.loaded.get() {
                        return view! { <Loading /> }.into_any();
                    }
                    if route.protected() && session.user.get().is_none() {
                        return view! { <Loading /> }.into_any();
                    }
                    match route {
                        Route::Home => view! { <HomePage /> }.into_any(),
                        Route::Search => view! { <SearchPage /> }.into_any(),
                        Route::Receipts => view! { <ReceiptsPage /> }.into_any(),
                        Route::AssetBalance => view! { <AssetBalancePage /> }.into_any(),
                        Route::Login => view! { <LoginPage /> }.into_any(),
                        Route::NotFound => view! { <NotFoundPage /> }.into_any(),
                    }
                }}
            </main>
            <SiteFooter />
        </div>
    }
}

#[component]
fn SearchPage() -> impl IntoView {
    let search = use_stock_search();
    let loading = search.search.pending();
    let has_invalid = search.has_invalid_code_param;

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if !loading.get_untracked() && !search.stock_code.get_untracked().is_empty() {
            search.search.dispatch(search.stock_code.get_untracked());
        }
    };

    view! {
        <div class="page-surface">
            <PageHeader
                title="銘柄検索"
                eyebrow="Search"
                description="銘柄コードまたは銘柄名を入力して株式情報を検索できます。"
            />
            <SearchForm
                stock_code=search.stock_code
                loading=loading.into()
                on_submit=on_submit
            />
            <Show when=move || has_invalid>
                <Alert variant="warning">"不正な銘柄コードが指定されています。"</Alert>
            </Show>
            {move || {
                let data = search.stock_data();
                let error = search.error_message();
                let is_loading = loading.get();
                if let Some(message) = error {
                    view! {
                        <Alert variant="danger">
                            <strong>"エラー:"</strong>
                            " "
                            {message}
                        </Alert>
                    }
                        .into_any()
                } else if let Some(stock) = data {
                    view! { <StockInfo stock=stock /> }.into_any()
                } else if !is_loading {
                    view! {
                        <EmptySearch />
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
    on_submit: impl Fn(web_sys::SubmitEvent) + 'static,
) -> impl IntoView {
    view! {
        <div class="panel-card mb-5 overflow-hidden">
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
                        <button
                            type="submit"
                            class="search-submit no-print"
                            disabled=move || loading.get() || stock_code.get().is_empty()
                            data-loading=move || loading.get().then_some("true")
                            aria-label=move || {
                                if loading.get() { "検索中" } else { "銘柄を検索" }
                            }
                        >
                            {move || {
                                if loading.get() {
                                    view! {
                                        <Spinner size="sm" class="mr-2" />
                                        "読み込み中..."
                                    }
                                        .into_any()
                                } else {
                                    "検索".into_any()
                                }
                            }}
                        </button>
                    </div>
                </form>
            </div>
        </div>
    }
}

#[component]
fn EmptySearch() -> impl IntoView {
    view! {
        <div class="empty-state py-10">
            <div class="mb-3 text-slate-400" aria-hidden="true">
                <svg
                    class="h-10 w-10"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        stroke-width="1.5"
                        d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                    />
                </svg>
            </div>
            <h3 class="text-base font-black text-slate-950">"銘柄を検索"</h3>
            <p class="mt-1.5 max-w-md text-sm font-medium text-slate-600">
                "銘柄コード（例：7203）または銘柄名を入力して検索してください。"
            </p>
        </div>
    }
}

#[component]
fn SearchHints() -> impl IntoView {
    view! {
        <div class="mt-6 rounded-xl border border-slate-950/10 bg-slate-50/80 p-4">
            <h3 class="mb-2 text-sm font-black text-slate-800">"検索のヒント"</h3>
            <ul class="space-y-1 text-sm font-medium text-slate-600">
                <li>"4桁の銘柄コードで検索できます（例：7203, 9984）"</li>
                <li>"会社名の一部でも検索できます（例：トヨタ）"</li>
                <li>"検索結果から各種証券サイトへのリンクを確認できます"</li>
            </ul>
        </div>
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
    let name = stock.name.clone();
    view! {
        <div>
            <div class="panel-card mb-4 overflow-hidden">
                <div class="border-b border-slate-950/10 bg-slate-950 px-5 py-4 text-white">
                    <div class="flex flex-wrap items-end justify-between gap-3">
                        <div>
                            <p class="text-[11px] font-black uppercase tracking-[0.22em] text-amber-300">
                                "Security"
                            </p>
                            <h2 class="mt-1 text-xl font-black leading-tight">{name}</h2>
                        </div>
                        <span class="code-chip">
                            {code.clone()}
                        </span>
                    </div>
                </div>
                <div>
                    <dl class="grid grid-cols-1 gap-px bg-slate-200 text-sm sm:grid-cols-2 lg:grid-cols-4">
                        {meta
                            .into_iter()
                            .map(|(label, value)| {
                                let display = if value.is_empty() {
                                    "-".to_string()
                                } else {
                                    value
                                };
                                view! {
                                    <div class="bg-white px-4 py-3">
                                        <dt class="text-[11px] font-black uppercase tracking-[0.16em] text-slate-500">
                                            {label}
                                        </dt>
                                        <dd class="mt-1 font-bold text-dark">{display}</dd>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </dl>
                    <div class="px-4 py-4">
                        <StockInfoLinks code=code />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StockInfoLinks(code: String) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-2">
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
                                class="h-3.5 w-3.5 shrink-0 text-slate-400"
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
mod tests {
    use super::*;

    #[test]
    fn route_for_known_and_unknown_paths() {
        assert_eq!(route_for_path("/"), Route::Home);
        assert_eq!(route_for_path("/search"), Route::Search);
        assert_eq!(route_for_path("/receipts"), Route::Receipts);
        assert_eq!(route_for_path("/assetbalance"), Route::AssetBalance);
        assert_eq!(route_for_path("/login"), Route::Login);
        assert_eq!(route_for_path("/404"), Route::NotFound);
        assert_eq!(route_for_path("/unknown"), Route::NotFound);
        assert_eq!(route_for_path(""), Route::NotFound);
    }

    #[test]
    fn route_titles_and_protection() {
        for route in [
            Route::Home,
            Route::Search,
            Route::Receipts,
            Route::AssetBalance,
            Route::Login,
            Route::NotFound,
        ] {
            assert!(!route.title().is_empty());
        }
        assert_eq!(Route::Home.title(), "ホーム");
        assert_eq!(Route::Search.title(), "銘柄検索");
        assert_eq!(Route::Receipts.title(), "取引明細");
        assert_eq!(Route::AssetBalance.title(), "資産管理");
        assert_eq!(Route::Login.title(), "ログイン");
        assert_eq!(Route::NotFound.title(), "ページが見つかりません");
        assert!(!Route::Home.protected());
        assert!(!Route::Search.protected());
        assert!(Route::Receipts.protected());
        assert!(Route::AssetBalance.protected());
        assert!(!Route::Login.protected());
        assert!(!Route::NotFound.protected());
    }

    #[test]
    fn stock_links_contain_code_placeholder() {
        assert!(!STOCK_LINKS.is_empty());
        for (name, template) in STOCK_LINKS {
            assert!(!name.is_empty());
            assert!(template.contains("{code}"));
            assert!(template.starts_with("https://"));
        }
    }
}
