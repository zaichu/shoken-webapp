use super::csv_rail::CsvActionRail;
use crate::support::csv_flow::CsvTabState;
use leptos::prelude::*;

/// CSV 取り込み・削除欄が要求するデータソース。
/// 取引明細(タブ付き)と資産残高の両方が同じ構成なので、状態取得と
/// 操作だけをトレイトで受け取り、Memo 生成と CsvActionRail への配線を共通化する。
pub trait CsvSource: Copy + Send + Sync + 'static {
    type Row: Clone + PartialEq + 'static;

    fn input_id(&self) -> &'static str;
    fn save_action(&self) -> &'static str;
    fn mode_label(&self) -> &'static str;
    fn toggle_testid(&self) -> &'static str;
    // 既定は input_id ベース。asset_balance は独自 id を持つため上書きする
    fn body_id(&self) -> String {
        format!("{}-body", self.input_id())
    }
    fn section_class(&self) -> &'static str;

    fn csv_state(&self) -> CsvTabState<Self::Row>;
    fn is_authenticated(&self) -> bool;
    fn input_disabled(&self) -> bool;
    fn save_disabled(&self, state: &CsvTabState<Self::Row>) -> bool {
        state.busy()
    }
    fn db_count(&self) -> usize;
    fn delete_disabled(&self, state: &CsvTabState<Self::Row>) -> bool {
        state.saving || state.deleting
    }

    fn select_file(&self, file: web_sys::File);
    fn save_csv(&self);
    fn open_delete_confirm(&self);
}

#[component]
pub fn CsvSection<S: CsvSource>(source: S) -> impl IntoView {
    let selected_file_name = Memo::new(move |_| source.csv_state().file_name.unwrap_or_default());
    let file_input_disabled =
        Memo::new(move |_| source.input_disabled() || !source.is_authenticated());
    let has_csv_file =
        Memo::new(move |_| source.is_authenticated() && source.csv_state().file_name.is_some());
    let save_label = Memo::new(move |_| source.csv_state().save_label(source.save_action()));
    let save_disabled = Memo::new(move |_| source.save_disabled(&source.csv_state()));
    let has_db_data = Memo::new(move |_| source.is_authenticated() && source.db_count() > 0);
    let delete_label = Memo::new(move |_| source.csv_state().delete_label(source.db_count()));
    let delete_disabled = Memo::new(move |_| source.delete_disabled(&source.csv_state()));
    let save_result = Memo::new(move |_| source.csv_state().import_result);

    view! {
        <CsvActionRail
            input_id=source.input_id()
            on_file_select=move |file| source.select_file(file)
            selected_file_name=selected_file_name
            file_input_disabled=file_input_disabled
            has_csv_file=has_csv_file
            save_label=save_label
            on_save=move || source.save_csv()
            save_disabled=save_disabled
            has_db_data=has_db_data
            delete_label=delete_label
            on_delete_request=move || source.open_delete_confirm()
            delete_disabled=delete_disabled
            save_result=save_result
            mode_label=source.mode_label()
            toggle_testid=source.toggle_testid()
            body_id=source.body_id()
            section_class=source.section_class()
        />
    }
}
