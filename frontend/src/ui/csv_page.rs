use crate::features::asset_balance::AssetBalanceCsvStore;
use crate::features::csv_page::CsvKind;
use crate::features::csv_sync::{CsvSyncStore, KindStatus};
use crate::features::receipts::{ReceiptsStore, ReceiptsTab, TabState};
use crate::session::use_session;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::format::{days_since_jst_date, format_iso_date, format_jst_time, format_short_date};
use crate::ui::labels;
use leptos::ev;
use leptos::html;
use leptos::portal::Portal;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// 種類ごとの行データ。統計が未取得なら count は None
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CsvKindRowData {
    pub kind: CsvKind,
    pub count: Option<i64>,
    pub latest_imported_at: Option<String>,
    /// csv_syncの状態
    pub sync_status: Option<KindStatus>,
}

impl CsvKindRowData {
    fn saved_count(&self) -> usize {
        self.count.unwrap_or(0).max(0) as usize
    }
}

/// CSV 管理画面の取得カード
#[component]
pub(crate) fn CsvManageFetchCard(
    running: ReadSignal<bool>,
    last_fetched: ReadSignal<Option<String>>,
    on_fetch: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    let session = use_session();
    view! {
        <section class="rounded-lg border border-border bg-surface-sunken p-4">
            <div class="flex flex-col sm:flex-row sm:flex-wrap sm:items-center sm:justify-between gap-4">
                <div>
                    <h2 class="text-sm font-bold text-text">{labels::CSV_MANAGE_FETCH_TITLE}</h2>
                </div>
                <div class="csv-manage-fetch-actions">
                    {move || {
                        session.user.get().is_some().then(|| {
                            view! {
                        <Button
                            variant=ButtonVariant::Primary(ButtonSize::Md)
                            testid="csv-fetch-button"
                            disabled=move || running.get()
                            on_click=move |ev| on_fetch.run(ev)
                            class="w-full"
                        >
                            {move || {
                                if running.get() {
                                    labels::CSV_MANAGE_FETCH_RUNNING
                                } else {
                                    labels::CSV_MANAGE_FETCH_BUTTON
                                }
                            }}
                        </Button>
                            }
                        })
                    }}
                    <p class="text-sm text-text-muted whitespace-nowrap" data-testid="csv-last-fetched">
                        {move || last_fetched.get().unwrap_or_else(|| labels::CSV_MANAGE_LAST_FETCHED_MISSING.to_string())}
                    </p>
                </div>
            </div>
        </section>
    }
}

/// 表側の「開いている種類」を、1つの種類の開閉フラグとして読み書きする。
/// 件数・状態の更新で行が作り直されても、開閉の状態が残る
#[derive(Clone, Copy)]
struct KindFlag {
    slot: RwSignal<Option<CsvKind>>,
    kind: CsvKind,
}

impl KindFlag {
    fn get(self) -> bool {
        self.slot.get() == Some(self.kind)
    }

    fn get_untracked(self) -> bool {
        self.slot.get_untracked() == Some(self.kind)
    }

    fn set(self, open: bool) {
        if open {
            self.slot.set(Some(self.kind));
        } else if self.slot.get_untracked() == Some(self.kind) {
            self.slot.set(None);
        }
    }

    fn toggle(self) {
        self.set(self.slot.get_untracked() != Some(self.kind));
    }
}

fn receipts_tab_of(kind: CsvKind) -> Option<ReceiptsTab> {
    kind.receipt_tab()
}

/// 種類ごとのテーブル行。⋯ のメニューから再取得・取り込み・削除を選ぶ
#[component]
fn CsvKindRow(
    row_data: CsvKindRowData,
    asset_csv: AssetBalanceCsvStore,
    receipts_store: ReceiptsStore,
    csv_sync: CsvSyncStore,
    menu_kind: RwSignal<Option<CsvKind>>,
    delete_kind: RwSignal<Option<CsvKind>>,
    detail_kind: RwSignal<Option<CsvKind>>,
    on_stats_changed: Callback<()>,
) -> impl IntoView {
    let kind = row_data.kind;
    let count = row_data.count;
    let saved_count = row_data.saved_count();
    let count_ready = count.is_some();
    let latest_imported_at = row_data.latest_imported_at;
    let sync_status = row_data.sync_status;

    let (status_text, status_class, has_error_detail, error_detail, status_key) =
        compute_status(kind, &sync_status, &latest_imported_at, count_ready);
    let row_failed = status_key == "failed";

    let tab = receipts_tab_of(kind);
    let meta = Memo::new(move |_| match kind {
        CsvKind::AssetStock => asset_csv.csv_meta(),
        CsvKind::AssetFund => asset_csv.fund_csv_meta(),
        _ => tab
            .map(|tab| receipts_store.csv_meta(tab))
            .unwrap_or_default(),
    });
    let deleting = Memo::new(move |_| meta.get().deleting);
    let delete_count = Memo::new(move |_| match kind {
        CsvKind::AssetStock => asset_csv.db_count(),
        CsvKind::AssetFund => asset_csv.fund_db_count(),
        _ => tab.map(|tab| receipts_store.count(tab)).unwrap_or(0),
    });
    let delete_count_ready = Memo::new(move |_| match kind {
        CsvKind::AssetStock => asset_csv.saved_count_ready(),
        CsvKind::AssetFund => asset_csv.fund_saved_count_ready(),
        _ => tab.is_some_and(|tab| {
            matches!(receipts_store.tab_state(tab), TabState::Ready(_))
                && receipts_store.refresh_error(tab).is_none()
        }),
    });
    let delete_count_error = Memo::new(move |_| match kind {
        CsvKind::AssetStock => asset_csv.saved_count_error(),
        CsvKind::AssetFund => asset_csv.fund_saved_count_error(),
        _ => tab.and_then(|tab| match receipts_store.tab_state(tab) {
            TabState::Failed(message) => Some(message),
            TabState::Loading => None,
            _ => receipts_store.refresh_error(tab),
        }),
    });
    let date_range = Memo::new(move |_| match kind {
        CsvKind::AssetStock => asset_csv.date_range(),
        CsvKind::AssetFund => asset_csv.fund_date_range(),
        _ => tab.and_then(|tab| {
            if let TabState::Ready(data) = receipts_store.tab_state(tab) {
                let dates: Vec<&str> = data.rows.iter().map(|row| row.date()).collect();
                if dates.is_empty() {
                    return None;
                }
                let min = dates.iter().min()?;
                let max = dates.iter().max()?;
                Some((format_iso_date(min), format_iso_date(max)))
            } else {
                None
            }
        }),
    });
    let delete_disabled = Memo::new(move |_| {
        let meta = meta.get();
        if meta.deleting {
            return true;
        }
        match kind {
            CsvKind::AssetStock => asset_csv.list_loading(),
            CsvKind::AssetFund => asset_csv.fund_list_loading(),
            _ => receipts_store.any_tab_fetching(),
        }
    });

    let menu_open = KindFlag {
        slot: menu_kind,
        kind,
    };
    let show_delete_confirm = KindFlag {
        slot: delete_kind,
        kind,
    };
    let delete_error = RwSignal::new(None::<String>);
    let detail_open = KindFlag {
        slot: detail_kind,
        kind,
    };
    let menu_container = NodeRef::<html::Div>::new();
    let menu_trigger = NodeRef::<html::Button>::new();
    let menu_list = NodeRef::<html::Div>::new();
    let file_input = NodeRef::<html::Input>::new();
    let input_id = format!("csv-file-input-{}", kind.kind_param());

    // 取り込みが終わったら件数を取り直す。確認待ちは回答まで待つ
    let sync_kind = kind.sync_kind();
    let live_status = Memo::new(move |_| {
        csv_sync
            .state()
            .kinds
            .iter()
            .find(|(candidate, _)| *candidate == sync_kind)
            .map(|(_, status)| status.clone())
    });
    let was_importing = RwSignal::new(false);
    Effect::new(move |_| match live_status.get() {
        Some(KindStatus::Importing) | Some(KindStatus::Confirming) => {
            was_importing.set(true);
        }
        Some(KindStatus::Imported { .. })
        | Some(KindStatus::Skipped { .. })
        | Some(KindStatus::Failed { .. })
            if was_importing.get_untracked() =>
        {
            was_importing.set(false);
            on_stats_changed.run(());
        }
        _ => {}
    });

    let focus_trigger = move || {
        if let Some(trigger) = menu_trigger.get_untracked() {
            let _ = trigger.focus();
        }
    };

    // 開いたメニューの外側を押したら閉じる。別の行の⋯はその行の容器の内側なので、
    // 先にこちらが閉じてから向こうの切り替え処理が開く(同時に2つ開かない)
    let on_pointer_down = window_event_listener(ev::pointerdown, move |ev| {
        if !menu_open.get_untracked() {
            return;
        }
        let inside = menu_container
            .get_untracked()
            .zip(
                ev.target()
                    .and_then(|target| target.dyn_into::<web_sys::Node>().ok()),
            )
            .is_some_and(|(container, target)| container.contains(Some(&target)));
        if !inside {
            menu_open.set(false);
        }
    });

    let on_key_down = window_event_listener(ev::keydown, move |ev| {
        if !menu_open.get_untracked() {
            return;
        }
        if ev.key() == "Escape" {
            menu_open.set(false);
            focus_trigger();
        }
    });

    on_cleanup(move || {
        on_pointer_down.remove();
        on_key_down.remove();
    });

    // 開いたら最初の項目へフォーカスを移す。利用者が既に別の場所へ移した
    // フォーカスは奪わない(別の行へ切り替えた直後など)
    Effect::new(move |_| {
        if menu_open.get() {
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(0).await;
                if !menu_open.get_untracked() {
                    return;
                }
                let on_trigger = menu_trigger.get_untracked().is_some_and(|trigger| {
                    web_sys::window()
                        .and_then(|window| window.document())
                        .and_then(|document| document.active_element())
                        .is_some_and(|active| active.is_same_node(Some(&trigger)))
                });
                if on_trigger
                    && let Some(menu) = menu_list.get_untracked()
                    && let Ok(Some(first)) = menu.query_selector("button")
                    && let Ok(first) = first.dyn_into::<web_sys::HtmlElement>()
                {
                    let _ = first.focus();
                }
            });
        }
    });

    // 削除は確認表示中だけ開始できるため、モーダルを開く時点でフラグを立てる
    let open_delete = move |_| {
        menu_open.set(false);
        focus_trigger();
        delete_error.set(None);
        // 開き直すたびに件数を取り直す
        match kind {
            CsvKind::AssetStock => asset_csv.refresh_for_delete(),
            CsvKind::AssetFund => asset_csv.refresh_fund_for_delete(),
            _ => {
                if let Some(tab) = tab {
                    receipts_store.refresh_for_delete(tab);
                }
            }
        }
        match kind {
            CsvKind::AssetStock => asset_csv.open_delete_confirm(),
            CsvKind::AssetFund => asset_csv.fund_open_delete_confirm(),
            _ => {
                if let Some(tab) = tab {
                    receipts_store.open_delete_confirm(tab);
                }
            }
        }
        show_delete_confirm.set(true);
    };

    let confirm_delete = move || {
        delete_error.set(None);
        match kind {
            CsvKind::AssetStock => {
                asset_csv.open_delete_confirm();
                asset_csv.confirm_delete_all();
            }
            CsvKind::AssetFund => {
                asset_csv.fund_open_delete_confirm();
                asset_csv.fund_confirm_delete_all();
            }
            _ => {
                if let Some(tab) = tab {
                    receipts_store.open_delete_confirm(tab);
                    receipts_store.confirm_delete_all(tab);
                }
            }
        }
    };

    let cancel_delete = move || {
        if !deleting.get_untracked() {
            show_delete_confirm.set(false);
            delete_error.set(None);
        }
    };

    // 削除が終わったら成功時だけ閉じ、失敗理由はモーダルに残す。件数はどちらでも取り直す
    let was_deleting = RwSignal::new(false);
    Effect::new(move |_| {
        let deleting_now = meta.get().deleting;
        if deleting_now {
            was_deleting.set(true);
            return;
        }
        if !was_deleting.get_untracked() || !show_delete_confirm.get() {
            return;
        }
        was_deleting.set(false);
        on_stats_changed.run(());
        match meta.get().error {
            Some(message) => delete_error.set(Some(message)),
            None => {
                show_delete_confirm.set(false);
                delete_error.set(None);
            }
        }
    });

    let on_refetch = move |_| {
        menu_open.set(false);
        focus_trigger();
        csv_sync.start_kind(kind.sync_kind());
    };

    let on_file_import = move |_| {
        menu_open.set(false);
        focus_trigger();
        if let Some(input) = file_input.get_untracked() {
            input.click();
        }
    };

    let on_file_change = move |ev: web_sys::Event| {
        let Some(input) = file_input.get_untracked() else {
            return;
        };
        let file = ev
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            .and_then(|element| element.files())
            .and_then(|files| files.get(0));
        input.set_value("");
        if let Some(file) = file {
            csv_sync.import_file(kind.sync_kind(), file);
        }
    };

    view! {
        <tr
            class=move || {
                if row_failed {
                    "csv-kind-row csv-kind-row-failed border-t border-border-subtle"
                } else {
                    "csv-kind-row border-t border-border-subtle"
                }
            }
            data-kind=kind.kind_param()
        >
            <td class="px-4 py-3 whitespace-nowrap max-sm:col-start-1 max-sm:row-start-1 max-sm:pb-0">
                <span class="text-text font-medium">{kind.label()}</span>
            </td>
            <td
                class="px-4 py-3 whitespace-nowrap text-right text-text-muted max-sm:col-start-2 max-sm:row-span-2 max-sm:row-start-1 max-sm:pr-2"
                data-testid="saved-count"
            >
                {match count {
                    Some(count) => labels::count(count.max(0) as usize),
                    None => "—".to_string(),
                }}
            </td>
            <td class="px-4 py-3 whitespace-nowrap max-sm:col-start-1 max-sm:row-start-2 max-sm:pt-0 max-sm:text-xs">
                <div class="flex items-center gap-2">
                    <span class="inline-flex items-center gap-1.5">
                        {status_dot_class(status_key)
                            .map(|dot| {
                                view! { <span class=dot aria-hidden="true"></span> }
                            })}
                        <span class=status_class data-status=status_key>{status_text}</span>
                    </span>
                    {has_error_detail.then(|| {
                        view! {
                            <button
                                type="button"
                                class="text-xs underline text-text-muted hover:text-text"
                                data-testid="status-detail"
                                on:click=move |_| detail_open.toggle()
                            >
                                {labels::CSV_MANAGE_FAILED_DETAIL_LABEL}
                            </button>
                        }
                    })}
                </div>
                {move || {
                    (has_error_detail && detail_open.get())
                        .then(|| {
                            view! {
                                <div
                                    class="mt-1 max-w-xs overflow-x-auto whitespace-nowrap text-xs text-negative-vivid"
                                    data-testid="status-detail-content"
                                    title=error_detail.clone()
                                >
                                    {error_detail.clone()}
                                </div>
                            }
                        })
                }}
            </td>
            <td class="px-4 py-3 whitespace-nowrap text-right max-sm:col-start-3 max-sm:row-span-2 max-sm:row-start-1">
                <div class="relative inline-block" node_ref=menu_container>
                    <Button
                        variant=ButtonVariant::Secondary(ButtonSize::Sm)
                        class="inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-surface-sunken text-text-muted"
                        aria_label="メニュー"
                        aria_expanded=move || menu_open.get()
                        on_click=move |_| menu_open.toggle()
                        node_ref=menu_trigger
                    >
                        <span class="text-lg font-mono">"⋯"</span>
                    </Button>
                    {move || menu_open.get().then(|| {
                        view! {
                            <div
                                node_ref=menu_list
                                class="absolute right-0 z-40 mt-1 w-56 overflow-hidden rounded-lg border border-border bg-surface p-1 shadow-lg"
                            >
                                <button
                                    type="button"
                                    class="w-full flex items-center gap-2 px-3 py-2 text-sm font-medium text-text-deep hover:bg-surface-raised rounded text-left"
                                    on:click=on_refetch
                                >
                                    <svg class="shrink-0 w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/></svg>
                                    {labels::CSV_MANAGE_MENU_REFETCH}
                                </button>
                                <button
                                    type="button"
                                    class="w-full flex items-center gap-2 px-3 py-2 text-sm font-medium text-text-deep hover:bg-surface-raised rounded text-left"
                                    on:click=on_file_import
                                >
                                    <svg class="shrink-0 w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M12 12v6m0-6l-2.5 2.5M12 12l2.5 2.5"/></svg>
                                    {labels::CSV_MANAGE_MENU_FILE_IMPORT}
                                </button>
                                <hr class="border-border my-1" />
                                <button
                                    type="button"
                                    class="w-full flex items-center gap-2 px-3 py-2 text-sm font-medium text-negative hover:bg-negative/10 rounded text-left disabled:opacity-50 disabled:hover:bg-transparent"
                                    on:click=open_delete
                                    disabled=move || {
                                        !count_ready || saved_count == 0 || delete_disabled.get()
                                    }
                                >
                                    <svg class="shrink-0 w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/></svg>
                                    {labels::CSV_MANAGE_MENU_DELETE}
                                </button>
                            </div>
                        }
                    })}
                </div>
                <input
                    type="file"
                    accept=".csv"
                    class="sr-only"
                    id=input_id.clone()
                    data-testid="csv-file-input"
                    aria-label="CSVファイルを選択"
                    node_ref=file_input
                    on:change=on_file_change
                />
            </td>
        </tr>
        {move || show_delete_confirm.get().then(|| {
            let count = delete_count.get();
            let range = date_range.get().map(|(min, max)| format!("{min} 〜 {max}"));
            view! {
                <Portal>
                <ConfirmDeleteModal
                    title=labels::csv_manage_delete_confirm_title(kind.label())
                    description=labels::csv_manage_delete_confirm_description(
                        kind.label(),
                        range.as_deref(),
                        count,
                    )
                    item_count=count
                    confirm_label=labels::csv_manage_delete_confirm_button(count)
                    loading=deleting
                    confirm_ready=delete_count_ready
                    error=Memo::new(move |_| {
                        delete_error.get().or_else(|| delete_count_error.get())
                    })
                    on_confirm=confirm_delete
                    on_cancel=cancel_delete
                />
                </Portal>
            }
        })}
    }
}

/// 状態テキストとクラスを計算する。件数が未取得の間は状態を出さない。
/// 5つ目の戻り値は `data-status` に出す意味的な状態(緑か黄かの分岐を確かめる用)
fn compute_status(
    kind: CsvKind,
    sync_status: &Option<KindStatus>,
    latest_imported_at: &Option<String>,
    count_ready: bool,
) -> (String, String, bool, String, &'static str) {
    // 取得中・取り込み中の状態を最優先
    if let Some(status) = sync_status {
        match status {
            KindStatus::Fetching => {
                return (
                    "取得中...".to_string(),
                    "text-accent-vivid".to_string(),
                    false,
                    String::new(),
                    "fetching",
                );
            }
            KindStatus::Importing => {
                return (
                    "取り込み中...".to_string(),
                    "text-accent-vivid".to_string(),
                    false,
                    String::new(),
                    "importing",
                );
            }
            KindStatus::Confirming => {
                return (
                    "確認待ち".to_string(),
                    "text-accent-vivid".to_string(),
                    false,
                    String::new(),
                    "confirming",
                );
            }
            KindStatus::Failed { reason } => {
                return (
                    labels::CSV_MANAGE_STATUS_FAILED.to_string(),
                    "text-negative-vivid".to_string(),
                    true,
                    reason.clone(),
                    "failed",
                );
            }
            KindStatus::Skipped { reason } => {
                return (
                    labels::csv_manage_status_skipped(reason.label()),
                    "text-text-muted".to_string(),
                    false,
                    String::new(),
                    "skipped",
                );
            }
            // 成功時は最終取込日時で判断する
            KindStatus::Imported { .. } | KindStatus::Done { .. } | KindStatus::Waiting => {}
        }
    }

    if !count_ready {
        return (
            "—".to_string(),
            "text-text-muted".to_string(),
            false,
            String::new(),
            "unknown",
        );
    }

    // 最終取込日時から状態を判定する。時刻は日本時間で出す
    if let Some(ts) = latest_imported_at
        && let Some(days_ago) = days_since_jst_date(ts)
    {
        let (text, class, key) =
            dated_status(kind, days_ago, &format_short_date(ts), &format_jst_time(ts));
        return (text, class.to_string(), false, String::new(), key);
    }

    (
        labels::CSV_MANAGE_STATUS_NEVER.to_string(),
        "text-text-muted".to_string(),
        false,
        String::new(),
        "never",
    )
}

/// 取込日からの経過日数による状態。残高は当日だけ緑、明細は7日以内が緑
fn dated_status(
    kind: CsvKind,
    days_ago: i64,
    date: &str,
    time: &str,
) -> (String, &'static str, &'static str) {
    if days_ago <= 0 {
        (
            labels::csv_manage_status_updated_today(time),
            "text-positive-vivid",
            "fresh",
        )
    } else if kind.is_fresh(days_ago) {
        (
            labels::csv_manage_status_updated_days_ago(days_ago, date, time),
            "text-positive-vivid",
            "fresh",
        )
    } else {
        (
            labels::csv_manage_status_updated_days_ago(days_ago, date, time),
            "text-warn-ink",
            "stale",
        )
    }
}

/// 状態の塗り丸の色。未取得・件数未取得は点を出さない
fn status_dot_class(status_key: &str) -> Option<&'static str> {
    match status_key {
        "fresh" => Some("inline-block h-2.5 w-2.5 shrink-0 rounded-full bg-positive-vivid"),
        "stale" => Some("inline-block h-2.5 w-2.5 shrink-0 rounded-full bg-warn-ink"),
        "failed" => Some("inline-block h-2.5 w-2.5 shrink-0 rounded-full bg-negative-vivid"),
        "fetching" | "importing" | "confirming" => {
            Some("inline-block h-2.5 w-2.5 shrink-0 rounded-full bg-accent-vivid")
        }
        _ => None,
    }
}

/// 種類ごとのテーブル
#[component]
pub(crate) fn CsvKindTable(
    rows: Memo<Vec<CsvKindRowData>>,
    asset_csv: AssetBalanceCsvStore,
    receipts_store: ReceiptsStore,
    csv_sync: CsvSyncStore,
    on_stats_changed: Callback<()>,
) -> impl IntoView {
    // 件数・状態が更新されると行が作り直されるため、開いているメニューと削除確認は表で持つ
    let menu_kind = RwSignal::new(None::<CsvKind>);
    let delete_kind = RwSignal::new(None::<CsvKind>);
    let detail_kind = RwSignal::new(None::<CsvKind>);
    view! {
        <div>
            <table class="csv-kind-table w-full" role="table">
                <colgroup class="max-sm:hidden">
                    <col class="csv-kind-col-kind" />
                    <col class="csv-kind-col-count" />
                    <col />
                    <col class="csv-kind-col-menu" />
                </colgroup>
                <thead class="bg-surface-sunken border-b border-border max-sm:hidden">
                    <tr>
                        <th class="px-4 py-3 text-left text-xs font-bold text-text-muted uppercase tracking-wider whitespace-nowrap">
                            {labels::CSV_MANAGE_COL_KIND}
                        </th>
                        <th class="px-4 py-3 text-right text-xs font-bold text-text-muted uppercase tracking-wider whitespace-nowrap">
                            {labels::CSV_MANAGE_COL_COUNT}
                        </th>
                        <th class="px-4 py-3 text-left text-xs font-bold text-text-muted uppercase tracking-wider whitespace-nowrap">
                            {labels::CSV_MANAGE_COL_STATUS}
                        </th>
                        <th class="px-4 py-3 text-right text-xs font-bold text-text-muted uppercase tracking-wider whitespace-nowrap">
                            {labels::CSV_MANAGE_COL_MENU}
                        </th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-border-subtle">
                    {move || {
                        rows
                            .get()
                            .into_iter()
                            .map(|row| {
                                view! {
                                    <CsvKindRow
                                        row_data=row
                                        asset_csv=asset_csv
                                        receipts_store=receipts_store
                                        csv_sync=csv_sync
                                        menu_kind=menu_kind
                                        delete_kind=delete_kind
                                        detail_kind=detail_kind
                                        on_stats_changed=on_stats_changed
                                    />
                                }
                            })
                            .collect_view()
                    }}
                </tbody>
            </table>
        </div>
    }
}

#[cfg(test)]
mod tests;
