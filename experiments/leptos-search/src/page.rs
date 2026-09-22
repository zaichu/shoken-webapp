use crate::api::Stock;
use crate::auth::{SessionUser, redirect_to, use_session};
use crate::home_page::HomePage;
use crate::receipts_page::ReceiptsPage;
use crate::search::use_stock_search;
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

const FOOTER_LINKS: &[(&str, &str)] = &[
    (
        "プライバシーポリシー",
        "https://github.com/zaichu/shoken-webapp/blob/main/docs/privacy-policy.md",
    ),
    (
        "利用規約",
        "https://github.com/zaichu/shoken-webapp/blob/main/docs/terms.md",
    ),
    (
        "Cookie ポリシー",
        "https://github.com/zaichu/shoken-webapp/blob/main/docs/cookie-policy.md",
    ),
];

#[derive(Clone, Copy, PartialEq)]
enum Route {
    Home,
    Search,
    Receipts,
    Login,
    NotFound,
}

impl Route {
    fn title(&self) -> &'static str {
        match self {
            Route::Home => "ホーム",
            Route::Search => "銘柄検索",
            Route::Receipts => "取引明細",
            Route::Login => "ログイン",
            Route::NotFound => "ページが見つかりません",
        }
    }

    fn protected(&self) -> bool {
        matches!(self, Route::Receipts)
    }
}

fn current_route() -> Route {
    match current_path().as_str() {
        "/" => Route::Home,
        "/search" => Route::Search,
        "/receipts" => Route::Receipts,
        "/login" => Route::Login,
        "/404" => Route::NotFound,
        _ => Route::NotFound,
    }
}

#[component]
pub fn App() -> impl IntoView {
    let route = current_route();
    let (user, loaded) = use_session();
    if let Some(document) = web_sys::window().and_then(|w| w.document()) {
        document.set_title(&format!("{} - {BASE_TITLE}", route.title()));
    }
    Effect::new(move |_| {
        if !loaded.get() {
            return;
        }
        let authed = user.get().is_some();
        if route.protected() && !authed {
            redirect_to("/login");
        } else if route == Route::Login && authed {
            redirect_to("/");
        }
    });
    view! {
        <div class="min-h-screen flex flex-col bg-slate-50 text-slate-950">
            <a
                href="#main-content"
                class="sr-only focus:not-sr-only focus:fixed focus:top-2 focus:left-2 focus:z-50 focus:rounded-md focus:bg-white focus:px-4 focus:py-2 focus:text-sm focus:font-semibold focus:shadow-lg focus:outline-2 focus:outline-primary"
            >
                "メインコンテンツへスキップ"
            </a>
            <SiteHeader user=user />
            <main id="main-content" class="mx-auto w-full max-w-[1680px] px-4 sm:px-6 lg:px-8 flex flex-1 flex-col py-5">
                {match route {
                    Route::Home => view! { <HomePage /> }.into_any(),
                    Route::Search => view! { <SearchPage /> }.into_any(),
                    Route::Receipts => view! { <ReceiptsPage /> }.into_any(),
                    Route::Login => view! { <LoginPlaceholder /> }.into_any(),
                    Route::NotFound => view! { <NotFoundPlaceholder /> }.into_any(),
                }}
            </main>
            <SiteFooter />
        </div>
    }
}

fn current_path() -> String {
    web_sys::window()
        .and_then(|window| window.location().pathname().ok())
        .unwrap_or_default()
}

#[component]
fn SiteHeader(user: RwSignal<Option<SessionUser>>) -> impl IntoView {
    view! {
        <header class="border-b border-slate-200 bg-white">
            <div class="mx-auto w-full max-w-[1680px] px-4 sm:px-6 lg:px-8 flex h-14 items-center justify-between">
                <a href="/" class="text-base font-black tracking-tight text-slate-950">
                    "証券Web"
                </a>
                <nav aria-label="メインナビゲーション">
                    <a href="/search" class="text-sm font-bold text-slate-700 hover:text-slate-950">
                        "銘柄検索"
                    </a>
                    <a href="/assetbalance" class="ml-4 text-sm font-bold text-slate-700 hover:text-slate-950">
                        "資産管理"
                    </a>
                    <a href="/receipts" class="ml-4 text-sm font-bold text-slate-700 hover:text-slate-950">
                        "取引明細"
                    </a>
                </nav>
                <div class="flex items-center gap-3">
                    {move || {
                        user.get()
                            .map(|session| {
                                view! { <span class="text-sm font-semibold">{session.display_name()}</span> }
                                    .into_any()
                            })
                            .unwrap_or_else(|| {
                                view! {
                                    <a
                                        href="/login"
                                        class="text-sm font-bold text-slate-700 hover:text-slate-950"
                                    >
                                        "ログイン"
                                    </a>
                                }
                                    .into_any()
                            })
                    }}
                </div>
            </div>
        </header>
    }
}

#[component]
fn LoginPlaceholder() -> impl IntoView {
    view! {
        <div class="mx-auto flex max-w-md flex-col items-center py-12">
            <h2 class="text-xl font-semibold">"ログイン"</h2>
        </div>
    }
}

#[component]
fn NotFoundPlaceholder() -> impl IntoView {
    view! {
        <div class="min-h-[50vh] flex items-center justify-center">
            <h1 class="text-xl font-black">"404 - ページが見つかりません"</h1>
        </div>
    }
}

#[component]
fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="mt-auto border-t border-slate-200 bg-white py-4">
            <div class="mx-auto w-full max-w-[1680px] px-4 sm:px-6 lg:px-8 flex flex-wrap justify-center gap-x-6 gap-y-1 text-sm text-slate-500">
                {FOOTER_LINKS
                    .iter()
                    .map(|(label, href)| {
                        view! {
                            <a
                                href={*href}
                                target="_blank"
                                rel="noopener noreferrer"
                                class="hover:text-slate-700 hover:underline"
                                aria-label={format!("{label}（新しいタブで開く）")}
                            >
                                {*label}
                            </a>
                        }
                    })
                    .collect_view()}
                <span>"© 2026 shoken-webapp"</span>
            </div>
        </footer>
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
fn PageHeader(
    title: &'static str,
    eyebrow: &'static str,
    description: &'static str,
) -> impl IntoView {
    view! {
        <div class="mb-5 max-sm:mb-2">
            <div class="flex flex-col gap-3 border-l-4 border-amber-500 pl-4 sm:flex-row sm:items-end sm:justify-between">
                <div>
                    <p class="mb-1 text-[11px] font-bold uppercase tracking-[0.22em] text-slate-500 max-sm:hidden">
                        {eyebrow}
                    </p>
                    <h1 class="text-2xl font-black leading-tight tracking-normal text-slate-950 max-sm:text-lg">
                        {title}
                    </h1>
                    <p class="mt-1 text-sm font-medium text-slate-600 max-sm:hidden">{description}</p>
                </div>
            </div>
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
        <div class="rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_14px_38px_-32px_rgba(15,23,42,0.85)] print:border-black print:shadow-none mb-5 overflow-hidden border-slate-950/10">
            <div class="p-4 sm:p-5">
                <form on:submit=on_submit>
                    <div class="flex w-full items-stretch rounded-lg border border-slate-300 bg-white p-1 shadow-inner shadow-slate-200/80 focus-within:border-amber-600 focus-within:ring-2 focus-within:ring-amber-500/20">
                        <div class="flex-1 min-w-0">
                            <div class="w-full">
                                <input
                                    type="text"
                                    class="block px-3 py-2 text-sm font-medium text-slate-950 bg-white border rounded-md transition-[border-color,box-shadow,background-color] placeholder:text-slate-400 focus:outline-none focus:ring-2 focus:ring-amber-500/25 border-slate-300 focus:border-amber-600 w-full h-11 rounded-r-none border-0 bg-transparent text-base shadow-none focus:ring-0"
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
                            class="inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print border border-slate-950 bg-slate-950 text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.12)] hover:bg-slate-800 active:bg-slate-950 px-4 py-2 text-sm max-sm:min-h-[44px] h-11 shrink-0 whitespace-nowrap rounded-md px-5"
                            disabled=move || loading.get() || stock_code.get().is_empty()
                            data-loading=move || loading.get().then_some("true")
                            aria-label=move || {
                                if loading.get() { "検索中" } else { "銘柄を検索" }
                            }
                        >
                            {move || {
                                if loading.get() {
                                    view! {
                                        <span class="inline-flex items-center mr-2">
                                            <svg
                                                class="animate-spin h-4 w-4"
                                                xmlns="http://www.w3.org/2000/svg"
                                                fill="none"
                                                viewBox="0 0 24 24"
                                                role="status"
                                                aria-label="読み込み中..."
                                            >
                                                <circle
                                                    class="opacity-25"
                                                    cx="12"
                                                    cy="12"
                                                    r="10"
                                                    stroke="currentColor"
                                                    stroke-width="4"
                                                />
                                                <path
                                                    class="opacity-75"
                                                    fill="currentColor"
                                                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                                                />
                                            </svg>
                                        </span>
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
fn Alert(variant: &'static str, children: Children) -> impl IntoView {
    let variant_class = match variant {
        "warning" => "border-amber-200 bg-amber-50 text-amber-900",
        "danger" => "border-red-200 bg-red-50 text-red-700",
        "success" => "border-teal-200 bg-teal-50 text-teal-800",
        _ => "border-blue-200 bg-blue-50 text-blue-800",
    };
    view! {
        <div
            class={format!(
                "rounded-lg border px-4 py-3 text-sm font-medium shadow-sm {variant_class}"
            )}
            role="alert"
        >
            {children()}
        </div>
    }
}

#[component]
fn EmptySearch() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center rounded-xl border border-dashed border-slate-300 bg-slate-50/70 px-4 py-8 text-center py-10">
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
            <div class="rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_14px_38px_-32px_rgba(15,23,42,0.85)] print:border-black print:shadow-none mb-4 overflow-hidden">
                <div class="border-b border-slate-950/10 bg-slate-950 px-5 py-4 text-white">
                    <div class="flex flex-wrap items-end justify-between gap-3">
                        <div>
                            <p class="text-[11px] font-black uppercase tracking-[0.22em] text-amber-300">
                                "Security"
                            </p>
                            <h2 class="mt-1 text-xl font-black leading-tight">{name}</h2>
                        </div>
                        <span class="inline-flex rounded-md border border-white/20 bg-white px-3 py-1 font-mono text-sm font-black tracking-wider text-slate-950">
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
                            class="inline-flex items-center justify-between gap-1 rounded-md border border-slate-200 bg-white px-3 py-2 text-sm font-medium text-primary hover:bg-slate-50 hover:border-primary transition-colors"
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

