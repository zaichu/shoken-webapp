use super::TAB_IDS;
use super::main_content::ReceiptsMainContent;
use crate::features::receipts::{
    ReceiptTabData, ReceiptsStore, ReceiptsTab, TabState, truncated_list_warning,
};
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;

pub(crate) fn empty_tab_data() -> ReceiptTabData {
    ReceiptTabData {
        rows: Vec::new(),
        summary: None,
        truncated: false,
    }
}

/// 開閉トグル(パネル内)から aria-controls で参照する aside の id
pub(crate) fn utility_rail_id(tab: ReceiptsTab) -> String {
    format!("receipt-utility-rail-{}", TAB_IDS[tab as usize])
}

// 中央列のタブ別の中身。タブボタンはページ直下に残し、切替でフォーカスを失わない
#[component]
pub(crate) fn ReceiptMainColumn(store: ReceiptsStore, tab: ReceiptsTab) -> impl IntoView {
    let main_store = store;
    let alert_store = store;
    let panel_state = Memo::new(move |_| store.tab_state(tab));
    view! {
        // 裏再取得・CSV の失敗は一覧を消さずに知らせる。パネルが畳まれても見えるよう表の上に出す
        {move || {
            let Some(message) = alert_store
                .csv_meta(tab)
                .error
                .or_else(|| alert_store.refresh_error(tab))
            else {
                return ().into_any();
            };
            view! {
                <div class="no-print">
                    <Alert variant=AlertVariant::Danger>
                        <strong>"エラー:"</strong>
                        " "
                        {message}
                    </Alert>
                </div>
            }
                .into_any()
        }}
        // パネルを畳んでも見えるよう、件数上限の警告は表の上(パネルの外)に出す
        {move || match panel_state.get() {
            TabState::Ready(data) if data.truncated => {
                view! {
                    <div>
                        <Alert variant=AlertVariant::Warning>
                            {truncated_list_warning()}
                        </Alert>
                    </div>
                }
                    .into_any()
            }
            _ => ().into_any(),
        }}
        {move || match panel_state.get() {
            TabState::Loading => {
                view! { <ListSkeleton variant=ListSkeletonVariant::Table /> }.into_any()
            }
            TabState::Ready(data) => {
                view! {
                    <ReceiptsMainContent
                        store=main_store
                        tab=tab
                        data=data
                    />
                }
                .into_any()
            }
            TabState::Failed(message) => {
                let retry_store = main_store;
                let preview = main_store.has_csv_preview(tab).then(|| {
                    view! {
                        <div class="mt-4">
                            <ReceiptsMainContent
                                store=main_store
                                tab=tab
                                data=empty_tab_data()
                            />
                        </div>
                    }
                });
                view! {
                    <ListLoadError message=message on_retry=move || retry_store.reload(tab) />
                    {preview}
                }
                    .into_any()
            }
        }}
    }
}
