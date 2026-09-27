use crate::features::asset_balance::view::AssetBalancePage;
use crate::features::home::HomePage;
use crate::features::login::LoginPage;
use crate::features::not_found::NotFoundPage;
use crate::features::receipts::view::ReceiptsPage;
use crate::features::stock_search::view::SearchPage;
use crate::session::{provide_session, SessionStore};
use crate::ui::elements::{current_path, Loading, SiteFooter, SiteHeader};
use leptos::prelude::*;

const BASE_TITLE: &str = "証券Web";

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

#[cfg(test)]
mod tests;
