use crate::components::ui::Spinner;
use crate::session::use_session;
use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let session = use_session();
    view! {
        <div class="mx-auto flex max-w-md flex-col items-center py-12">
            {move || {
                let login = move |_| {
                    session.login();
                };
                if !session.loaded.get() {
                    return view! {
                        <div class="flex items-center justify-center py-12">
                            <Spinner size="lg" class="text-primary" />
                        </div>
                    }
                        .into_any();
                }
                if session.user.get().is_some() {
                    return view! {
                        <div class="flex items-center justify-center py-12">
                            <Spinner size="lg" class="text-primary" />
                        </div>
                    }
                        .into_any();
                }
                view! {
                    <div class="login-card">
                        <div class="space-y-6 p-6 text-center">
                            <h2 class="text-xl font-semibold">"ログイン"</h2>
                            <p class="text-sm text-secondary">
                                "Googleアカウントでログインしてください"
                            </p>
                            <button
                                type="button"
                                class="login-button"
                                on:click=login
                            >
                                <img
                                    src="https://www.gstatic.com/firebasejs/ui/2.0.0/images/auth/google.svg"
                                    alt="Google"
                                    width="20"
                                    height="20"
                                />
                                "Googleでログイン"
                            </button>
                        </div>
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
}
