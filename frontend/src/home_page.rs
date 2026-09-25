use leptos::prelude::*;

const STATUS_ITEMS: &[(&str, &str, Option<&str>, &str, &str)] = &[
    (
        "銘柄検索",
        "検索",
        Some("/search"),
        "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
        "h-5 w-5 text-slate-950",
    ),
    (
        "資産管理",
        "一覧確認",
        Some("/assetbalance"),
        "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
        "h-5 w-5 text-slate-950",
    ),
    (
        "取引明細",
        "明細確認",
        Some("/receipts"),
        "M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
        "h-5 w-5 text-slate-950",
    ),
    (
        "CSV取込",
        "CSV反映",
        None,
        "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12",
        "h-5 w-5 text-slate-500",
    ),
];

const NEXT_ACTIONS: &[(&str, &str)] = &[
    ("/search", "銘柄検索"),
    ("/assetbalance", "資産管理"),
    ("/receipts", "取引明細"),
];

const FLOW_STEPS: &[(&str, &str)] = &[
    ("01", "CSV取得"),
    ("02", "各ページで取込"),
    ("03", "資産と明細を確認"),
];

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="page-surface space-y-7">
            <div class="grid gap-6 lg:grid-cols-[minmax(0,1.25fr)_minmax(20rem,0.75fr)] lg:items-end">
                <div>
                    <p class="mb-2 text-[11px] font-black uppercase tracking-[0.28em] text-amber-700">
                        "Portfolio Desk"
                    </p>
                    <h1 class="text-3xl font-black leading-tight tracking-normal text-slate-950 sm:text-4xl">
                        "証券Web"
                    </h1>
                    <p class="mt-2 max-w-2xl text-sm font-medium text-slate-600">
                        "資産、配当、取引明細をひとつの作業面で確認します。"
                    </p>
                </div>
                <div class="rounded-xl border border-slate-950/10 bg-slate-950 p-4 text-white shadow-[0_18px_44px_-34px_rgba(15,23,42,0.95)]">
                    <p class="text-[11px] font-bold uppercase tracking-[0.22em] text-amber-300">
                        "Current Focus"
                    </p>
                    <div class="mt-3 flex flex-wrap gap-2">
                        {NEXT_ACTIONS
                            .iter()
                            .map(|(to, label)| {
                                view! {
                                    <a
                                        href={*to}
                                        class="rounded-md border border-white/15 bg-white/10 px-3 py-2 text-sm font-bold text-white transition-colors hover:bg-white hover:text-slate-950"
                                    >
                                        {*label}
                                    </a>
                                }
                            })
                            .collect_view()}
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
                {STATUS_ITEMS
                    .iter()
                    .map(|(label, sub, to, icon_d, icon_class)| {
                        let inner = view! {
                            <div class="flex items-start justify-between gap-3">
                                <span class="inline-flex h-10 w-10 items-center justify-center rounded-md border border-slate-950/10 bg-white shadow-sm">
                                    <svg
                                        class={*icon_class}
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                        aria-hidden="true"
                                        focusable="false"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="1.8"
                                            d={*icon_d}
                                        />
                                    </svg>
                                </span>
                                <span class="text-[11px] font-black uppercase tracking-[0.18em] text-slate-400">
                                    {*sub}
                                </span>
                            </div>
                            <p class="mt-4 text-base font-black text-slate-950">{*label}</p>
                        };
                        match to {
                            Some(to) => {
                                view! {
                                    <a
                                        href={*to}
                                        class="group rounded-xl border border-slate-950/10 bg-white/80 px-4 py-4 shadow-sm transition-[border-color,box-shadow,transform] hover:-translate-y-0.5 hover:border-amber-500 hover:shadow-[0_18px_40px_-32px_rgba(15,23,42,0.85)]"
                                    >
                                        {inner}
                                    </a>
                                }
                                    .into_any()
                            }
                            None => {
                                view! {
                                    <div class="rounded-xl border border-dashed border-slate-300 bg-slate-100/70 px-4 py-4">
                                        {inner}
                                        <p class="mt-2 text-xs font-medium text-slate-500">"各ページから取込可能"</p>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    })
                    .collect_view()}
            </div>

            <div class="grid gap-4 border-t border-slate-950/10 pt-5 lg:grid-cols-[minmax(0,1fr)_minmax(18rem,0.55fr)]">
                <section>
                    <h2 class="text-sm font-black text-slate-950">"データ確認フロー"</h2>
                    <div class="mt-3 grid gap-2 sm:grid-cols-3">
                        {FLOW_STEPS
                            .iter()
                            .map(|(step, text)| {
                                view! {
                                    <div class="rounded-lg border border-slate-950/10 bg-white/75 px-3 py-3">
                                        <span class="text-[11px] font-black uppercase tracking-[0.18em] text-amber-700">
                                            {*step}
                                        </span>
                                        <p class="mt-1 text-sm font-bold text-slate-800">{*text}</p>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </div>
                </section>
                <section class="rounded-lg border border-slate-950/10 bg-white/70 px-4 py-3">
                    <h2 class="text-sm font-black text-slate-950">"操作ショートカット"</h2>
                    <div class="mt-3 flex flex-wrap gap-2">
                        {NEXT_ACTIONS
                            .iter()
                            .map(|(to, label)| {
                                view! {
                                    <a
                                        href={*to}
                                        class="inline-flex items-center rounded-md border border-slate-300 bg-white px-3 py-1.5 text-sm font-bold text-slate-700 transition-colors hover:border-slate-950 hover:text-slate-950"
                                    >
                                        {*label}
                                    </a>
                                }
                            })
                            .collect_view()}
                    </div>
                </section>
            </div>
        </div>
    }
}
