use yew::prelude::*;
use yew_router::prelude::*;

use crate::{app::Route, data::use_info::UserInfo};

#[function_component]
pub fn Layout(props: &yew::html::ChildrenProps) -> Html {
    let _user_info = use_context::<UserInfo>();
    html! {
        <>
            // { render_auth_component(user_info) }
            <nav class="navbar bg-dark navbar-expand-lg bg-body-tertiary" data-bs-theme="dark">
                <div class="container-fluid" style="max-width: 1600px;">
                    <Link<Route> classes="navbar-brand" to={Route::Home}>{ "証券Web" }</Link<Route>>
                    <button class="navbar-toggler" type="button" data-bs-toggle="collapse" data-bs-target="#navbarNav" aria-controls="navbarNav" aria-expanded="false" aria-label="Toggle navigation">
                        <span class="navbar-toggler-icon"></span>
                    </button>
                    <div class="collapse navbar-collapse" id="navbarNav">
                        <ul class="nav navbar-nav nav-underline justify-content-center">
                            <li class="nav-item">
                                <Link<Route> classes="nav-link" to={Route::Search}>{ "銘柄検索" }</Link<Route>>
                            </li>
                            <li class="nav-item">
                                <Link<Route> classes="nav-link" to={Route::Receipts}>{ "受取金" }</Link<Route>>
                            </li>
                        </ul>
                    </div>
                </div>
            </nav>
            <main class="container mt-4" style="max-width: 1600px;">
                { props.children.clone() }
            </main>
            <footer class="bg-light text-center text-lg-start mt-4">
                <div class="container p-1">
                    <p class="text-center mt-3">{ "© 2024 証券Web" }</p>
                </div>
            </footer>
        </>
    }
}
/*
#[function_component]
fn Login() -> Html {
    let on_click = Callback::from(move |_| {
        let future = async {
            match google_oauth().await {
                Ok((auth_url, _csrf_token)) => {
                    let window = window().unwrap();
                    window.location().set_href(&auth_url.to_string()).unwrap();
                }
                Err(err) => console::log!(&err.to_string()),
            }
        };
        yew::platform::spawn_local(future);
    });

    html! {
    <div class="mb-3">
        <button onclick={on_click} class="btn btn-primary btn-lg">
            <i class="fab fa-google me-2"/>
            { "Googleでログイン" }
        </button>
    </div>
    }
}

#[function_component]
fn Logout() -> Html {
    let on_click = Callback::from(move |_| {
        if let Some(storage) = window().and_then(|w| w.local_storage().ok()).flatten() {
            _ = storage.remove_item("user_info");
            yew::platform::spawn_local(async move {
                if let Some(window) = window() {
                    let _ = window.location().set_href(&Route::Home.to_path());
                }
            });
        }
    });

    html! {
    <div class="mb-3">
        <button onclick={on_click} class="btn btn-outline-danger">
            <i class="fas fa-sign-out-alt me-2"/>
                { "ログアウト" }
        </button>
    </div>
    }
}

fn render_auth_component(user_info: Option<UserInfo>) -> Html {
    let user_info = user_info.unwrap_or_default();
    match user_info.auth_code {
        Some(_) => html! { <Logout /> },
        None => html! { <Login /> },
    }
}
 */
