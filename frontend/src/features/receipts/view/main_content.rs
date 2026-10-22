use crate::features::receipts::dividend_info::{
    DividendInfoStore, DividendSummarySection, search_security_code,
};
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::kind::dividend_totals;
use crate::features::receipts::view::ReceiptsSearchBar;
use crate::features::receipts::view::{ReceiptTable, SummaryStrip};
use crate::features::receipts::{
    ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState, truncated_list_warning,
};
use crate::session::use_session;
use crate::ui::button::{ButtonSize, ButtonVariant, LinkButton};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use crate::ui::labels;
use crate::ui::page_skeleton::{FilteredCount, SkeletonNote, SkeletonSearch, SkeletonSummary};
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
    // 絞り込み中の件数。検索条件が無いときは注記に出さない
    let filtered_count = Signal::derive(move || {
        let (filtered, total) = match store.tab_state(tab) {
            TabState::Ready(data) => (
                filtered_rows.get().len(),
                data.total.unwrap_or(data.rows.len()),
            ),
            _ => (0, 0),
        };
        FilteredCount {
            filtered,
            total,
            filtering: !search.with(|state| state.query.is_empty()),
        }
    });
    let basis = vec![String::from(labels::SUMMARY_SAME_TARGET)];
    // 対象期間の注記。絞り込み後の明細の最初と最後の日を出す
    let period = Memo::new(move |_| {
        crate::features::receipts::view::summary::period_note(&filtered_rows.get())
            .into_iter()
            .collect::<Vec<String>>()
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
        // 帯と注記。読み込み中や空のときは出さない
        {move || {
            let ready = matches!(store.tab_state(tab), TabState::Ready(_)) && !is_empty.get();
            if !ready {
                return ().into_any();
            }
            view! {
                <div class="receipts-sticky-top">
                    <SkeletonSummary>
                        <section id="receipts-summary">
                            {move || {
                                // 銘柄を指定したときは配当情報のカードに置き換える
                                if show_dividend_info.get() {
                                    view! {
                                        <DividendSummarySection
                                            store=dividend_info.unwrap()
                                            totals=dividend_totals_val.get().unwrap()
                                            expanded=summary_expanded
                                            mobile_expanded=store.mobile_summary_expanded
                                        />
                                    }
                                        .into_any()
                                } else {
                                    header
                                        .get()
                                        .map(|items| view! { <SummaryStrip items=items /> })
                                        .into_any()
                                }
                            }}
                        </section>
                    </SkeletonSummary>
                </div>
                <SkeletonNote notes=period.get() filtered=filtered_count basis=basis.clone() />
            }
                .into_any()
        }}
        // 検索欄は帯と表の間に固定で置く(読み込み中や空でも操作できるようにする)
        <SkeletonSearch>
            <div class="hidden lg:block" role="search" aria-label="取引明細の検索">
                <ReceiptsSearchBar store=store tab=tab />
            </div>
        </SkeletonSearch>
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
                    <EmptyState
                        title="データがありません"
                        description=tab.empty_hint()
                        icon=EmptyStateIcon::Tray
                    >
                        <LinkButton
                            variant=ButtonVariant::Primary(ButtonSize::Md)
                            href=csv_href.get()
                        >
                            "CSVを取り込む"
                        </LinkButton>
                    </EmptyState>
                }
                    .into_any()
            }
            TabState::Ready(_) => {
                view! {
                    <section id="receipts-list">
                        <ReceiptTable
                            tab=tab
                            rows=filtered_rows.get()
                            all_rows=display_rows.get()
                            query=search.with(|s| s.selected_queries.securities.clone())
                            expanded_ids=store.expanded
                            compact_mobile=true
                        />
                    </section>
                }
                    .into_any()
            }
            TabState::Loading => view! { <div class="py-12 text-center text-sm text-text-muted" data-testid="receipt-loading">"読み込み中..."</div> }.into_any(),
        }}
    }
}
