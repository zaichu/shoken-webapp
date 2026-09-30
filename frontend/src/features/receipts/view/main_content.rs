use super::summary::{header_summary, SummaryStrip};
use super::table::ReceiptTable;
use super::workspace::utility_rail_id;
use crate::features::receipts::dividend_info::{
    search_security_code, DividendInfoStore, DividendSummarySection,
};
use crate::features::receipts::filter::filter_receipts;
use crate::features::receipts::kind::{DividendKind, ReceiptKind};
use crate::features::receipts::{ReceiptRow, ReceiptTabData, ReceiptsStore, ReceiptsTab};
use crate::session::use_session;
use crate::support::row::Row;
use crate::ui::badge::{Badge, BadgeVariant};
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::csv_preview::CsvPreviewBanner;
use crate::ui::csv_section::click_csv_input;
use crate::ui::disclosure::{DisclosureStyle, DisclosureToggle};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use leptos::ev;
use leptos::prelude::*;

#[component]
pub(crate) fn ReceiptsMainContent(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    data: ReceiptTabData,
    // 取得失敗タブではレールが表示上強制的に開くため、開閉状態と齟齬するトグルは出さない
    rail_toggle: bool,
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
        {rail_toggle.then(|| {
            view! {
                <UtilityRailToggle
                    store=store
                    tab=tab
                    filtered=filtered
                    total=Signal::derive(move || display_rows.get().len())
                />
            }
        })}
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
            if let Some(info) = dividend_info {
                if !search_security_code(&rows, &query).is_empty() {
                    let totals = DividendKind::totals(&rows);
                    return view! {
                        {preview_active
                            .get()
                            .then(|| {
                                view! {
                                    <CsvPreviewBanner description="表と集計は取り込むファイルの内容です。保存するまで登録済みのデータは変わりません。" />
                                }
                            })}
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
                {preview.then(|| {
                    view! {
                        <CsvPreviewBanner description="表と集計は取り込むファイルの内容です。保存するまで登録済みのデータは変わりません。" />
                    }
                })}
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

/// PC のみ出す右レールの開閉バー。畳んでいても絞り込み中が分かるよう件数を表の上に出す
#[component]
fn UtilityRailToggle(
    store: ReceiptsStore,
    tab: ReceiptsTab,
    filtered: Memo<Vec<ReceiptRow>>,
    #[prop(into)] total: Signal<usize>,
) -> impl IntoView {
    let rail_open = store.utility_rail_open;
    let search = store.search;
    let filtering = Signal::derive(move || !search.with(|state| state.is_default()));
    view! {
        <div class="hidden lg:block no-print">
            <Card variant=CardVariant::Collapsible testid="receipt-utility-toggle-bar">
                <DisclosureToggle
                    style=DisclosureStyle::Rail
                    expanded=Signal::derive(move || rail_open.get())
                    controls=utility_rail_id(tab)
                    aria_label=Signal::derive(move || {
                        if rail_open.get() {
                            "取り込み・検索パネルを閉じる".to_string()
                        } else {
                            "取り込み・検索パネルを開く".to_string()
                        }
                    })
                    testid="receipt-utility-toggle"
                    hint=true
                    on_toggle=move || store.toggle_utility_rail()
                >
                    <span class="flex min-w-0 items-center gap-2">
                        <span class="text-sm font-bold text-text">"取り込み・検索"</span>
                        {move || {
                            (!rail_open.get() && filtering.get()).then(|| {
                                view! {
                                    <Badge variant=BadgeVariant::Accent>
                                        {format!(
                                            "絞り込み中 {} / {} 件",
                                            filtered.get().len(),
                                            total.get(),
                                        )}
                                    </Badge>
                                }
                            })
                        }}
                    </span>
                </DisclosureToggle>
            </Card>
        </div>
    }
}
