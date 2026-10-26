use super::use_csv_sync;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels;
use crate::ui::state::{Alert, AlertVariant};
use leptos::prelude::*;

/// Chrome ウェブストアの拡張機能ページ(公開版の ID)
const STORE_URL: &str = "https://chromewebstore.google.com/detail/hneplimfghlcckeielonpeklmjagmceh";

/// 確認と案内の本体。種類ごとの進行・結果は CSV 管理の表に出す
#[component]
pub(crate) fn CsvSyncBody() -> impl IntoView {
    let store = use_csv_sync();
    move || {
        let state = store.state();
        view! {
            {state.login_wait.then(|| {
                view! { <Alert variant=AlertVariant::Warning>{labels::CSV_SYNC_LOGIN_WAIT}</Alert> }
            })}
            {state.import.confirm.clone().map(|confirm| {
                let kind_label = confirm.kind.label();
                view! {
                    <div class="mt-2" data-testid="csv-sync-confirm">
                        <Alert variant=AlertVariant::Warning>
                            <p>
                                {labels::csv_sync_plunge_question(
                                    kind_label,
                                    confirm.new_rows,
                                    confirm.saved_rows,
                                )}
                            </p>
                            <div class="mt-2 flex gap-2">
                                <Button
                                    variant=ButtonVariant::Primary(ButtonSize::Sm)
                                    testid="csv-sync-confirm-yes"
                                    on_click=move |_| store.answer_confirm(true)
                                >
                                    {labels::YES}
                                </Button>
                                <Button
                                    variant=ButtonVariant::Secondary(ButtonSize::Sm)
                                    testid="csv-sync-confirm-no"
                                    on_click=move |_| store.answer_confirm(false)
                                >
                                    {labels::NO}
                                </Button>
                            </div>
                        </Alert>
                    </div>
                }
            })}
            {state.missing.then(|| {
                view! {
                    <div class="mt-2">
                        <Alert variant=AlertVariant::Warning>
                            <strong>{labels::CSV_SYNC_MISSING_TITLE}</strong>
                            <p class="mt-1 font-normal">
                                {labels::CSV_SYNC_MISSING_DESCRIPTION}
                                " "
                                <a
                                    href=STORE_URL
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    class="underline"
                                    aria-label=format!(
                                        "{}（新しいタブで開く）",
                                        labels::CSV_SYNC_STORE_LINK
                                    )
                                >
                                    {labels::CSV_SYNC_STORE_LINK}
                                </a>
                            </p>
                        </Alert>
                    </div>
                }
            })}
            {state.error.clone().map(|message| {
                view! {
                    <div class="mt-2">
                        <Alert variant=AlertVariant::Danger>
                            <strong>"エラー:"</strong>
                            " "
                            {message}
                        </Alert>
                    </div>
                }
            })}
            {state.needs_file_access_guide().then(|| {
                view! {
                    <div class="mt-2">
                        <Alert variant=AlertVariant::Warning>
                            <strong>{labels::CSV_SYNC_FILE_ACCESS_TITLE}</strong>
                            <ol class="mt-1 list-decimal pl-5 font-normal">
                                {labels::CSV_SYNC_FILE_ACCESS_STEPS
                                    .iter()
                                    .map(|step| view! { <li>{*step}</li> })
                                    .collect_view()}
                            </ol>
                        </Alert>
                    </div>
                }
            })}
        }
    }
}
