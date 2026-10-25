use crate::features::asset_balance::fund::AssetKind;
use crate::features::asset_balance::store::csv::{AssetBalanceCsvStore, csv_preview_rows};
use crate::features::asset_balance::view::{AssetBalanceCsvPreview, FundBalanceCsvPreview};
use crate::support::csv_flow::CsvTabMeta;
use crate::ui::csv_page::CsvSource;
use crate::ui::labels;
use leptos::prelude::*;

/// 資産管理(国内株式・投資信託)それぞれの CSV 操作元。種別ごとに固定
#[derive(Clone, Copy)]
pub(crate) struct AssetBalanceCsvSource {
    pub(crate) store: AssetBalanceCsvStore,
    pub(crate) kind: AssetKind,
}

impl AssetBalanceCsvSource {
    fn is_fund(&self) -> bool {
        self.kind == AssetKind::Fund
    }
}

impl CsvSource for AssetBalanceCsvSource {
    fn input_id(&self) -> &'static str {
        if self.is_fund() {
            "csv-file-input-fundbalance"
        } else {
            "csv-file-input-assetbalance"
        }
    }

    fn save_action(&self) -> &'static str {
        "全件置換で保存"
    }

    fn save_mode_label(&self) -> &'static str {
        labels::CSV_MANAGE_SAVE_MODE_REPLACE
    }

    // 追加保存と違い、置換の対象が全件だとボタン上でも分かるようにする
    fn save_button_label(&self, valid_rows: Option<usize>) -> String {
        match valid_rows {
            Some(rows) => format!("{rows}件 全件置換で保存"),
            None => "全件置換で保存".to_string(),
        }
    }

    fn csv_meta(&self) -> CsvTabMeta {
        if self.is_fund() {
            self.store.fund_csv_meta()
        } else {
            self.store.csv_meta()
        }
    }

    fn is_authenticated(&self) -> bool {
        self.store.is_authenticated()
    }

    fn input_disabled(&self) -> bool {
        self.store.csv_input_disabled(self.kind)
    }

    fn errors_block_save(&self) -> bool {
        true
    }

    fn csv_format_hint(&self) -> &'static str {
        if self.is_fund() {
            "対応形式: 投資信託の保有残高CSV(見出し行の先頭は「投資信託種別」)"
        } else {
            "対応形式: 資産残高CSV(見出し行の先頭は「銘柄コード」)"
        }
    }

    fn db_count(&self) -> usize {
        if self.is_fund() {
            self.store.fund_db_count()
        } else {
            self.store.db_count()
        }
    }

    fn delete_count_ready(&self) -> bool {
        if self.is_fund() {
            self.store.fund_saved_count_ready()
        } else {
            self.store.saved_count_ready()
        }
    }

    fn delete_count_error(&self) -> Option<String> {
        if self.is_fund() {
            self.store.fund_saved_count_error()
        } else {
            self.store.saved_count_error()
        }
    }

    fn date_range(&self) -> Option<(String, String)> {
        if self.is_fund() {
            self.store.fund_date_range()
        } else {
            self.store.date_range()
        }
    }

    fn refresh_for_delete(&self) {
        if self.is_fund() {
            self.store.refresh_fund_for_delete();
        } else {
            self.store.refresh_for_delete();
        }
    }

    fn delete_disabled(&self, meta: &CsvTabMeta) -> bool {
        let list_loading = if self.is_fund() {
            self.store.fund_list_loading()
        } else {
            self.store.list_loading()
        };
        meta.saving || meta.deleting || list_loading
    }

    fn select_file(&self, file: web_sys::File) {
        self.store.select_file(self.kind, file)
    }

    fn save_csv(&self) {
        self.store.save_csv(self.kind)
    }

    fn open_delete_confirm(&self) {
        if self.is_fund() {
            self.store.fund_open_delete_confirm()
        } else {
            self.store.open_delete_confirm()
        }
    }

    fn confirm_delete_all(&self) {
        if self.is_fund() {
            self.store.fund_confirm_delete_all()
        } else {
            self.store.confirm_delete_all()
        }
    }
}

/// 国内株式の検証済みプレビュー。行が無い間は何も出さない
#[component]
pub(crate) fn StockCsvPreview(store: AssetBalanceCsvStore) -> impl IntoView {
    move || {
        let rows = csv_preview_rows(&store.csv_state());
        (!rows.is_empty()).then(|| view! { <AssetBalanceCsvPreview rows=rows /> })
    }
}

/// 投資信託の検証済みプレビュー。行が無い間は何も出さない
#[component]
pub(crate) fn FundCsvPreview(store: AssetBalanceCsvStore) -> impl IntoView {
    move || {
        let rows = csv_preview_rows(&store.fund_csv_state());
        (!rows.is_empty()).then(|| view! { <FundBalanceCsvPreview rows=rows /> })
    }
}
