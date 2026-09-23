use leptos::prelude::*;

pub use crate::dto::SessionUser;

pub async fn fetch_session_user() -> Option<SessionUser> {
    let response = gloo_net::http::Request::get("/api/v1/session")
        .send()
        .await
        .ok()?;
    if !response.ok() {
        return None;
    }
    let user = response.json::<SessionUser>().await.ok()?;
    if user.id.is_empty() {
        return None;
    }
    Some(user)
}

pub fn redirect_to(path: &str) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href(path);
    }
}

pub fn use_session() -> (RwSignal<Option<SessionUser>>, RwSignal<bool>) {
    let user = RwSignal::new(None::<SessionUser>);
    let loaded = RwSignal::new(false);
    leptos::task::spawn_local(async move {
        user.set(fetch_session_user().await);
        loaded.set(true);
    });
    (user, loaded)
}
