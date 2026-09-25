use crate::session::use_session;
use crate::ui::Spinner;
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
                    <div class="rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_14px_38px_-32px_rgba(15,23,42,0.85)] print:border-black print:shadow-none w-full shadow-md">
                        <div class="space-y-6 p-6 text-center">
                            <h2 class="text-xl font-semibold">"ログイン"</h2>
                            <p class="text-sm text-secondary">
                                "Googleアカウントでログインしてください"
                            </p>
                            <button
                                type="button"
                                class="inline-flex w-full items-center justify-center gap-2 rounded-md border border-dark px-4 py-2 text-sm font-semibold text-dark transition-colors hover:bg-dark hover:text-white"
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
