use super::summary::{SummaryStrip, header_summary};
use super::table::ReceiptTable;
use crate::features::receipts::dividend_info::{
    DividendInfoStore, DividendSummarySection, search_security_code,
};
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::kind::dividend_totals;
use crate::features::receipts::{ReceiptRow, ReceiptTabData, ReceiptsStore, ReceiptsTab};
use crate::session::use_session;
use crate::support::row::Row;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::csv_preview::CsvPreviewBanner;
use crate::ui::csv_section::click_csv_input;
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use leptos::ev;
use leptos::prelude::*;

pub(crate) fn display_rows_for(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    rows: Vec<ReceiptRow>,
) -> Vec<ReceiptRow> {
    match store.csv_state(tab).preview {
        Some(preview) if !preview.rows.is_empty() => {
            preview.rows.into_iter().map(Row::Preview).collect()
        }
        _ => rows,
    }
}

fn preview_banner() -> impl IntoView {
    view! {
        <CsvPreviewBanner description="表と集計は取り込むファイルの内容です。保存するまで登録済みのデータは変わりません。" />
    }
}

#[component]
pub(crate) fn ReceiptsMainContent(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    data: ReceiptTabData,
) -> impl IntoView {
    let search = store.search;
    let summary = data.summary.clone();
    let display_store = store;
    let display_rows = Memo::new(move |_| display_rows_for(display_store, tab, data.rows.clone()));
    let preview_store = store;
    let preview_active = Memo::new(move |_| preview_store.has_csv_preview(tab));
    let csv_store = store;
    // CTA は隠しファイル入力を押すだけなので、入力が disabled の間はこちらも止める
    let csv_input_disabled = Memo::new(move |_| csv_store.csv_input_disabled(tab));
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
    view! {
        {move || {
            let display = display_rows.get();
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
                                    <Button
                                        variant=ButtonVariant::Primary(ButtonSize::Md)
                                        disabled=move || csv_input_disabled.get()
                                        aria_disabled=move || csv_input_disabled.get()
                                        on_click=move |_| click_csv_input(tab.csv_input_id())
                                    >
                                        "CSVを取り込む"
                                    </Button>
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
                    {preview_active.get().then(preview_banner)}
                    <section id="receipts-summary">
                        <DividendSummarySection
                            store=info
                            totals=totals
                            expanded=summary_expanded
                            mobile_expanded=store.mobile_summary_expanded
                            preview=preview_active.get()
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
                    summary: summary.clone(),
                    truncated: false,
                },
                &query,
                preview_active.get(),
            );
            let preview = preview_active.get();
            view! {
                {preview.then(preview_banner)}
                <section id="receipts-summary">
                    <SummaryStrip items=header expanded=store.mobile_summary_expanded preview=preview />
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
