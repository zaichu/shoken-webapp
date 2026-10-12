use super::summary::{SummaryStrip, header_summary};
use super::table::ReceiptTable;
use crate::features::receipts::dividend_info::{
    DividendInfoStore, DividendSummarySection, search_security_code,
};
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::kind::dividend_totals;
use crate::features::receipts::{ReceiptTabData, ReceiptsStore, ReceiptsTab};
use crate::session::use_session;
use crate::ui::button::{ButtonSize, ButtonVariant, LinkButton};
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use leptos::ev;
use leptos::prelude::*;

#[component]
pub(crate) fn ReceiptsMainContent(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    data: ReceiptTabData,
) -> impl IntoView {
    let search = store.search;
    let summary = data.summary.clone();
    let display_rows = data.rows.clone();
    let filtered = Memo::new(move |_| filter_receipts(tab, &display_rows, &search.get().query));
    // 配当タブの銘柄コード検索時に DividendInfo を出すための取得状態。
    // フィルタ結果・クエリ・セッション変化に追随して対象コードを更新する。
    let session = use_session();
    let dividend_info = (tab == ReceiptsTab::Dividend).then(|| DividendInfoStore::new(session));
    let summary_expanded = RwSignal::new(true);
    if let Some(info) = dividend_info {
        let filtered_rows = filtered;
        let search_query = search;
        Effect::new(move |_| {
            let authenticated = session.user.get().is_some();
            let generation = session.generation.get();
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
    view! {
        {move || {
            let display = data.rows.clone();
            if display.is_empty() {
                return view! {
                    <Card variant=CardVariant::Shell testid="receipt-card">
                        <div class="p-0" data-testid="receipt-card-body">
                            <EmptyState
                                title="データがありません"
                                description=tab.empty_hint().to_string()
                                icon=EmptyStateIcon::Tray
                            >
                                <div class="mt-4">
                                    <LinkButton
                                        variant=ButtonVariant::Primary(ButtonSize::Md)
                                        href=csv_href.get()
                                    >
                                        "CSVを取り込む"
                                    </LinkButton>
                                </div>
                            </EmptyState>
                        </div>
                    </Card>
                }
                    .into_any();
            }
            let query = search.with(|s| s.query.clone());
            let rows = filtered.get();
            // 銘柄コード検索時は上段の集計カードの代わりに
            // DividendInfo（embedded）を折り畳み式で出す
            if let Some(info) = dividend_info
                && !search_security_code(&rows, &query).is_empty()
            {
                let totals = dividend_totals(&rows);
                return view! {
                    <section id="receipts-summary">
                        <DividendSummarySection
                            store=info
                            totals=totals
                            expanded=summary_expanded
                            mobile_expanded=store.mobile_summary_expanded
                        />
                    </section>
                    <section id="receipts-list">
                        <ReceiptTable
                            tab=tab
                            rows=rows
                            all_rows=display
                            query=query
                            expanded_ids=store.expanded
                        />
                    </section>
                }
                .into_any();
            }
            let header = header_summary(
                tab,
                &ReceiptTabData {
                    rows: rows.clone(),
                    total: None,
                    summary: summary.clone(),
                    truncated: false,
                },
                &query,
            );
            view! {
                <section id="receipts-summary">
                    <SummaryStrip items=header expanded=store.mobile_summary_expanded />
                </section>
                <section id="receipts-list">
                    <ReceiptTable
                        tab=tab
                        rows=rows
                        all_rows=display
                        query=query
                        expanded_ids=store.expanded
                    />
                </section>
            }.into_any()
        }}
    }
}
