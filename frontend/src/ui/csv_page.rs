use crate::features::asset_balance::AssetBalanceCsvStore;
use crate::features::csv_sync::CsvSyncStore;
use crate::features::receipts::ReceiptsStore;
use crate::support::csv_flow::CsvTabMeta;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::labels;
use leptos::prelude::*;

#[allow(unused_imports)]
use crate::features::asset_balance::{
    AssetBalanceCsvSource, AssetKind, FundCsvPreview, StockCsvPreview,
};
#[allow(unused_imports)]
use crate::features::receipts::{ReceiptCsvSource, ReceiptsCsvPreview, ReceiptsTab};
#[allow(unused_imports)]
use crate::support::csv_flow::CsvUploadResponseExt;

// 全件置換ではエラー行を含む CSV を backend が拒否するため、有効行があっても保存を止める
#[allow(dead_code)]
pub(crate) fn save_disabled_by_preview(meta: &CsvTabMeta, errors_block_save: bool) -> bool {
    meta.busy()
        || !meta.has_preview_rows()
        || (errors_block_save
            && meta
                .preview
                .as_ref()
                .is_some_and(|preview| !preview.errors.is_empty()))
}

/// CSV 管理・削除ページが要求するデータソース。
/// 取引明細(種別付き)と資産残高が同じページ構成を共有するため、
/// 状態取得と操作をトレイトで受け取る。状態は行データを含まない CsvTabMeta
#[allow(dead_code)]
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
    /// 保存済み全件数。削除ページで対象件数を示すために使う
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

/// 種類ごとの行データ
#[derive(Clone, Debug, PartialEq)]
pub struct CsvKindRowData {
    pub kind: crate::features::csv_page::CsvKind,
    pub count: i64,
    pub latest_imported_at: Option<String>,
    /// csv_syncの状態
    pub sync_status: Option<crate::features::csv_sync::KindStatus>,
}

/// CSV 管理画面の取得カード
#[component]
pub fn CsvManageFetchCard(
    running: ReadSignal<bool>,
    fetch_result: ReadSignal<Option<String>>,
    last_fetched: ReadSignal<Option<String>>,
    on_fetch: impl Fn(web_sys::MouseEvent) + Copy + 'static,
) -> impl IntoView {
    view! {
        <section class="rounded-lg border border-border bg-surface-sunken p-4">
            <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                <div>
                    <h2 class="text-sm font-bold text-text">{labels::CSV_MANAGE_FETCH_TITLE}</h2>
                    <p class="mt-0.5 text-xs text-text-soft">{labels::CSV_MANAGE_FETCH_HINT}</p>
                </div>
                <div class="flex flex-col sm:flex-row gap-3 items-start sm:items-center w-full sm:w-auto">
                    <Button
                        variant=ButtonVariant::Primary(ButtonSize::Md)
                        disabled=move || running.get()
                        on_click=on_fetch
                        class="w-full sm:w-auto"
                    >
                        {move || {
                            if running.get() {
                                labels::CSV_MANAGE_FETCH_RUNNING
                            } else {
                                labels::CSV_MANAGE_FETCH_BUTTON
                            }
                        }}
                    </Button>
                    <div class="flex-1 min-w-0">
                        {move || {
                            fetch_result.get().map(|msg| {
                                view! {
                                    <p class="text-sm text-text" role="status" aria-live="polite">
                                        {msg}
                                    </p>
                                }
                            })
                        }}
                        {move || {
                            last_fetched.get().and_then(|lf| {
                                fetch_result.get().is_none().then_some(lf)
                            }).map(|lf| {
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

/// 種類ごとのテーブル行 - シンプルな実装（メニューは単純なボタン群で代替）
#[component]
pub fn CsvKindRow(
    row_data: CsvKindRowData,
    asset_csv: AssetBalanceCsvStore,
    receipts_store: ReceiptsStore,
    csv_sync: CsvSyncStore,
) -> impl IntoView {
    let kind = row_data.kind;
    let count = row_data.count;
    let latest_imported_at = row_data.latest_imported_at;
    let sync_status = row_data.sync_status;

    // 状態表示の計算
    let (status_text, status_class, has_error_detail, error_detail) =
        compute_status(&sync_status, &latest_imported_at, count == 0);

    // 削除確認モーダル
    let show_delete_confirm = RwSignal::new(false);
    let delete_error = RwSignal::new(None::<String>);

    let open_delete = move |_| {
        delete_error.set(None);
        // 件数を取り直す
        match kind {
            crate::features::csv_page::CsvKind::AssetStock => {
                asset_csv.refresh_for_delete();
            }
            crate::features::csv_page::CsvKind::AssetFund => {
                asset_csv.refresh_fund_for_delete();
            }
            _ => {
                if let Some(tab) = kind.receipt_tab() {
                    receipts_store.refresh_for_delete(tab);
                }
            }
        }
        show_delete_confirm.set(true);
    };

    let confirm_delete = move || {
        match kind {
            crate::features::csv_page::CsvKind::AssetStock => {
                asset_csv.confirm_delete_all();
            }
            crate::features::csv_page::CsvKind::AssetFund => {
                asset_csv.fund_confirm_delete_all();
            }
            _ => {
                if let Some(tab) = kind.receipt_tab() {
                    receipts_store.confirm_delete_all(tab);
                }
            }
        }
        show_delete_confirm.set(false);
    };

    let cancel_delete = move || {
        show_delete_confirm.set(false);
        delete_error.set(None);
    };

    // メニュー開閉
    let menu_open = RwSignal::new(false);

    let on_refetch = move |_| {
        menu_open.set(false);
        // 段階1では全種類取得と同じ処理を呼ぶ
        csv_sync.start();
    };

    let on_file_import = move |_| {
        menu_open.set(false);
        // 既存の手動取り込みフローへ誘導（タブ切り替え等で対応）
    };

    view! {
        <tr class="border-t border-border-subtle" data-kind=kind.kind_param()>
            <td class="px-4 py-3 whitespace-nowrap">
                <span class="text-text font-medium">{kind.label()}</span>
            </td>
            <td class="px-4 py-3 whitespace-nowrap text-right text-text-muted">
                {labels::count(count as usize)}
            </td>
            <td class="px-4 py-3 whitespace-nowrap">
                <div class="flex items-center gap-2">
                    <span class=status_class>{status_text}</span>
                    {has_error_detail.then(|| {
                        view! {
                            <button
                                type="button"
                                class="text-xs underline text-text-muted hover:text-text"
                                data-testid="status-detail"
                                on:click=move |_| {
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.alert_with_message(&error_detail);
                                    }
                                }
                            >
                                {labels::CSV_MANAGE_FAILED_DETAIL_LABEL}
                            </button>
                        }
                    })}
                </div>
            </td>
            <td class="px-4 py-3 whitespace-nowrap text-right">
                <div class="relative inline-block">
                    <Button
                        variant=ButtonVariant::Secondary(ButtonSize::Sm)
                        class="inline-flex h-8 w-8 items-center justify-center rounded-md hover:bg-surface-sunken text-text-muted"
                        aria_label="メニュー"
                        aria_expanded=move || menu_open.get()
                        on_click=move |_| menu_open.update(|o| *o = !*o)
                    >
                        <span class="text-lg font-mono">"⋯"</span>
                    </Button>
                    {move || menu_open.get().then(|| {
                        view! {
                            <div class="absolute right-0 z-40 mt-1 w-56 overflow-hidden rounded-lg border border-border bg-surface p-1 shadow-lg">
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
                                    class="w-full flex items-center gap-2 px-3 py-2 text-sm font-medium text-negative hover:bg-negative/10 rounded text-left"
                                    on:click=open_delete
                                    disabled=count == 0
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
        {show_delete_confirm.get().then(|| {
            view! {
                <ConfirmDeleteModal
                    title=labels::csv_manage_delete_confirm_title(kind.delete_subject())
                    description=labels::csv_manage_delete_confirm_description(kind.delete_subject(), count as usize)
                    item_count=count as usize
                    confirm_label=labels::csv_manage_delete_confirm_button(count as usize)
                    loading=Memo::new(move |_| false)
                    confirm_ready=Memo::new(move |_| true)
                    error=Memo::new(move |_| delete_error.get())
                    on_confirm=confirm_delete
                    on_cancel=cancel_delete
                />
            }
        })}
    }
}

/// 状態テキストとクラスを計算
#[allow(clippy::collapsible_if)]
fn compute_status(
    sync_status: &Option<crate::features::csv_sync::KindStatus>,
    latest_imported_at: &Option<String>,
    _is_empty: bool,
) -> (String, String, bool, String) {
    use crate::features::csv_sync::KindStatus;

    // 取得中・取り込み中の状態を最優先
    if let Some(status) = sync_status {
        match status {
            KindStatus::Fetching => {
                return (
                    "取得中...".to_string(),
                    "text-accent-vivid".to_string(),
                    false,
                    String::new(),
                );
            }
            KindStatus::Importing => {
                return (
                    "取り込み中...".to_string(),
                    "text-accent-vivid".to_string(),
                    false,
                    String::new(),
                );
            }
            KindStatus::Confirming => {
                return (
                    "確認待ち".to_string(),
                    "text-accent-vivid".to_string(),
                    false,
                    String::new(),
                );
            }
            KindStatus::Failed { reason } => {
                return (
                    labels::CSV_MANAGE_STATUS_FAILED.to_string(),
                    "text-negative-vivid".to_string(),
                    true,
                    reason.clone(),
                );
            }
            KindStatus::Skipped { reason } => {
                return (
                    format!("取り込まなかった({})", reason.label()),
                    "text-text-muted".to_string(),
                    false,
                    String::new(),
                );
            }
            KindStatus::Imported { .. } | KindStatus::Done { .. } => {
                // 成功時は最終取込日時で判断
            }
            KindStatus::Waiting => {}
        }
    }

    // 最終取込日時から状態を判定
    if let Some(ts) = latest_imported_at {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
            let jst = dt.with_timezone(&chrono::FixedOffset::east_opt(9 * 3600).unwrap());
            let today_jst = chrono::Local::now().date_naive(); // 近似
            let imported_date = jst.date_naive();
            let days_diff = (today_jst - imported_date).num_days();

            let date_str = jst.format("%m/%d").to_string();
            let time_str = jst.format("%H:%M").to_string();

            if days_diff <= 0 {
                // 今日
                return (
                    labels::csv_manage_status_updated_today(&time_str),
                    "text-positive-vivid".to_string(),
                    false,
                    String::new(),
                );
            } else {
                // 1日以上前
                return (
                    labels::csv_manage_status_updated_days_ago(days_diff, &date_str, &time_str),
                    "text-text-muted".to_string(),
                    false,
                    String::new(),
                );
            }
        }
    }

    // 取得履歴なし
    (
        labels::CSV_MANAGE_STATUS_NEVER.to_string(),
        "text-text-muted".to_string(),
        false,
        String::new(),
    )
}

/// 種類ごとのテーブル
#[component]
pub fn CsvKindTable(
    rows: Memo<Vec<CsvKindRowData>>,
    asset_csv: AssetBalanceCsvStore,
    receipts_store: ReceiptsStore,
    csv_sync: CsvSyncStore,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <table class="w-full" role="table">
                <thead class="bg-surface-sunken border-b border-border">
                    <tr>
                        <th class="px-4 py-3 text-left text-xs font-bold text-text-muted uppercase tracking-wider">
                            {labels::CSV_MANAGE_COL_KIND}
                        </th>
                        <th class="px-4 py-3 text-right text-xs font-bold text-text-muted uppercase tracking-wider">
                            {labels::CSV_MANAGE_COL_COUNT}
                        </th>
                        <th class="px-4 py-3 text-left text-xs font-bold text-text-muted uppercase tracking-wider">
                            {labels::CSV_MANAGE_COL_STATUS}
                        </th>
                        <th class="px-4 py-3 text-right text-xs font-bold text-text-muted uppercase tracking-wider">
                            {labels::CSV_MANAGE_COL_MENU}
                        </th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-border-subtle">
                    {move || {
                        rows.get().into_iter().map(|row| {
                            view! {
                                <CsvKindRow row_data=row asset_csv=asset_csv receipts_store=receipts_store csv_sync=csv_sync />
                            }
                        }).collect_view()
                    }}
                </tbody>
            </table>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_disabled_by_preview_blocks_on_errors() {
        let meta = crate::support::csv_flow::CsvTabMeta {
            previewing: false,
            preview: Some(crate::support::csv_flow::CsvPreviewMeta {
                valid_rows: 5,
                errors: vec![crate::api::dto::CsvRowError {
                    row: 1,
                    message: "error".to_string(),
                }],
                has_rows: true,
            }),
            saving: false,
            deleting: false,
            error: None,
            import_result: None,
            file_name: None,
            show_delete_confirm: false,
        };
        assert!(save_disabled_by_preview(&meta, true));
        assert!(!save_disabled_by_preview(&meta, false));
    }
}
