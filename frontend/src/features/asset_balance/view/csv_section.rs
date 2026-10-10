use crate::features::asset_balance::store::csv::AssetBalanceCsvStore;
use crate::support::csv_flow::CsvTabMeta;
use crate::ui::csv_section::CsvSource;

impl CsvSource for AssetBalanceCsvStore {
    fn input_id(&self) -> &'static str {
        "csv-file-input-assetbalance"
    }

    fn save_action(&self) -> &'static str {
        "全件置換で保存"
    }

    fn mode_label(&self) -> &'static str {
        "全件置換"
    }

    fn toggle_testid(&self) -> &'static str {
        "assetbalance-csv-toggle"
    }

    fn body_id(&self) -> String {
        "assetbalance-csv-body".to_string()
    }

    fn section_class(&self) -> &'static str {
        "sm:border-b sm:border-ink/10"
    }

    fn csv_meta(&self) -> CsvTabMeta {
        AssetBalanceCsvStore::csv_meta(self)
    }

    fn is_authenticated(&self) -> bool {
        AssetBalanceCsvStore::is_authenticated(self)
    }

    fn input_disabled(&self) -> bool {
        self.csv_input_disabled()
    }

    fn save_disabled(&self, meta: &CsvTabMeta) -> bool {
        meta.busy() || !meta.has_preview_rows()
    }

    fn db_count(&self) -> usize {
        AssetBalanceCsvStore::db_count(self)
    }

    fn delete_disabled(&self, meta: &CsvTabMeta) -> bool {
        meta.saving || meta.deleting || self.list_loading()
    }

    fn select_file(&self, file: web_sys::File) {
        AssetBalanceCsvStore::select_file(self, file)
    }

    fn save_csv(&self) {
        AssetBalanceCsvStore::save_csv(self)
    }

    fn open_delete_confirm(&self) {
        AssetBalanceCsvStore::open_delete_confirm(self)
    }
}
