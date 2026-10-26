use crate::api::dto::CsvUploadResponse;
use crate::features::asset_balance::{
    AssetBalanceCsvSource, AssetBalanceCsvStore, AssetKind, FundCsvPreview, StockCsvPreview,
};
use crate::features::csv_page::CsvKind;
use crate::features::csv_sync::{CsvSyncStore, KindStatus};
use crate::features::receipts::{ReceiptCsvSource, ReceiptsCsvPreview, ReceiptsStore, ReceiptsTab};
use crate::session::use_session;
use crate::support::csv_flow::{CsvTabMeta, CsvUploadResponseExt};
use crate::ui::button::{Button, ButtonSize, ButtonVariant, LinkButton};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::csv_preview::{
    CsvPreviewNotice, CsvRowErrorList, preview_notice_text, shared_message,
    should_show_preview_notice,
};
use crate::ui::format::{days_since_jst_date, format_jst_time, format_short_date};
use crate::ui::labels;
use crate::ui::state::{Alert, AlertVariant, LoadingStrip};
use leptos::ev;
use leptos::html;
use leptos::portal::Portal;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

// 全件置換ではエラー行を含む CSV を backend が拒否するため、有効行があっても保存を止める
pub(crate) fn save_disabled_by_preview(meta: &CsvTabMeta, errors_block_save: bool) -> bool {
    meta.busy()
        || !meta.has_preview_rows()
        || (errors_block_save
            && meta
                .preview
                .as_ref()
                .is_some_and(|preview| !preview.errors.is_empty()))
}

/// CSV の取り込み操作に必要なデータソース。取引明細(種別付き)と資産残高で同じ操作バーを
/// 共有するため、状態取得と操作をトレイトで受け取る。状態は行データを含まない CsvTabMeta
pub trait CsvSource: Copy + Send + Sync + 'static {
    fn input_id(&self) -> &'static str;
    /// 検証結果の通知で使う方式込みの動詞("追加で保存" / "全件置換で保存")
    fn save_action(&self) -> &'static str;
    /// 「保存方式」の短い表示名("追加" / "全件置換")
    fn save_mode_label(&self) -> &'static str;
    /// 操作バーの保存ボタンのラベル。プレビューが無い間は件数を言えないので None
    fn save_button_label(&self, valid_rows: Option<usize>) -> String {
        match valid_rows {
            Some(rows) => format!("{rows}件を保存する"),
            None => "保存する".to_string(),
        }
    }
    fn csv_meta(&self) -> CsvTabMeta;
    fn is_authenticated(&self) -> bool;
    // 未認証・取得中・CSV 処理中など、ファイル選択を受け付けられない状態の判定
    fn input_disabled(&self) -> bool;
    /// エラー付きプレビューでは保存を止めるか(全件置換など一部保存ができない方式)
    fn errors_block_save(&self) -> bool {
        false
    }
    // 検証前・検証失敗・全行エラー(有効行0)では保存を送らない
    fn save_disabled(&self, meta: &CsvTabMeta) -> bool {
        save_disabled_by_preview(meta, self.errors_block_save())
    }
    /// 操作バーに出す、想定するCSV形式の案内
    fn csv_format_hint(&self) -> &'static str;
    /// 保存済み全件数。削除確認で対象件数を示すために使う
    fn db_count(&self) -> usize;
    /// 保存済み全件数が確定したか(一覧が未取得・取得失敗なら false で削除を止める)
    fn delete_count_ready(&self) -> bool;
    /// 全件数が確定できない理由(一覧取得の失敗表示用)
    fn delete_count_error(&self) -> Option<String> {
        None
    }
    /// 保存済みデータの日付範囲(最小・最大)。取得済み行から算出し、未取得なら None
    fn date_range(&self) -> Option<(String, String)> {
        None
    }
    /// 削除件数の再取得を要求する(件数取得失敗時の再読み込み用)
    fn refresh_for_delete(&self) {}
    fn delete_disabled(&self, meta: &CsvTabMeta) -> bool {
        meta.saving || meta.deleting
    }
    fn select_file(&self, file: web_sys::File);
    fn save_csv(&self);
    fn open_delete_confirm(&self);
    fn confirm_delete_all(&self);
}

/// 行ごとの CsvSource。種類が動的に決まるため、どちらかの実装を持って操作を委譲する
#[derive(Clone, Copy)]
enum RowSource {
    Asset(AssetBalanceCsvSource),
    Receipt(ReceiptCsvSource),
}

impl RowSource {
    fn new(kind: CsvKind, asset_csv: AssetBalanceCsvStore, receipts_store: ReceiptsStore) -> Self {
        match kind {
            CsvKind::AssetStock => Self::Asset(AssetBalanceCsvSource {
                store: asset_csv,
                kind: AssetKind::Stock,
            }),
            CsvKind::AssetFund => Self::Asset(AssetBalanceCsvSource {
                store: asset_csv,
                kind: AssetKind::Fund,
            }),
            _ => Self::Receipt(ReceiptCsvSource {
                store: receipts_store,
                tab: kind.receipt_tab().unwrap_or(ReceiptsTab::Dividend),
            }),
        }
    }

    fn db_count(&self) -> usize {
        match self {
            Self::Asset(source) => source.db_count(),
            Self::Receipt(source) => source.db_count(),
        }
    }

    fn delete_count_ready(&self) -> bool {
        match self {
            Self::Asset(source) => source.delete_count_ready(),
            Self::Receipt(source) => source.delete_count_ready(),
        }
    }

    fn delete_count_error(&self) -> Option<String> {
        match self {
            Self::Asset(source) => source.delete_count_error(),
            Self::Receipt(source) => source.delete_count_error(),
        }
    }

    fn date_range(&self) -> Option<(String, String)> {
        match self {
            Self::Asset(source) => source.date_range(),
            Self::Receipt(source) => source.date_range(),
        }
    }

    fn delete_disabled(&self, meta: &CsvTabMeta) -> bool {
        match self {
            Self::Asset(source) => source.delete_disabled(meta),
            Self::Receipt(source) => source.delete_disabled(meta),
        }
    }

    fn refresh_for_delete(&self) {
        match self {
            Self::Asset(source) => source.refresh_for_delete(),
            Self::Receipt(source) => source.refresh_for_delete(),
        }
    }

    fn open_delete_confirm(&self) {
        match self {
            Self::Asset(source) => source.open_delete_confirm(),
            Self::Receipt(source) => source.open_delete_confirm(),
        }
    }

    fn confirm_delete_all(&self) {
        match self {
            Self::Asset(source) => source.confirm_delete_all(),
            Self::Receipt(source) => source.confirm_delete_all(),
        }
    }
}

/// 種類ごとの行データ。統計が未取得なら count は None
#[derive(Clone, Debug, PartialEq)]
pub struct CsvKindRowData {
    pub kind: CsvKind,
    pub count: Option<i64>,
    pub latest_imported_at: Option<String>,
    /// csv_syncの状態
    pub sync_status: Option<KindStatus>,
}

impl CsvKindRowData {
    pub fn saved_count(&self) -> usize {
        self.count.unwrap_or(0).max(0) as usize
    }
}

/// CSV 管理画面の取得カード
#[component]
pub fn CsvManageFetchCard(
    running: ReadSignal<bool>,
    last_fetched: ReadSignal<Option<String>>,
    on_fetch: Callback<web_sys::MouseEvent>,
) -> impl IntoView {
    let session = use_session();
    view! {
        <section class="rounded-lg border border-border bg-surface-sunken p-4">
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                <div>
                    <h2 class="text-sm font-bold text-text">{labels::CSV_MANAGE_FETCH_TITLE}</h2>
                    <p class="mt-0.5 text-xs text-text-soft">{labels::CSV_MANAGE_FETCH_HINT}</p>
                </div>
                <div class="flex flex-col sm:flex-row gap-3 items-start sm:items-center w-full sm:w-auto">
                    {move || {
                        session.user.get().is_some().then(|| {
                            view! {
                        <Button
                            variant=ButtonVariant::Primary(ButtonSize::Md)
                            testid="csv-fetch-button"
                            disabled=move || running.get()
                            on_click=move |ev| on_fetch.run(ev)
                            class="w-full sm:w-auto sm:min-w-44"
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
                    <div class="flex-1 min-w-0">
                        {move || {
                            last_fetched.get().map(|lf| {
                                view! {
                                    <p class="text-sm text-text-muted" role="status" aria-live="polite">
                                        {lf}
                                    </p>
                                }
                            })
                        }}
                    </div>
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

/// 種類ごとのテーブル行。⋯ のメニューから再取得・取り込み・削除を選ぶ
#[component]
pub fn CsvKindRow(
    row_data: CsvKindRowData,
    asset_csv: AssetBalanceCsvStore,
    receipts_store: ReceiptsStore,
    csv_sync: CsvSyncStore,
    open_import: RwSignal<Option<CsvKind>>,
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

    // 削除の対象件数と失敗理由は一覧ストア（保存済みデータ）から取る
    let source = RowSource::new(kind, asset_csv, receipts_store);
    let meta = Memo::new(move |_| match kind {
        CsvKind::AssetStock => asset_csv.csv_meta(),
        CsvKind::AssetFund => asset_csv.fund_csv_meta(),
        _ => kind
            .receipt_tab()
            .map(|tab| receipts_store.csv_meta(tab))
            .unwrap_or_default(),
    });
    let deleting = Memo::new(move |_| meta.get().deleting);
    let delete_count = Memo::new(move |_| source.db_count());
    let delete_count_ready = Memo::new(move |_| source.delete_count_ready());
    let delete_count_error = Memo::new(move |_| source.delete_count_error());
    let date_range = Memo::new(move |_| source.date_range());
    let delete_disabled = Memo::new(move |_| source.delete_disabled(&meta.get()));

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
        source.refresh_for_delete();
        source.open_delete_confirm();
        show_delete_confirm.set(true);
    };

    let confirm_delete = move || {
        delete_error.set(None);
        source.open_delete_confirm();
        source.confirm_delete_all();
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

    let importing = Memo::new(move |_| open_import.get() == Some(kind));

    let on_refetch = move |_| {
        menu_open.set(false);
        focus_trigger();
        csv_sync.start_kind(kind.sync_kind());
    };

    let on_file_import = move |_| {
        menu_open.set(false);
        focus_trigger();
        open_import.set(Some(kind));
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
                    <span class=status_class data-status=status_key>{status_text}</span>
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
            </td>
        </tr>
        {move || importing.get().then(|| {
            view! {
                <tr class="border-t border-border-subtle" data-testid="csv-import-row">
                    <td colspan="4" class="px-4 py-3">
                        <CsvImportPanel
                            kind=kind
                            asset_csv=asset_csv
                            receipts_store=receipts_store
                            on_saved=on_stats_changed
                            on_close=Callback::new(move |_| open_import.set(None))
                        />
                    </td>
                </tr>
            }
        })}
        {move || show_delete_confirm.get().then(|| {
            let count = delete_count.get();
            let range = date_range.get().map(|(min, max)| format!("{min} 〜 {max}"));
            view! {
                <Portal>
                <ConfirmDeleteModal
                    title=labels::csv_manage_delete_confirm_title(kind.delete_subject())
                    description=labels::csv_manage_delete_confirm_description(
                        kind.delete_subject(),
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

/// 種類ごとの取り込み欄。既存のファイル選択→プレビュー→保存をそのまま使う
#[component]
fn CsvImportPanel(
    kind: CsvKind,
    asset_csv: AssetBalanceCsvStore,
    receipts_store: ReceiptsStore,
    on_saved: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    let close = move |_| on_close.run(());
    view! {
        <div class="flex flex-col gap-2 rounded-lg border border-border bg-surface p-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <p class="text-sm font-bold text-text">
                    {kind.label()} " へファイルから取り込み"
                </p>
                <Button variant=ButtonVariant::Secondary(ButtonSize::Xs) on_click=close>
                    "閉じる"
                </Button>
            </div>
            {match kind {
                CsvKind::AssetStock => view! {
                    <CsvImportBody
                        source=AssetBalanceCsvSource { store: asset_csv, kind: AssetKind::Stock }
                        preview=view! { <StockCsvPreview store=asset_csv /> }.into_any()
                        list_href=kind.list_href()
                        on_saved=on_saved
                    />
                }
                    .into_any(),
                CsvKind::AssetFund => view! {
                    <CsvImportBody
                        source=AssetBalanceCsvSource { store: asset_csv, kind: AssetKind::Fund }
                        preview=view! { <FundCsvPreview store=asset_csv /> }.into_any()
                        list_href=kind.list_href()
                        on_saved=on_saved
                    />
                }
                    .into_any(),
                _ => {
                    let tab = kind.receipt_tab().unwrap_or(ReceiptsTab::Dividend);
                    view! {
                        <CsvImportBody
                            source=ReceiptCsvSource { store: receipts_store, tab }
                            preview=view! { <ReceiptsCsvPreview store=receipts_store tab=tab /> }
                                .into_any()
                            list_href=kind.list_href()
                            on_saved=on_saved
                        />
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

/// 検証が終わった直後のフォーカス先。保存できるなら保存ボタン(Enter で保存)、
/// そうでなければ選び直せるファイル入力へ戻す(失敗時にフォーカスが迷子にならないように)
fn focus_element(selector: &'static str) {
    leptos::task::spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(0).await;
        let element = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.query_selector(selector).ok())
            .flatten();
        if let Some(element) = element {
            let _ = element.unchecked_into::<web_sys::HtmlElement>().focus();
        }
    });
}

/// 取り込み枠の本文。操作バー・検証の通知・プレビュー・保存結果を並べる
#[component]
fn CsvImportBody<S: CsvSource>(
    source: S,
    preview: AnyView,
    list_href: &'static str,
    on_saved: Callback<()>,
) -> impl IntoView {
    let meta = Memo::new(move |_| source.csv_meta());
    let notice = Memo::new(move |_| {
        let meta = meta.get();
        if should_show_preview_notice(
            source.is_authenticated(),
            meta.file_name.is_some(),
            meta.previewing,
        ) {
            meta.preview
                .map(|preview| (preview.valid_rows, preview.errors))
                .filter(|(valid_rows, errors)| *valid_rows == 0 || !errors.is_empty())
        } else {
            None
        }
    });
    let save_result = Memo::new(move |_| meta.get().import_result);
    let meta_error = Memo::new(move |_| meta.get().error);

    let prev_previewing = RwSignal::new(false);
    Effect::new(move |_| {
        let previewing = meta.get().previewing;
        let was = prev_previewing.get_untracked();
        prev_previewing.set(previewing);
        if !was || previewing {
            return;
        }
        let meta = source.csv_meta();
        if meta.preview.is_some() && !source.save_disabled(&meta) {
            focus_element("[data-testid='csv-save-button']");
        } else {
            focus_element("[data-testid='csv-file-input']");
        }
    });

    // 保存が終わったら保存済み件数を取り直す
    let saved_before = RwSignal::new(false);
    Effect::new(move |_| {
        let saved_now = meta.get().import_result.is_some();
        if saved_now && !saved_before.get_untracked() {
            on_saved.run(());
        }
        saved_before.set(saved_now);
    });

    view! {
        <div class="flex flex-col gap-2">
            <p class="text-xs font-medium text-text-muted">
                "保存方式: " {source.save_mode_label()}
            </p>
            {move || {
                meta.get()
                    .previewing
                    .then(|| view! { <LoadingStrip text="CSVファイルを解析しています...".to_string() /> })
            }}
            <CsvActionBar source=source />
            {move || {
                meta_error
                    .get()
                    .map(|message| {
                        view! {
                            <Alert variant=AlertVariant::Danger>
                                <strong>"エラー:"</strong>
                                " "
                                {message}
                            </Alert>
                        }
                    })
            }}
            {move || {
                notice
                    .get()
                    .map(|(valid_rows, errors)| {
                        view! {
                            <CsvPreviewNotice
                                valid_rows=valid_rows
                                save_action=source.save_action()
                                errors=errors
                                errors_block_save=source.errors_block_save()
                            />
                        }
                    })
            }}
            {preview}
            {move || {
                save_result
                    .get()
                    .map(|result| {
                        view! { <CsvSaveResultNotice result=result list_href=list_href /> }
                    })
            }}
        </div>
    }
}

/// ファイルの選択から保存までを1つのバーに置く。
/// 選ぶ前後でボタンの位置が変わらないため、マウスをほとんど動かさずに済む
#[component]
pub fn CsvActionBar<S: CsvSource>(source: S) -> impl IntoView {
    // 保存の再送待ちは ApiClient の内部で完結するため、フック経由で受けて
    // 操作バーの保存ボタンに「再試行中...」を出す
    let retrying = RwSignal::new(false);
    crate::api::client::set_retry_hook(move || retrying.set(true));
    let saving = Memo::new(move |_| source.csv_meta().saving);
    Effect::new(move |_| {
        if !saving.get() && retrying.get_untracked() {
            retrying.set(false);
        }
    });
    let input_id = source.input_id();
    let disabled = Memo::new(move |_| source.input_disabled());
    let has_file = Memo::new(move |_| source.csv_meta().file_name.is_some());
    let file_name = Memo::new(move |_| source.csv_meta().file_name.unwrap_or_default());
    let status_text = Memo::new(move |_| {
        let meta = source.csv_meta();
        if meta.previewing {
            Some("解析中...".to_string())
        } else if meta.saving {
            Some(
                if retrying.get() {
                    "再試行中..."
                } else {
                    "保存中..."
                }
                .to_string(),
            )
        } else {
            meta.preview.as_ref().map(|preview| {
                preview_notice_text(
                    preview.valid_rows,
                    preview.errors.len(),
                    source.save_action(),
                    source.errors_block_save(),
                )
            })
        }
    });
    let save_label = Memo::new(move |_| {
        let meta = source.csv_meta();
        if meta.saving {
            if retrying.get() {
                "再試行中..."
            } else {
                "保存中..."
            }
            .to_string()
        } else if meta.previewing {
            "解析中...".to_string()
        } else {
            source.save_button_label(meta.preview.as_ref().map(|preview| preview.valid_rows))
        }
    });
    let save_disabled = Memo::new(move |_| {
        let meta = source.csv_meta();
        !source.is_authenticated() || meta.file_name.is_none() || source.save_disabled(&meta)
    });
    let on_change = move |event: web_sys::Event| {
        if disabled.get_untracked() {
            return;
        }
        let input = event_target::<web_sys::HtmlInputElement>(&event);
        if let Some(file) = input.files().and_then(|files| files.get(0)) {
            source.select_file(file);
            // 同じファイルを続けて選択できるよう値をリセットする
            input.set_value("");
        }
    };
    let pick_button_class = "inline-flex min-h-11 shrink-0 cursor-pointer items-center justify-center rounded-md border border-border-strong bg-surface px-4 text-sm font-bold text-text transition-colors hover:border-border-bold hover:bg-surface-sunken focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-ink";
    let pick_trigger_class = move || {
        if disabled.get() {
            "flex min-h-20 items-center gap-3 rounded-lg border border-border-strong bg-surface-sunken px-4 py-3 transition-colors pointer-events-none opacity-65"
        } else {
            "flex min-h-20 cursor-pointer items-center gap-3 rounded-lg border border-border-strong bg-surface-sunken px-4 py-3 transition-colors hover:border-border-xstrong hover:bg-surface-raised focus-within:outline-2 focus-within:outline-offset-2 focus-within:outline-ink"
        }
    };
    view! {
        <div
            class="flex gap-2 py-2"
            data-testid="csv-action-bar"
        >
            {move || {
                if has_file.get() {
                    let file_input = view! {
                        <input
                            id=input_id
                            data-testid="csv-file-input"
                            type="file"
                            accept=".csv"
                            class="sr-only"
                            on:change=on_change
                            disabled=move || disabled.get()
                            aria-label="CSVファイルを選択"
                            // 入力の値は同じファイルの再選択のために消すので、読み上げは表示中のファイル名へ逃がす
                            aria-describedby=format!("{input_id}-filename")
                        />
                    };
                    view! {
                        // スマホではボタンが次の行に回り込み、ファイル名と状態は1行で読める幅を保つ
                        <div class="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-2 rounded-lg border border-border-strong bg-surface px-4 py-3 shadow-sm">
                            <CsvFileIcon />
                            <div class="min-w-0 flex-1 basis-56">
                                <p id=format!("{input_id}-filename") class="truncate text-sm font-bold text-text">
                                    {file_name}
                                </p>
                                {move || {
                                    status_text.get().map(|text| {
                                        view! {
                                            <p class="text-xs font-medium text-text-soft" role="status" aria-live="polite">
                                                {text}
                                            </p>
                                        }
                                    })
                                }}
                            </div>
                            // ラベル内に入力を置かないと focus-within が発火しない
                            <label class=pick_button_class>
                                {file_input}
                                "別のファイル"
                            </label>
                            <Button
                                variant=ButtonVariant::Primary(ButtonSize::Md)
                                testid="csv-save-button"
                                disabled=move || save_disabled.get()
                                aria_disabled=move || save_disabled.get()
                                // 前回の保存以外で立った再試行フラグを、保存の開始時に必ず落とす
                                on_click=move |_| {
                                    retrying.set(false);
                                    source.save_csv();
                                }
                            >
                                {move || save_label.get()}
                            </Button>
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <label class=pick_trigger_class data-testid="csv-file-trigger">
                            <CsvFileIcon />
                            <div class="min-w-0 flex-1">
                                <p class="text-sm font-bold text-text">
                                    {labels::CSV_IMPORT_PICK_PROMPT}
                                </p>
                                <p class="text-xs text-text-muted">{source.csv_format_hint()}</p>
                            </div>
                            <span class="inline-flex min-h-11 shrink-0 items-center justify-center rounded-md border border-brand bg-brand px-4 text-sm font-bold text-text-inverse shadow-edge-lit hover:bg-brand-deep">
                                "CSVを選ぶ"
                            </span>
                            <input
                                id=input_id
                                data-testid="csv-file-input"
                                type="file"
                                accept=".csv"
                                class="sr-only"
                                on:change=on_change
                                disabled=move || disabled.get()
                                aria-label="CSVファイルを選択"
                            />
                        </label>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
fn CsvFileIcon() -> impl IntoView {
    view! {
        <span class="inline-flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-surface text-text-muted shadow-sm">
            <svg
                class="h-5 w-5"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
                aria-hidden="true"
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.8"
                    d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M12 12v6m0-6l-2.5 2.5M12 12l2.5 2.5"
                />
            </svg>
        </span>
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

/// 種類ごとのテーブル
#[component]
pub fn CsvKindTable(
    rows: Memo<Vec<CsvKindRowData>>,
    asset_csv: AssetBalanceCsvStore,
    receipts_store: ReceiptsStore,
    csv_sync: CsvSyncStore,
    open_import: RwSignal<Option<CsvKind>>,
    on_stats_changed: Callback<()>,
) -> impl IntoView {
    // 件数・状態が更新されると行が作り直されるため、開いているメニューと削除確認は表で持つ
    let menu_kind = RwSignal::new(None::<CsvKind>);
    let delete_kind = RwSignal::new(None::<CsvKind>);
    let detail_kind = RwSignal::new(None::<CsvKind>);
    view! {
        <div>
            <table class="w-full" role="table">
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
                                        open_import=open_import
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

// 成功・追加なし・失敗を見出しで分ける(件数の有無だけでは区別しにくいため)
pub(crate) fn save_result_heading(result: &CsvUploadResponse) -> &'static str {
    match save_result_tone(result) {
        SaveResultTone::Success => "保存しました",
        SaveResultTone::NoAddition => "追加するデータはありませんでした",
        SaveResultTone::Failure => {
            if result.inserted == 0 {
                "保存できませんでした"
            } else {
                "一部の行を保存できませんでした"
            }
        }
    }
}

// 件数はバッジの集合ではなく一文で伝える。方式は画面上部に既出のため連結しない
pub(crate) fn save_result_line(result: &CsvUploadResponse) -> String {
    [
        Some(result.inserted_text()),
        result.skipped_text(),
        result.error_count_text(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("・")
}

// 成功・追加なし・失敗をアイコンと色で分ける(文言の違いだけでは区別しにくいため)。
// スキップだけで終わった保存は成功扱いにしない
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum SaveResultTone {
    Success,
    NoAddition,
    Failure,
}

pub(crate) fn save_result_tone(result: &CsvUploadResponse) -> SaveResultTone {
    if !result.errors.is_empty() {
        SaveResultTone::Failure
    } else if result.inserted > 0 {
        SaveResultTone::Success
    } else {
        SaveResultTone::NoAddition
    }
}

const SAVE_RESULT_STYLES: [(&str, &str, &str); 3] = [
    (
        "rounded-note border px-4 py-3.5 border-positive-border/80 bg-positive-soft/80",
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-surface border-positive-border text-positive-strong",
        "M5.5 10.5l3 3 6-7",
    ),
    (
        "rounded-note border px-4 py-3.5 border-accent-border/80 bg-accent-soft/80",
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-surface border-accent-border text-accent-strong",
        "M5.5 8h9m-9 4h9",
    ),
    (
        "rounded-note border px-4 py-3.5 border-negative-border/80 bg-negative-soft/80",
        "mt-0.5 inline-flex h-8 w-8 shrink-0 items-center justify-center rounded-full border bg-surface border-negative-border text-negative-strong",
        "M5.5 5.5l9 9m0-9l-9 9",
    ),
];

#[component]
fn CsvSaveResultNotice(result: CsvUploadResponse, list_href: &'static str) -> impl IntoView {
    let has_errors = !result.errors.is_empty();
    let aggregated = shared_message(&result.errors).is_some();
    let index = match save_result_tone(&result) {
        SaveResultTone::Success => 0,
        SaveResultTone::NoAddition => 1,
        SaveResultTone::Failure => 2,
    };
    let (section_class, icon_class, icon_path) = SAVE_RESULT_STYLES[index];
    let heading = save_result_heading(&result);
    let line = save_result_line(&result);
    let skipped_note = (save_result_tone(&result) == SaveResultTone::NoAddition
        && result.skipped > 0)
        .then(|| {
            format!(
                "{}件はすべて保存済みのため、スキップしました。データは変わりません。",
                result.skipped
            )
        });
    view! {
        <section
            class=section_class
            data-testid="csv-save-result-notice"
            role="status"
            aria-live="polite"
        >
            <div class="flex items-start gap-3">
                <span class=icon_class aria-hidden="true">
                    <svg class="h-4.5 w-4.5" viewBox="0 0 20 20" fill="none" stroke="currentColor">
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="1.8"
                            d=icon_path
                        />
                    </svg>
                </span>
                <div class="min-w-0 flex-1">
                    <p class="text-sm font-semibold text-text">{heading}</p>
                    <p class="mt-1 text-sm text-text-soft">{line}</p>
                    {skipped_note.map(|note| {
                        view! { <p class="mt-1 text-sm text-text-soft">{note}</p> }
                    })}
                    {has_errors.then(|| {
                        let all_failed = result.inserted == 0 && result.skipped == 0;
                        let list = view! {
                            <CsvRowErrorList errors=result.errors.clone() all_failed=all_failed />
                        };
                        if aggregated {
                            view! { <div class="mt-3 text-sm text-text-muted">{list}</div> }
                                .into_any()
                        } else {
                            view! {
                                <details class="mt-3 rounded-2xl border border-accent-border/80 bg-surface/80 px-3 py-2">
                                    <summary class="cursor-pointer text-sm font-medium text-text-soft">
                                        "エラー詳細を表示"
                                    </summary>
                                    <div class="text-sm text-text-muted">{list}</div>
                                </details>
                            }
                            .into_any()
                        }
                    })}
                    <div class="mt-3">
                        <LinkButton variant=ButtonVariant::Primary(ButtonSize::Md) href=list_href>
                            "一覧で確認する"
                        </LinkButton>
                    </div>
                </div>
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests;
