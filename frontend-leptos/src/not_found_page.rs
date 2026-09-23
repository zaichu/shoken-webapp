use leptos::prelude::*;

const RELATED_LINKS: &[(&str, &str)] = &[("/", "ホーム"), ("/search", "銘柄検索"), ("/receipts", "取引明細")];

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div class="min-h-[50vh] flex items-center justify-center">
            <div class="text-center">
                <h1 class="text-xl font-black">"404 - ページが見つかりません"</h1>
                <p class="mt-2 text-sm font-medium text-slate-600">
                    "お探しのページは存在しないか、移動した可能性があります。"
                </p>
                <div class="mt-4 flex flex-col items-center gap-4">
                    <a
                        href="/"
                        class="inline-flex items-center justify-center rounded-md font-bold bg-slate-950 text-white px-4 py-2 text-sm"
                    >
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
    }
}
