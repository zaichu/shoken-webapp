use crate::features::receipts::dividend_info::{
    DividendInfoStore, DividendSummarySection, search_security_code,
};
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::kind::dividend_totals;
use crate::features::receipts::view::{ReceiptTable, SummaryStrip};
use crate::features::receipts::{
    ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState, truncated_list_warning,
};
use crate::session::use_session;
use crate::ui::button::{ButtonSize, ButtonVariant, LinkButton};
use crate::ui::card::{Card, CardVariant};
use crate::ui::state::{Alert, AlertVariant, ListLoadError};
use leptos::ev;
use leptos::prelude::*;

#[component]
pub fn ReceiptsMainContent(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
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

        // 別タブで更新された保有数を取り込むため、タブ復帰時に再取得する
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
        // 裏再取得の失敗は一覧を消さずに知らせる
        {move || {
            let Some(message) = store.refresh_error(tab) else {
                return ().into_any();
            };
            view! {
                <Alert variant=AlertVariant::Danger>
                    <strong>"エラー:"</strong>
                    " "
                    {message}
                </Alert>
            }
                .into_any()
        }}
        // 件数上限の警告は表の上に出す
        {move || match store.tab_state(tab) {
            TabState::Ready(data) if data.truncated => {
                view! { <Alert variant=AlertVariant::Warning>{truncated_list_warning()}</Alert> }
                    .into_any()
            }
            _ => ().into_any(),
        }}
        {move || match store.tab_state(tab) {
            TabState::Failed(message) => {
                let retry_store = store;
                view! {
                    <ListLoadError message=message on_retry=move || retry_store.reload(tab) />
                }
                    .into_any()
            }
            TabState::Ready(_) if is_empty.get() => {
                view! {
                    <Card variant=CardVariant::Soft class="p-6" testid="receipt-empty">
                        <p class="mb-4 text-center text-text-muted">{tab.empty_hint()}</p>
                        <LinkButton
                            variant=ButtonVariant::Primary(ButtonSize::Md)
                            href=csv_href.get()
                        >
                            "CSVを取り込む"
                        </LinkButton>
                    </Card>
                }
                    .into_any()
            }
            TabState::Ready(_) if show_dividend_info.get() => {
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
                                compact_mobile=true
                            />
                        </section>
                    </div>
                }
                    .into_any()
            }
            TabState::Ready(_) => {
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
                                compact_mobile=true
                            />
                        </section>
                    </div>
                }
                    .into_any()
            }
            TabState::Loading => view! { <div class="py-12 text-center text-sm text-text-muted" data-testid="receipt-loading">"読み込み中..."</div> }.into_any(),
        }}
    }
}
