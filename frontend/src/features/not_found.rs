use leptos::prelude::*;

const RELATED_LINKS: &[(&str, &str)] = &[
    ("/", "ホーム"),
    ("/search", "銘柄検索"),
    ("/receipts", "取引明細"),
];

const PRIMARY_BUTTON_MD: &str = "inline-flex items-center justify-center rounded-md font-bold transition-[background-color,border-color,color,box-shadow,transform] focus:outline-none focus:ring-2 focus:ring-amber-500/50 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-60 no-print border border-slate-950 bg-slate-950 text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.12)] hover:bg-slate-800 active:bg-slate-950 px-4 py-2 text-sm max-sm:min-h-[44px]";

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="min-h-[50vh] flex items-center justify-center">
            <div class="empty-state">
                <div class="mb-3 text-slate-400" aria-hidden="true">
                    <svg
                        class="h-16 w-16"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        aria-hidden="true"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="1.5"
                            d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                        />
                    </svg>
                </div>
                <h1 class="text-2xl font-black text-slate-950">"404 - ページが見つかりません"</h1>
                <p class="mt-1.5 max-w-md text-sm font-medium text-slate-600">
                    "お探しのページは存在しないか、移動した可能性があります。"
                </p>
                <div class="mt-4">
                    <div class="flex flex-col items-center gap-4">
                        <a href="/" class=PRIMARY_BUTTON_MD>
                            "ホームに戻る"
                        </a>
                        <div class="flex flex-wrap justify-center gap-3">
                            {RELATED_LINKS
                                .iter()
                                .map(|(to, label)| {
                                    view! {
                                        <a href={*to} class="text-base text-primary hover:underline">
                                            {*label}
                                        </a>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
