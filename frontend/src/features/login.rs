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
            <div class="mb-6 flex flex-col items-center text-center">
                <img src="/icons/icon.svg" alt="" class="header-logo" />
                <p class="mt-3 text-xl font-black leading-tight tracking-normal text-ink">
                    "証券Web"
                </p>
                <p class="mt-1 text-sm font-medium text-text-muted">
                    "資産、配当、取引明細をひとつの作業面で確認します。"
                </p>
            </div>
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
