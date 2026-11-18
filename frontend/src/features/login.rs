use crate::session;
use crate::ui::labels;
use crate::ui::login_prompt::{LoginLoading, LoginPageCenter, LoginPrompt};
use crate::ui::page_frame::PageFrame;
use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let session = session::use_session();
    view! {
        <PageFrame title=labels::LOGIN_TITLE>
            <LoginPageCenter>
                {move || {
                    if !session.loaded.get() || session.user.get().is_some() {
                        view! { <LoginLoading /> }.into_any()
                    } else {
                        view! { <LoginPrompt class="w-full" /> }.into_any()
                    }
                }}
            </LoginPageCenter>
        </PageFrame>
    }
}
