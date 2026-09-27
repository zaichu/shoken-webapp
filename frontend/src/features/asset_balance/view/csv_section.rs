use crate::features::asset_balance::csv::AssetBalanceCsvRow;
use crate::features::asset_balance::csv_store::{can_save_csv, AssetBalanceCsvStore};
use crate::support::csv_flow::CsvTabState;
use crate::ui::csv_section::{CsvSection, CsvSource};
use leptos::prelude::*;

impl CsvSource for AssetBalanceCsvStore {
    type Row = AssetBalanceCsvRow;

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
        "sm:border-b sm:border-slate-950/10"
    }

    fn csv_state(&self) -> CsvTabState<AssetBalanceCsvRow> {
        AssetBalanceCsvStore::csv_state(self)
    }

    fn is_authenticated(&self) -> bool {
        AssetBalanceCsvStore::is_authenticated(self)
    }

    fn input_disabled(&self) -> bool {
        self.csv_busy() || self.list_loading()
    }

    fn save_disabled(&self, state: &CsvTabState<AssetBalanceCsvRow>) -> bool {
        state.busy() || !can_save_csv(state)
    }

    fn db_count(&self) -> usize {
        AssetBalanceCsvStore::db_count(self)
    }

    fn delete_disabled(&self, state: &CsvTabState<AssetBalanceCsvRow>) -> bool {
        state.saving || state.deleting || self.list_loading()
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

#[component]
pub(crate) fn AssetBalanceCsvSection(store: AssetBalanceCsvStore) -> impl IntoView {
    view! {
        // divide の半透明線は下地色で見え方が変わるため、sm 以上は内側 section 側の線に揃える
        <div class="sm:border-b-0">
            <CsvSection source=store />
        </div>
    }
}
