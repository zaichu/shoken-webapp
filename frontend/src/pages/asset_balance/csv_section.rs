use super::csv::{can_save_csv, AssetBalanceCsvStore};
use crate::components::csv_rail::CsvActionRail;
use leptos::prelude::*;

#[component]
pub(crate) fn AssetBalanceCsvSection(store: AssetBalanceCsvStore) -> impl IntoView {
    let selected = store.clone();
    let selected_file_name = Memo::new(move |_| selected.csv_state().file_name.unwrap_or_default());
    let disabled_store = store.clone();
    let file_input_disabled = Memo::new(move |_| {
        !disabled_store.is_authenticated()
            || disabled_store.csv_busy()
            || disabled_store.list_loading()
    });
    let has_file = store.clone();
    let has_csv_file =
        Memo::new(move |_| has_file.is_authenticated() && has_file.csv_state().file_name.is_some());
    let label_store = store.clone();
    let save_label = Memo::new(move |_| label_store.csv_state().save_label("全件置換で保存"));
    let save_dis = store.clone();
    let save_disabled =
        Memo::new(move |_| save_dis.csv_state().busy() || !can_save_csv(&save_dis.csv_state()));
    let has_db = store.clone();
    let has_db_data = Memo::new(move |_| has_db.is_authenticated() && has_db.db_count() > 0);
    let del_label = store.clone();
    let delete_label = Memo::new(move |_| del_label.csv_state().delete_label(del_label.db_count()));
    let del_dis = store.clone();
    let delete_disabled = Memo::new(move |_| {
        let state = del_dis.csv_state();
        state.saving || state.deleting || del_dis.list_loading()
    });
    let result_store = store.clone();
    let save_result = Memo::new(move |_| result_store.csv_state().import_result);
    let file_select = store.clone();
    let save = store.clone();
    let delete_request = store.clone();
    view! {
        // divide の半透明線は下地色で見え方が変わるため、sm 以上は内側 section 側の線に揃える
        <div class="sm:border-b-0">
            <CsvActionRail
                input_id="csv-file-input-assetbalance"
                toggle_testid="assetbalance-csv-toggle"
                body_id="assetbalance-csv-body"
                section_class="sm:border-b sm:border-slate-950/10"
                on_file_select=move |file| file_select.select_file(file)
                selected_file_name=selected_file_name
                file_input_disabled=file_input_disabled
                has_csv_file=has_csv_file
                save_label=save_label
                on_save=move || save.save_csv()
                save_disabled=save_disabled
                has_db_data=has_db_data
                delete_label=delete_label
                on_delete_request=move || delete_request.open_delete_confirm()
                delete_disabled=delete_disabled
                save_result=save_result
                mode_label="全件置換"
            />
        </div>
    }
}
