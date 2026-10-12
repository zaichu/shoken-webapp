use crate::features::asset_balance::{
    AssetBalanceCsvDeletePage, AssetBalanceCsvPage, AssetBalancePage, AssetBalanceState,
};
use crate::features::home::{HomeOverviewData, HomePage};
use crate::features::login::LoginPage;
use crate::features::not_found::NotFoundPage;
use crate::features::receipts::{
    ReceiptsConditionsPage, ReceiptsCsvDeletePage, ReceiptsCsvPage, ReceiptsPage, ReceiptsStore,
};
use crate::features::stock_search::StockSearchPage;
use crate::session::provide_session;
use crate::ui::site::{AccountDeleteConfirm, AccountDeleteDialog, BottomNav, SiteFooter};
use crate::ui::state::Loading;
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const BASE_TITLE: &str = "証券Web";

pub(crate) fn current_location() -> String {
    web_sys::window()
        .and_then(|window| {
            let location = window.location();
            let pathname = location.pathname().ok()?;
            let search = location.search().ok()?;
            Some(format!("{pathname}{search}"))
        })
        .unwrap_or_default()
}

/// アプリ内遷移でナビのアクティブ表示を追随させるための現在パス(pathname + search)
#[derive(Clone, Copy)]
pub(crate) struct CurrentPath(pub RwSignal<String>);

// ?kind= のような単一クエリを pathname + search 形式の文字列から取る。
// 同じキーの複数指定は先勝ち、パーセントデコードはしない(種別値はASCIIのみ)
pub(crate) fn query_param(location: &str, key: &str) -> Option<String> {
    let query = location.split_once('?')?.1.split('#').next()?;
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .find(|(name, _)| *name == key)
        .map(|(_, value)| value.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Route {
    Home,
    Search,
    Receipts,
    ReceiptsConditions,
    ReceiptsCsv,
    ReceiptsCsvDelete,
    AssetBalance,
    AssetBalanceComposition,
    AssetBalanceHolding,
    AssetBalanceCsv,
    AssetBalanceCsvDelete,
    Login,
    NotFound,
}

impl Route {
    fn title(&self) -> &'static str {
        match self {
            Route::Home => "ホーム",
            Route::Search => "銘柄検索",
            Route::Receipts => "取引明細",
            Route::ReceiptsConditions => "検索条件",
            Route::ReceiptsCsv | Route::AssetBalanceCsv => "CSV管理",
            Route::ReceiptsCsvDelete | Route::AssetBalanceCsvDelete => "データ削除",
            Route::AssetBalance => "資産管理",
            Route::AssetBalanceComposition => "構成比",
            Route::AssetBalanceHolding => "保有銘柄詳細",
            Route::Login => "ログイン",
            Route::NotFound => "ページが見つかりません",
        }
    }
}

fn holding_id(path: &str) -> Option<&str> {
    let id = path.strip_prefix("/assetbalance/holdings/")?;
    (!id.is_empty() && !id.contains('/')).then_some(id)
}

fn route_for_path(path: &str) -> Route {
    match path {
        "/" => Route::Home,
        "/search" => Route::Search,
        "/receipts" => Route::Receipts,
        "/receipts/conditions" => Route::ReceiptsConditions,
        "/receipts/csv" => Route::ReceiptsCsv,
        "/receipts/csv/delete" => Route::ReceiptsCsvDelete,
        "/assetbalance" => Route::AssetBalance,
        "/assetbalance/composition" => Route::AssetBalanceComposition,
        "/assetbalance/csv" => Route::AssetBalanceCsv,
        "/assetbalance/csv/delete" => Route::AssetBalanceCsvDelete,
        path if holding_id(path).is_some() => Route::AssetBalanceHolding,
        "/login" => Route::Login,
        "/404" => Route::NotFound,
        _ => Route::NotFound,
    }
}

// 取引明細・資産管理配下は未ログインで開けない(新設ルートもプレフィックスで保護する)
fn protected_path(path: &str) -> bool {
    let path = pathname_of(path);
    matches!(path, "/receipts" | "/assetbalance")
        || path.starts_with("/receipts/")
        || path.starts_with("/assetbalance/")
}

pub(crate) fn pathname_of(path: &str) -> &str {
    path.split(['?', '#']).next().unwrap_or(path)
}

// 同一オリジンのルート相対リンクだけをアプリ内遷移にする(外部・mailto 等は素通し)
fn spa_path(href: &str) -> Option<String> {
    if !href.starts_with('/') || href.starts_with("//") {
        return None;
    }
    Some(href.to_string())
}

fn spa_href_parts(href: Option<&str>, has_download: bool, target: Option<&str>) -> Option<String> {
    if has_download {
        return None;
    }
    if target.is_some_and(|target| target != "_self") {
        return None;
    }
    spa_path(href?)
}

fn spa_href(anchor: &web_sys::Element) -> Option<String> {
    spa_href_parts(
        anchor.get_attribute("href").as_deref(),
        anchor.has_attribute("download"),
        anchor.get_attribute("target").as_deref(),
    )
}

fn hash_only_change(from: &str, to: &str) -> bool {
    to.len() > from.len() && to.starts_with(from) && to.as_bytes()[from.len()] == b'#'
}

fn strip_hash(path: &str) -> &str {
    path.split('#').next().unwrap_or(path)
}

// 本文は main がスクロールする。ページ遷移ではそのコンテナを先頭に戻す
fn scroll_top(window: &web_sys::Window) {
    if let Some(main) = window
        .document()
        .and_then(|document| document.get_element_by_id("main-content"))
    {
        main.set_scroll_top(0);
    }
    window.scroll_to_with_x_and_y(0.0, 0.0);
}

// SPA 遷移でフォーカスが取り残されないよう main に移す(tabindex=-1 で Tab 順には入らない)
fn focus_main(window: &web_sys::Window) {
    if let Some(main) = window
        .document()
        .and_then(|document| document.get_element_by_id("main-content"))
        .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = main.focus();
    }
}

// ページをまたいで保持するストアをぶら下げるためのアプリ寿命のオーナー
#[derive(Clone)]
pub(crate) struct AppOwner(pub Owner);

// ページごとの遅延状態スロット。ページ遷移でビューが作り直されても取得済みデータを
// 失わないよう、各ページは初回利用時に自分のスロットへ構築結果を置く。
// Owner は Send ではないため LocalStorage のシグナルを使う
#[derive(Clone)]
pub(crate) struct PageStates {
    // テストでは use_receipts_data が都度新しいストアを返すため、このスロットは読まれない
    #[cfg_attr(test, allow(dead_code))]
    pub(crate) receipts: RwSignal<Option<(Owner, ReceiptsStore)>, LocalStorage>,
    pub(crate) asset_balance: RwSignal<Option<(Owner, AssetBalanceState)>, LocalStorage>,
    pub(crate) home_overview: RwSignal<Option<(Owner, HomeOverviewData)>, LocalStorage>,
}

impl Default for PageStates {
    fn default() -> Self {
        Self {
            receipts: RwSignal::new_local(None),
            asset_balance: RwSignal::new_local(None),
            home_overview: RwSignal::new_local(None),
        }
    }
}

// 空のスロットを AppOwner の子オーナーで構築して保持する共通手順。
// Owner::new() は現オーナーの子として登録されページと一緒に破棄されるため、AppOwner の子を使う
pub(crate) fn page_state<T: Copy + 'static>(
    slot: RwSignal<Option<(Owner, T)>, LocalStorage>,
    build: impl FnOnce() -> T,
) -> T {
    if let Some((_, state)) = slot.get_untracked() {
        return state;
    }
    let owner = use_context::<AppOwner>()
        .map(|app| app.0)
        .unwrap_or_default()
        .child();
    let state = owner.with(build);
    slot.set(Some((owner, state)));
    state
}

fn navigate(path: RwSignal<String>, to: &str, replace: bool) {
    let Some(window) = web_sys::window() else {
        return;
    };
    if let Ok(history) = window.history() {
        let result = if replace {
            history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(to))
        } else {
            history.push_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(to))
        };
        let _ = result;
    }
    path.set(strip_hash(to).to_string());
    scroll_top(&window);
    focus_main(&window);
}

#[component]
pub fn App() -> impl IntoView {
    let path = RwSignal::new(current_location());
    provide_context(CurrentPath(path));
    provide_context(AppOwner(Owner::current().unwrap_or_default()));
    provide_context(PageStates::default());
    provide_context(AccountDeleteConfirm(RwSignal::new(false)));
    let route = Memo::new(move |_| route_for_path(pathname_of(&path.get())));

    // 同一オリジンのリンクをクリック遷移に変え、wasm とセッション確認のやり直しを省く
    let on_click = window_event_listener(ev::click, move |ev| {
        if ev.default_prevented()
            || ev.button() != 0
            || ev.meta_key()
            || ev.ctrl_key()
            || ev.shift_key()
            || ev.alt_key()
        {
            return;
        }
        let Some(anchor) = ev
            .target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .and_then(|element| element.closest("a[href]").ok().flatten())
        else {
            return;
        };
        let Some(to) = spa_href(&anchor) else {
            return;
        };
        // #要素 への移動はブラウザの既定のスクロールに任せる
        if hash_only_change(&path.get_untracked(), &to) {
            return;
        }
        ev.prevent_default();
        if to == path.get_untracked() {
            if let Some(window) = web_sys::window() {
                scroll_top(&window);
            }
            return;
        }
        navigate(path, &to, false);
    });
    on_cleanup(move || drop(on_click));

    let on_popstate = window_event_listener(ev::popstate, move |_| {
        // #フラグメント遷移でも popstate は発火するので、パスが変わらない限り再描画しない
        let next = current_location();
        if path.get_untracked() == next {
            return;
        }
        path.set(next);
        if let Some(window) = web_sys::window() {
            scroll_top(&window);
            // 戻る/進むでも遷移元のフォーカスがアンマウントされて取り残されないようにする
            focus_main(&window);
        }
    });
    on_cleanup(move || drop(on_popstate));

    let session = provide_session();
    // route だけを見ると不明パス同士の遷移(NotFound→NotFound)で再発火しないため path を追跡する
    Effect::new(move |_| {
        if route.get() == Route::NotFound && pathname_of(&path.get()) != "/404" {
            navigate(path, "/404", true);
        }
    });
    Effect::new(move |_| {
        let title = route.get().title();
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            document.set_title(&format!("{title} - {BASE_TITLE}"));
        }
    });
    Effect::new(move |_| {
        if !session.loaded.get() {
            return;
        }
        let authed = session.user.get().is_some();
        let current = path.get();
        if protected_path(&current) && !authed {
            navigate(path, "/login", true);
        } else if route.get() == Route::Login && authed {
            navigate(path, "/", true);
        }
    });
    let show_bottom_nav = move || {
        matches!(
            route.get(),
            Route::Home
                | Route::Search
                | Route::Receipts
                | Route::ReceiptsConditions
                | Route::ReceiptsCsv
                | Route::ReceiptsCsvDelete
                | Route::AssetBalance
                | Route::AssetBalanceComposition
                | Route::AssetBalanceHolding
                | Route::AssetBalanceCsv
                | Route::AssetBalanceCsvDelete
        )
    };
    view! {
        <div class="app-shell">
            <a
                href="#main-content"
                class="skip-link"
            >
                "メインコンテンツへスキップ"
            </a>
            <main
                id="main-content"
                tabindex="-1"
                class="app-scroll"
            >
                {move || {
                    // ?code= などクエリだけの遷移でも再マウントさせるためパス全体を追跡する
                    path.get();
                    if !session.loaded.get() {
                        return view! { <Loading /> }.into_any();
                    }
                    if protected_path(&path.get()) && session.user.get().is_none() {
                        return view! { <Loading /> }.into_any();
                    }
                    match route.get() {
                        Route::Home => view! { <HomePage /> }.into_any(),
                        Route::Search => view! { <StockSearchPage /> }.into_any(),
                        Route::Receipts => view! { <ReceiptsPage /> }.into_any(),
                        Route::ReceiptsConditions => view! { <ReceiptsConditionsPage /> }.into_any(),
                        Route::ReceiptsCsv => view! { <ReceiptsCsvPage /> }.into_any(),
                        Route::ReceiptsCsvDelete => view! { <ReceiptsCsvDeletePage /> }.into_any(),
                        Route::AssetBalance => view! { <AssetBalancePage /> }.into_any(),
                        Route::AssetBalanceCsv => view! { <AssetBalanceCsvPage /> }.into_any(),
                        Route::AssetBalanceCsvDelete => {
                            view! { <AssetBalanceCsvDeletePage /> }.into_any()
                        }
                        Route::Login => view! { <LoginPage /> }.into_any(),
                        // 設計済みルートは認証・タイトル・ナビ活性の土台だけ先に用意し、
                        // ページ本体は #1269-#1270 で実装する
                        _ => view! { <NotFoundPage /> }.into_any(),
                    }
                }}
                <SiteFooter />
            </main>
            {move || show_bottom_nav().then(|| view! { <BottomNav class="lg:hidden" /> })}
            <AccountDeleteDialog />
        </div>
    }
}

#[cfg(test)]
mod tests;
