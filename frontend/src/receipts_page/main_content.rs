use super::summary::{header_summary, SummaryStrip};
use super::table::ReceiptTable;
use super::workspace::empty_hint;
use crate::dividend_info::{search_security_code, DividendInfoStore, DividendSummarySection};
use crate::dto::Dividend;
use crate::receipts::{ReceiptItem, ReceiptTabData, ReceiptsStore, ReceiptsTab};
use crate::receipts_csv::CsvPreviewRow;
use crate::receipts_domain::calculate_dividends;
use crate::receipts_filter::filter_receipts;
use crate::session::use_session;
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
    let display_store = store.clone();
    let display_rows = Memo::new(move |_| match display_store.csv_state(tab).preview {
        Some(preview) if !preview.rows.is_empty() => preview
            .rows
            .iter()
            .map(CsvPreviewRow::to_receipt_item)
            .collect(),
        _ => data.rows.clone(),
    });
    let preview_store = store.clone();
    let preview_active = Memo::new(move |_| preview_store.has_csv_preview(tab));
    let filtered =
        Memo::new(move |_| filter_receipts(tab, &display_rows.get(), &search.get().query));
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
            let dividends: Vec<Dividend> = rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::Dividend(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let code = search_security_code(&dividends, &query);
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
    view! {
        {move || {
            let display = display_rows.get();
            if display.is_empty() {
                return view! {
                    <div
                        class="overflow-hidden rounded-xl border border-slate-950/10 bg-white/95 shadow-[0_16px_44px_-36px_rgba(15,23,42,0.9)]"
                        data-testid="receipt-card"
                    >
                        <div class="p-0" data-testid="receipt-card-body">
                            <div class="empty-state">
                                <h3 class="text-base font-black text-slate-950">
                                    "データがありません"
                                </h3>
                                <p class="mt-1.5 max-w-md text-sm font-medium text-slate-600">
                                    {empty_hint(tab)}
                                </p>
                            </div>
                        </div>
                    </div>
                }
                    .into_any();
            }
            let query = search.with(|s| s.query.clone());
            let rows = filtered.get();
            // 銘柄コード検索時は上段の集計カードの代わりに
            // React の DividendInfo（embedded）を折り畳み式で出す
            if let Some(info) = dividend_info {
                let dividends: Vec<Dividend> = rows
                    .iter()
                    .filter_map(|item| match item {
                        ReceiptItem::Dividend(row) => Some(row.clone()),
                        _ => None,
                    })
                    .collect();
                if !search_security_code(&dividends, &query).is_empty() {
                    let totals = calculate_dividends(&dividends);
                    return view! {
                        <DividendSummarySection
                            store=info
                            totals=totals
                            expanded=summary_expanded
                            mobile_expanded=store.mobile_summary_expanded
                        />
                        <ReceiptTable
                            tab=tab
                            rows=rows
                            all_rows=display
                            query=query
                            expanded_ids=store.expanded
                        />
                    }
                    .into_any();
                }
            }
            let header = header_summary(
                tab,
                &ReceiptTabData {
                    rows: rows.clone(),
                    summary: summary.clone(),
                    truncated: false,
                },
                &query,
                preview_active.get(),
            );
            view! {
                <SummaryStrip items=header expanded=store.mobile_summary_expanded />
                <ReceiptTable
                    tab=tab
                    rows=rows
                    all_rows=display
                    query=query
                    expanded_ids=store.expanded
                />
            }.into_any()
        }}
    }
}
