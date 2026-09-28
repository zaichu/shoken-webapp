use super::summary::{header_summary, SummaryStrip};
use super::table::ReceiptTable;
use crate::features::receipts::dividend_info::{
    search_security_code, DividendInfoStore, DividendSummarySection,
};
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::kind::{DividendKind, ReceiptKind};
use crate::features::receipts::{ReceiptTabData, ReceiptsStore, ReceiptsTab};
use crate::session::use_session;
use crate::support::row::Row;
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::EmptyState;
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
    let display_store = store;
    let display_rows = Memo::new(move |_| match display_store.csv_state(tab).preview {
        Some(preview) if !preview.rows.is_empty() => {
            preview.rows.into_iter().map(Row::Preview).collect()
        }
        _ => data.rows.clone(),
    });
    let preview_store = store;
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
                            />
                        </div>
                    </Card>
                }
                    .into_any();
            }
            let query = search.with(|s| s.query.clone());
            let rows = filtered.get();
            // 銘柄コード検索時は上段の集計カードの代わりに
            // DividendInfo（embedded）を折り畳み式で出す
            if let Some(info) = dividend_info {
                if !search_security_code(&rows, &query).is_empty() {
                    let totals = DividendKind::totals(&rows);
                    return view! {
                        {preview_active.get().then(|| view! { <PreviewBanner /> })}
                        <DividendSummarySection
                            store=info
                            totals=totals
                            expanded=summary_expanded
                            mobile_expanded=store.mobile_summary_expanded
                            preview=preview_active.get()
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
            let preview = preview_active.get();
            view! {
                {preview.then(|| view! { <PreviewBanner /> })}
                <SummaryStrip items=header expanded=store.mobile_summary_expanded preview=preview />
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

#[component]
fn PreviewBanner() -> impl IntoView {
    view! {
        // 読み上げは既存のプレビュー通知(role="status")が担うので、帯は見た目だけにする
        <div
            data-testid="receipt-preview-banner"
            class="mb-3 rounded-lg border border-accent-border-strong bg-accent-soft px-4 py-2.5 text-sm text-accent-text"
        >
            <span class="font-bold">"プレビュー中(未保存)"</span>
            <span class="ml-2">"表と集計は取り込むファイルの内容です。保存するまで登録済みのデータは変わりません。"</span>
        </div>
    }
}
