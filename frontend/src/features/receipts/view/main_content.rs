use crate::features::receipts::dividend_info::{
    DividendInfoStore, DividendSummarySection, search_security_code,
};
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::kind::dividend_totals;
use crate::features::receipts::view::ReceiptsSearchBar;
use crate::features::receipts::view::{ReceiptTable, SummaryStrip};
use crate::features::receipts::{
    ReceiptRow, ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState, truncated_list_warning,
};
use crate::session::use_session;
use crate::ui::button::{ButtonSize, ButtonVariant, LinkButton};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use crate::ui::labels;
use crate::ui::page_skeleton::{FilteredCount, SkeletonNote, SkeletonSearch, SkeletonSummary};
use crate::ui::state::{Alert, AlertVariant, ListLoadError};
use leptos::ev;
use leptos::prelude::*;

// 絞り込み結果から配当情報カードへ切り替える条件。検索語が銘柄に定まるときだけ置き換える
pub(crate) fn show_dividend_section(
    tab: ReceiptsTab,
    filtered: &[ReceiptRow],
    query: &str,
) -> bool {
    tab == ReceiptsTab::Dividend && !search_security_code(filtered, query).is_empty()
}

// 絞り込み中の件数。取得上限で行が欠けても API の total を優先する
pub(crate) fn derive_filtered_count(
    data: &ReceiptTabData,
    filtered_len: usize,
    filtering: bool,
) -> FilteredCount {
    FilteredCount {
        filtered: filtered_len,
        total: data.total.unwrap_or(data.rows.len()),
        filtering,
    }
}

#[component]
pub fn ReceiptsMainContent(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let search = store.search;
    let summary_expanded = RwSignal::new(true);

    let dividend_info = (tab == ReceiptsTab::Dividend).then(|| {
        let session = use_session();
        DividendInfoStore::new(session)
    });

    // タブの取得結果は1か所で受け、絞り込み結果は1つの Memo で共有する。
    // 判定・集計・期間・件数・表がそれぞれ複製・再実行しないようにする
    let tab_state = Memo::new(move |_| store.tab_state(tab));
    let filtered_rows = Memo::new(move |_| {
        let query = search.with(|state| state.query.clone());
        tab_state.with(|state| match state {
            TabState::Ready(data) => filter_receipts(tab, &data.rows, &query),
            _ => Vec::new(),
        })
    });

    if let Some(info) = dividend_info {
        Effect::new(move |_| {
            let authenticated = use_session().user.get().is_some();
            let generation = use_session().generation.get();
            let query = search.with(|state| state.query.clone());
            let code = filtered_rows.with(|rows| search_security_code(rows, &query));
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

    let csv_href = "/csv";

    let is_empty = Memo::new(move |_| {
        tab_state.with(|state| matches!(state, TabState::Ready(data) if data.rows.is_empty()))
    });
    let show_dividend_info = Memo::new(move |_| {
        dividend_info.is_some()
            && filtered_rows.with(|rows| {
                show_dividend_section(tab, rows, &search.with(|state| state.query.clone()))
            })
    });
    let dividend_totals_val = Memo::new(move |_| {
        if !show_dividend_info.with(|show| *show) {
            return None;
        }
        filtered_rows.with(|rows| Some(dividend_totals(rows)))
    });
    let header = Memo::new(move |_| {
        if show_dividend_info.with(|show| *show) {
            return None;
        }
        let query = search.with(|state| state.query.clone());
        tab_state.with(|state| match state {
            TabState::Ready(data) => {
                Some(crate::features::receipts::view::summary::header_summary(
                    tab,
                    &ReceiptTabData {
                        rows: filtered_rows.with(|rows| rows.clone()),
                        total: None,
                        summary: data.summary.clone(),
                        truncated: false,
                    },
                    &query,
                ))
            }
            _ => None,
        })
    });
    let all_rows = Memo::new(move |_| {
        tab_state.with(|state| match state {
            TabState::Ready(data) => data.rows.clone(),
            _ => Vec::new(),
        })
    });
    // 絞り込み中の件数。検索条件が無いときは注記に出さない
    let filtered_count = Memo::new(move |_| {
        let filtering = search.with(|state| state.has_any_filter());
        tab_state.with(|state| match state {
            TabState::Ready(data) => {
                derive_filtered_count(data, filtered_rows.with(|rows| rows.len()), filtering)
            }
            _ => FilteredCount {
                filtered: 0,
                total: 0,
                filtering,
            },
        })
    });
    let basis = vec![String::from(labels::SUMMARY_SAME_TARGET)];
    // 対象期間の注記。絞り込み後の明細の最初と最後の日を出す
    let period = Memo::new(move |_| {
        filtered_rows.with(|rows| {
            crate::features::receipts::view::summary::period_note(rows)
                .into_iter()
                .collect::<Vec<String>>()
        })
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
        {move || {
            let truncated = tab_state
                .with(|state| matches!(state, TabState::Ready(data) if data.truncated));
            if truncated {
                view! { <Alert variant=AlertVariant::Warning>{truncated_list_warning()}</Alert> }
                    .into_any()
            } else {
                ().into_any()
            }
        }}
        // 帯と注記。読み込み中や空のときは出さない
        {move || {
            let ready =
                tab_state.with(|state| matches!(state, TabState::Ready(_))) && !is_empty.get();
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
                <SkeletonNote notes=period.get() filtered=filtered_count.into() basis=basis.clone() />
            }
                .into_any()
        }}
        // 検索欄は帯と表の間に固定で置く(読み込み中や空でも操作できるようにする)
        <SkeletonSearch>
            <div class="hidden lg:block" role="search" aria-label="取引明細の検索">
                <ReceiptsSearchBar store=store tab=Signal::stored(tab) />
            </div>
        </SkeletonSearch>
        {move || {
            tab_state.with(|state| match state {
                TabState::Failed(message) => {
                    let retry_store = store;
                    let message = message.clone();
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
                            href=csv_href
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
                            all_rows=all_rows.get()
                            query=search.with(|s| s.selected_queries.securities.clone())
                            expanded_ids=store.expanded
                            compact_mobile=true
                        />
                    </section>
                }
                    .into_any()
            }
            TabState::Loading => view! { <div class="py-12 text-center text-sm text-text-muted" data-testid="receipt-loading">"読み込み中..."</div> }.into_any(),
        })}}
    }
}
