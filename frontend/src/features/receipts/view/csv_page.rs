use super::table::ReceiptTable;
use crate::features::receipts::model::format_date;
use crate::features::receipts::{
    ReceiptRow, ReceiptsStore, ReceiptsTab, TabState, use_receipts_data,
};
use crate::session::use_session;
use crate::support::csv_flow::CsvTabMeta;
use crate::support::row::Row;
use crate::ui::csv_page::{CsvManagePage, CsvSource};
use leptos::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct ReceiptCsvSource {
    pub(crate) store: ReceiptsStore,
    pub(crate) tab: ReceiptsTab,
}

impl CsvSource for ReceiptCsvSource {
    fn input_id(&self) -> &'static str {
        self.tab.csv_input_id()
    }

    fn save_action(&self) -> &'static str {
        "追加で保存"
    }

    fn save_mode_label(&self) -> &'static str {
        "追加"
    }

    fn mode_label(&self) -> &'static str {
        "追加保存"
    }

    fn csv_meta(&self) -> CsvTabMeta {
        self.store.csv_meta(self.tab)
    }

    fn is_authenticated(&self) -> bool {
        self.store.is_authenticated()
    }

    fn input_disabled(&self) -> bool {
        self.store.csv_input_disabled(self.tab)
    }

    fn csv_format_hint(&self) -> &'static str {
        match self.tab {
            ReceiptsTab::Dividend => {
                "対応形式: 「配当金・分配金・利金一覧」CSV(見出し行の先頭は「入金日」)"
            }
            ReceiptsTab::DomesticStock => {
                "対応形式: 国内株式の取引履歴CSV(見出し行の先頭は「約定日」)"
            }
            ReceiptsTab::MutualFund => {
                "対応形式: 投資信託の取引履歴CSV(見出し行の先頭は「約定日」)"
            }
        }
    }

    fn db_count(&self) -> usize {
        self.store.count(self.tab)
    }

    fn delete_count_ready(&self) -> bool {
        // 裏再取得の失敗は一覧を Ready のまま残すため、失敗が残る間は古い件数を未確定として扱う
        matches!(self.store.tab_state(self.tab), TabState::Ready(_))
            && self.store.refresh_error(self.tab).is_none()
    }

    fn delete_count_error(&self) -> Option<String> {
        match self.store.tab_state(self.tab) {
            TabState::Failed(message) => Some(message),
            // 取り直し中は前回の失敗ではなく取得中として扱う
            TabState::Loading => None,
            _ => self.store.refresh_error(self.tab),
        }
    }

    fn date_range(&self) -> Option<(String, String)> {
        if let TabState::Ready(data) = self.store.tab_state(self.tab) {
            let dates: Vec<&str> = data.rows.iter().map(|row| row.date()).collect();
            if dates.is_empty() {
                return None;
            }
            let min = dates.iter().min()?;
            let max = dates.iter().max()?;
            Some((format_date(min), format_date(max)))
        } else {
            None
        }
    }

    fn refresh_for_delete(&self) {
        self.store.refresh_for_delete(self.tab);
    }

    fn delete_disabled(&self, meta: &CsvTabMeta) -> bool {
        meta.saving || meta.deleting || self.store.any_tab_fetching()
    }

    fn select_file(&self, file: web_sys::File) {
        self.store.select_file(self.tab, file)
    }

    fn save_csv(&self) {
        self.store.save_csv(self.tab)
    }

    fn discard_csv(&self) {
        self.store.discard_csv(self.tab)
    }

    fn open_delete_confirm(&self) {
        self.store.open_delete_confirm(self.tab)
    }

    fn confirm_delete_all(&self) {
        self.store.confirm_delete_all(self.tab)
    }
}

// ?kind= はページの再マウントでしか変わらないため、マウント時に1回だけ読む
fn kind_tab() -> ReceiptsTab {
    ReceiptsTab::from_kind_param(
        crate::app::query_param(&crate::app::current_location(), "kind").as_deref(),
    )
}

// 世代リセットが起きると ensure の対象が配当へ差し替わるので、
// 世代を確定させてから対象タブの一覧を未取得なら取得する。
// ?kind= 直リンクでは一覧側の選択タブも URL の種別に同期し、「一覧に戻る」が同じタブを開く。
// active_tab を直接変えると検索語が前のタブから残るため、通常のタブ切替と同じリセットを通す
pub(crate) fn ensure_kind(store: ReceiptsStore, tab: ReceiptsTab) {
    store.ensure(tab);
    store.select_tab(tab);
}

#[component]
pub fn ReceiptsCsvPage() -> impl IntoView {
    let store = use_receipts_data(use_session(), ReceiptsTab::Dividend);
    let tab = kind_tab();
    ensure_kind(store, tab);
    let source = ReceiptCsvSource { store, tab };
    view! {
        <CsvManagePage
            source=source
            subject=tab.label()
            back_href="/receipts"
            delete_href=String::new()
            preview=view! { <ReceiptsCsvPreview store=store tab=tab /> }.into_any()
        />
    }
}

/// 旧削除ページ URL へのアクセスを CSV 管理ページへリダイレクトする
#[component]
pub fn ReceiptsCsvDeletePage() -> impl IntoView {
    let tab = kind_tab();
    let csv_href = format!("/receipts/csv?kind={}", tab.kind_param());
    leptos::task::spawn_local(async move {
        // 次のマイクロタスクで遷移させ、レンダリングを挟まない
        if let Some(window) = web_sys::window() {
            window.location().set_href(&csv_href).ok();
        }
    });
    view! { <div>"CSV管理ページへ移動します..."</div> }
}

#[component]
fn ReceiptsCsvPreview(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    move || {
        let preview = store.csv_state(tab).preview?;
        if preview.rows.is_empty() {
            return None;
        }
        let rows: Vec<ReceiptRow> = preview.rows.into_iter().map(Row::Preview).collect();
        Some(view! {
            <section data-testid="csv-preview">
                <h2 class="text-sm font-bold text-text">"プレビュー"</h2>
                <p class="mt-1 text-xs text-text-soft">
                    "表は取り込むファイルの内容です。保存するまで登録済みのデータは変わりません。"
                </p>
                <div class="mt-2">
                    <ReceiptTable
                        tab=tab
                        rows=rows.clone()
                        all_rows=rows
                        query="".to_string()
                        expanded_ids=store.expanded
                        compact_mobile=true
                    />
                </div>
            </section>
        })
    }
}
