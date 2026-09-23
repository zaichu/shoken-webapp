use crate::receipts::{use_receipts_data, ReceiptItem, ReceiptsStore, ReceiptsTab, TabState};
use crate::session::use_session;
use leptos::prelude::*;

const TAB_IDS: [&str; 3] = ["dividend", "domesticstock", "mutualfund"];

#[component]
pub fn ReceiptsPage() -> impl IntoView {
    let store = use_receipts_data(use_session(), ReceiptsTab::Dividend);

    view! {
        <div class="page-surface">
            <div class="mb-5 max-sm:mb-2">
                <div class="flex flex-col gap-3 border-l-4 border-amber-500 pl-4 sm:flex-row sm:items-end sm:justify-between">
                    <div>
                        <p class="mb-1 text-[11px] font-bold uppercase tracking-[0.22em] text-slate-500 max-sm:hidden">
                            "Transactions"
                        </p>
                        <h1 class="text-2xl font-black leading-tight tracking-normal text-slate-950 max-sm:text-lg">
                            "取引明細"
                        </h1>
                        <p class="mt-1 text-sm font-medium text-slate-600 max-sm:hidden">
                            "配当金・国内株式・投資信託の取引明細を管理します。"
                        </p>
                    </div>
                </div>
            </div>
            {{
                let failed = store.clone();
                move || {
                    failed.error().map(|message| {
                        view! { <div role="alert">{message}</div> }
                    })
                }
            }}
            <nav class="mb-2 no-print" aria-label="取引明細タブ">
                <div
                    class="flex flex-wrap gap-1.5 rounded-xl border border-slate-950/10 bg-white/70 p-1 shadow-sm max-sm:flex-nowrap max-sm:gap-1 max-sm:overflow-x-auto"
                    role="tablist"
                >
                    {ReceiptsTab::ALL
                        .iter()
                        .copied()
                        .map(|tab| {
                            view! { <TabButton store=store.clone() tab=tab /> }
                        })
                        .collect_view()}
                </div>
            </nav>
            <div class="mt-0">
                <div data-testid="receipts-workspace">
                    {ReceiptsTab::ALL
                        .iter()
                        .copied()
                        .map(|tab| {
                            view! { <TabPanel store=store.clone() tab=tab /> }
                        })
                        .collect_view()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn TabButton(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let label = tab.label();
    let selected = store.clone();
    let index = store.clone();
    let clicked = store.clone();
    let counted = store.clone();
    view! {
        <button
            id={format!("tab-{slug}")}
            type="button"
            role="tab"
            aria-selected=move || selected.active_tab.get() == tab
            aria-controls={format!("tabpanel-{slug}")}
            tabindex=move || if index.active_tab.get() == tab { "0" } else { "-1" }
            on:click=move |_| clicked.select_tab(tab)
        >
            {label}
            <span data-testid={format!("tab-count-{slug}")}>
                {move || counted.count(tab).to_string()}
            </span>
        </button>
    }
}

#[component]
fn TabPanel(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let slug = TAB_IDS[tab as usize];
    let hidden = store.clone();
    let rendered = store.clone();
    view! {
        <div
            id={format!("tabpanel-{slug}")}
            role="tabpanel"
            aria-labelledby={format!("tab-{slug}")}
            hidden=move || hidden.active_tab.get() != tab
        >
            {move || {
                let store = rendered.clone();
                if store.active_tab.get() != tab {
                    return ().into_any();
                }
                if !store.is_authenticated() {
                    return view! { <p role="status">"読み込み中..."</p> }.into_any();
                }
                match store.tab_state(tab) {
                    TabState::Loading => view! { <p role="status">"読み込み中..."</p> }.into_any(),
                    TabState::Failed(message) => {
                        view! { <div role="alert">{message}</div> }.into_any()
                    }
                    TabState::Ready(rows) => {
                        if rows.is_empty() {
                            view! {
                                <div>
                                    <h3>"データがありません"</h3>
                                    <p>{empty_hint(tab)}</p>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! { <ReceiptTable tab=tab rows=rows /> }.into_any()
                        }
                    }
                }
            }}
        </div>
    }
}

fn empty_hint(tab: ReceiptsTab) -> &'static str {
    match tab {
        ReceiptsTab::Dividend => "配当金明細をCSVで追加してください",
        ReceiptsTab::DomesticStock => "国内株式明細をCSVで追加してください",
        ReceiptsTab::MutualFund => "投資信託明細をCSVで追加してください",
    }
}

#[component]
fn ReceiptTable(tab: ReceiptsTab, rows: Vec<ReceiptItem>) -> impl IntoView {
    let headers: &[&str] = match tab {
        ReceiptsTab::Dividend => &[
            "入金日",
            "商品",
            "口座",
            "銘柄コード",
            "銘柄名",
            "単価",
            "配当金",
            "税額",
            "受取額",
        ],
        ReceiptsTab::DomesticStock => &[
            "約定日",
            "銘柄コード",
            "銘柄名",
            "口座",
            "数量",
            "損益",
            "税額",
        ],
        ReceiptsTab::MutualFund => &["約定日", "ファンド名", "口座", "数量", "損益", "税額"],
    };
    view! {
        <table>
            <thead>
                <tr>
                    {headers.iter().map(|header| view! { <th>{*header}</th> }).collect_view()}
                </tr>
            </thead>
            <tbody>
                {rows
                    .into_iter()
                    .map(|row| {
                        let cells = row.cells();
                        view! {
                            <tr>
                                {cells.into_iter().map(|cell| view! { <td>{cell}</td> }).collect_view()}
                            </tr>
                        }
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}
