use crate::session::use_session;
use crate::ui::login_prompt::LoginPrompt;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Spinner, SpinnerSize};
use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let session = use_session();
    view! {
        <PageFrame title="ログイン">
        <div class="mx-auto flex min-h-[50vh] w-full max-w-md flex-1 flex-col items-center justify-center py-12">
            {move || {
                if !session.loaded.get() || session.user.get().is_some() {
                    return view! {
                        <div class="flex items-center justify-center py-12">
                            <Spinner size=SpinnerSize::Lg class="text-text-deep" />
                        </div>
                    }
                        .into_any();
                }
                view! { <LoginPrompt class="w-full" /> }.into_any()
            }}
        </div>
        </PageFrame>
    }
}
