use super::csv_rail::CsvActionRail;
use crate::support::csv_flow::CsvTabMeta;
use crate::ui::csv_preview::{preview_notice_text, should_show_preview_notice};
use leptos::prelude::*;

/// CSV 取り込み・削除欄が要求するデータソース。
/// 取引明細(タブ付き)と資産残高の両方が同じ構成なので、状態取得と
/// 操作だけをトレイトで受け取り、Memo 生成と CsvActionRail への配線を共通化する。
/// 状態は行データを含まない CsvTabMeta で受け取り、プレビュー行の複製を避ける
pub trait CsvSource: Copy + Send + Sync + 'static {
    fn input_id(&self) -> &'static str;
    fn save_action(&self) -> &'static str;
    fn mode_label(&self) -> &'static str;
    fn toggle_testid(&self) -> &'static str;
    // 既定は input_id ベース。asset_balance は独自 id を持つため上書きする
    fn body_id(&self) -> String {
        format!("{}-body", self.input_id())
    }
    fn section_class(&self) -> &'static str;

    fn csv_meta(&self) -> CsvTabMeta;
    fn is_authenticated(&self) -> bool;
    // 未認証・取得中・CSV 処理中など、ファイル選択を受け付けられない状態の判定。
    // 空状態の取り込み CTA とも同じ条件にするため認証も含める
    fn input_disabled(&self) -> bool;
    fn save_disabled(&self, meta: &CsvTabMeta) -> bool {
        meta.busy()
    }
    fn db_count(&self) -> usize;
    fn delete_disabled(&self, meta: &CsvTabMeta) -> bool {
        meta.saving || meta.deleting
    }

    fn select_file(&self, file: web_sys::File);
    fn save_csv(&self);
    fn open_delete_confirm(&self);
}

// 空状態の「CSVを取り込む」から隠しファイル入力のファイル選択を開く
pub fn click_csv_input(input_id: &str) {
    use wasm_bindgen::JsCast;
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    if let Some(element) = document.get_element_by_id(input_id) {
        element.unchecked_ref::<web_sys::HtmlElement>().click();
    }
}

#[component]
pub fn CsvSection<S: CsvSource>(
    source: S,
    #[prop(optional)] expanded: Option<RwSignal<bool>>,
    #[prop(optional)] external_toggle: bool,
) -> impl IntoView {
    let selected_file_name = Memo::new(move |_| source.csv_meta().file_name.unwrap_or_default());
    let file_input_disabled = Memo::new(move |_| source.input_disabled());
    let has_csv_file =
        Memo::new(move |_| source.is_authenticated() && source.csv_meta().file_name.is_some());
    let save_label = Memo::new(move |_| source.csv_meta().save_label(source.save_action()));
    let save_disabled = Memo::new(move |_| source.save_disabled(&source.csv_meta()));
    let has_db_data = Memo::new(move |_| source.is_authenticated() && source.db_count() > 0);
    let delete_label = Memo::new(move |_| source.csv_meta().delete_label(source.db_count()));
    let delete_disabled = Memo::new(move |_| source.delete_disabled(&source.csv_meta()));
    let save_result = Memo::new(move |_| source.csv_meta().import_result);
    // 何件をどう保存するかは保存ボタンの直前に置き、押す直前の目に入るようにする
    let preview_notice = Memo::new(move |_| {
        let meta = source.csv_meta();
        if should_show_preview_notice(
            source.is_authenticated(),
            meta.file_name.is_some(),
            meta.previewing,
        ) {
            meta.preview.map(|preview| {
                (
                    preview_notice_text(preview.valid_rows, source.save_action()),
                    preview.errors,
                )
            })
        } else {
            None
        }
    });

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
            preview_notice=preview_notice
            mode_label=source.mode_label()
            toggle_testid=source.toggle_testid()
            body_id=source.body_id()
            section_class=source.section_class()
            expanded=expanded
            external_toggle=external_toggle
        />
    }
}
