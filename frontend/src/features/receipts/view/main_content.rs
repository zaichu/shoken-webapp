use crate::features::receipts::dividend_info::{
    DividendInfoStore, DividendSummarySection, search_security_code,
};
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::kind::dividend_totals;
use crate::features::receipts::view::{ReceiptTable, SummaryStrip};
use crate::features::receipts::{ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState};
use crate::session::use_session;
use leptos::ev;
use leptos::prelude::*;

pub fn utility_display_rows(
    store: ReceiptsStore,
    tab: ReceiptsTab,
) -> Vec<crate::features::receipts::ReceiptRow> {
    match store.tab_state(tab) {
        TabState::Ready(data) => data.rows,
        _ => Vec::new(),
    }
}

#[component]
pub fn ReceiptsMainContent(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let _tab_state = store.tab_state(tab);
    let search = store.search;
    let summary_expanded = RwSignal::new(true);

    let dividend_info = (tab == ReceiptsTab::Dividend).then(|| {
        let session = use_session();
        DividendInfoStore::new(session)
    });

    if let Some(info) = dividend_info {
        let filtered_rows = Signal::derive(move || {
            if let TabState::Ready(data) = store.tab_state(tab) {
                filter_receipts(tab, &data.rows, &search.get().query)
            } else {
                Vec::new()
            }
        });
        let search_query = search;
        Effect::new(move |_| {
            let authenticated = use_session().user.get().is_some();
            let generation = use_session().generation.get();
            let query = search_query.with(|state| state.query.clone());
            let rows = filtered_rows.get();
            let code = search_security_code(&rows, &query);
            info.set_code(generation, authenticated, &code);
        });

        let refresh = move || {
            if web_sys::window()
                .and_then(|window| window.document())
                .is_some_and(|document| !document.hidden())
            {
                info.refresh_balance();
            }
        };
        let on_visible = window_event_listener(
            ev::Custom::<web_sys::Event>::new("visibilitychange"),
            move |_| refresh(),
        );
        let on_focus = window_event_listener(ev::focus, move |_| refresh());
        on_cleanup(move || {
            on_visible.remove();
            on_focus.remove();
        });
    }

    let csv_href = Signal::stored(format!("/receipts/csv?kind={}", tab.kind_param()));

    let is_empty = Signal::derive(
        move || matches!(store.tab_state(tab), TabState::Ready(data) if data.rows.is_empty()),
    );
    let show_dividend_info = Signal::derive(move || {
        dividend_info.is_some() && {
            let rows = if let TabState::Ready(data) = store.tab_state(tab) {
                filter_receipts(tab, &data.rows, &search.get().query)
            } else {
                Vec::new()
            };
            !search_security_code(&rows, &search.with(|s| s.query.clone())).is_empty()
        }
    });
    let dividend_totals_val = Signal::derive(move || {
        if show_dividend_info.get() {
            let rows = if let TabState::Ready(data) = store.tab_state(tab) {
                filter_receipts(tab, &data.rows, &search.get().query)
            } else {
                Vec::new()
            };
            Some(dividend_totals(&rows))
        } else {
            None
        }
    });
    let header = Signal::derive(move || {
        if !show_dividend_info.get() {
            if let TabState::Ready(data) = store.tab_state(tab) {
                let query = search.with(|s| s.query.clone());
                let rows = filter_receipts(tab, &data.rows, &query);
                Some(crate::features::receipts::view::summary::header_summary(
                    tab,
                    &ReceiptTabData {
                        rows: rows.clone(),
                        total: None,
                        summary: data.summary.clone(),
                        truncated: false,
                    },
                    &query,
                ))
            } else {
                None
            }
        } else {
            None
        }
    });
    let display_rows = Signal::derive(move || {
        if let TabState::Ready(data) = store.tab_state(tab) {
            data.rows.clone()
        } else {
            Vec::new()
        }
    });
    let filtered_rows = Signal::derive(move || {
        if let TabState::Ready(data) = store.tab_state(tab) {
            filter_receipts(tab, &data.rows, &search.get().query)
        } else {
            Vec::new()
        }
    });

    view! {
        {move || {
            if !matches!(store.tab_state(tab), TabState::Ready(_)) {
                view! { <div class="flex items-center justify-center h-64" data-testid="receipt-loading">"読み込み中..."</div> }.into_any()
            } else if is_empty.get() {
                view! {
                    <div class="flex flex-col items-center justify-center py-12 px-4 text-center" data-testid="receipt-empty">
                        <p class="text-text-muted mb-4">{tab.empty_hint()}</p>
                        <a
                            href=csv_href.get()
                            class="inline-flex min-h-11 items-center justify-center px-5 text-sm font-bold text-text-inverse rounded-md border border-ink bg-ink hover:bg-ink-hover transition-colors"
                        >
                            "CSVを取り込む"
                        </a>
                    </div>
                }.into_any()
            } else if show_dividend_info.get() {
                view! {
                    <div class="flex flex-col gap-3">
                        <section id="receipts-summary">
                            <DividendSummarySection
                                store=dividend_info.unwrap()
                                totals=dividend_totals_val.get().unwrap()
                                expanded=summary_expanded
                                mobile_expanded=store.mobile_summary_expanded
                            />
                        </section>
                        <section id="receipts-list">
                            <ReceiptTable
                                tab=tab
                                rows=filtered_rows.get()
                                all_rows=display_rows.get()
                                query=search.with(|s| s.query.clone())
                                expanded_ids=store.expanded
                            />
                        </section>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="flex flex-col gap-3">
                        <section id="receipts-summary">
                            <Show when=move || header.get().is_some() fallback=|| ()>
                                {move || header.get().map(|h| view! { <SummaryStrip items=h expanded=store.mobile_summary_expanded /> })}
                            </Show>
                        </section>
                        <section id="receipts-list">
                            <ReceiptTable
                                tab=tab
                                rows=filtered_rows.get()
                                all_rows=display_rows.get()
                                query=search.with(|s| s.query.clone())
                                expanded_ids=store.expanded
                            />
                        </section>
                    </div>
                }.into_any()
            }
        }}
    }
}
