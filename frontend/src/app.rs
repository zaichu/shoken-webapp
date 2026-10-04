use crate::features::asset_balance::AssetBalancePage;
use crate::features::home::HomePage;
use crate::features::login::LoginPage;
use crate::features::not_found::NotFoundPage;
use crate::features::receipts::ReceiptsPage;
use crate::features::stock_search::SearchPage;
use crate::session::provide_session;
use crate::ui::elements::{current_location, CurrentPath, Loading, SiteFooter, SiteHeader};
use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

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

fn pathname_of(path: &str) -> &str {
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

fn scroll_to_top(window: &web_sys::Window) {
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
    scroll_to_top(&window);
    focus_main(&window);
}

#[component]
pub fn App() -> impl IntoView {
    let path = RwSignal::new(current_location());
    provide_context(CurrentPath(path));
    provide_context(AppOwner(Owner::current().unwrap_or_default()));
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
                scroll_to_top(&window);
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
            scroll_to_top(&window);
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
        let route = route.get();
        if route.protected() && !authed {
            navigate(path, "/login", true);
        } else if route == Route::Login && authed {
            navigate(path, "/", true);
        }
    });
    view! {
        <div class="min-h-screen flex flex-col bg-surface-sunken text-ink">
            <a
                href="#main-content"
                class="skip-link"
            >
                "メインコンテンツへスキップ"
            </a>
            <SiteHeader />
            <main
                id="main-content"
                tabindex="-1"
                class=move || {
                    if matches!(route.get(), Route::Receipts | Route::AssetBalance | Route::Search) {
                        // ワークスペース系は全幅グリッド(左パネル x=0 密着)のため中央コンテナを外す
                        "w-full flex flex-1 flex-col py-5".to_string()
                    } else {
                        "mx-auto w-full max-w-wide px-4 sm:px-6 lg:px-8 print:px-2 flex flex-1 flex-col py-5"
                            .to_string()
                    }
                }
            >
                {move || {
                    // ?code= などクエリだけの遷移でも再マウントさせるためパス全体を追跡する
                    path.get();
                    if !session.loaded.get() {
                        return view! { <Loading /> }.into_any();
                    }
                    let route = route.get();
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
