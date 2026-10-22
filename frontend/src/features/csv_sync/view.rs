use super::{KindStatus, use_csv_sync};
use crate::session::use_session;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::labels;
use crate::ui::state::{Alert, AlertVariant};
use leptos::prelude::*;

/// Chrome ウェブストアの拡張機能ページ(公開版の ID)
const STORE_URL: &str = "https://chromewebstore.google.com/detail/hneplimfghlcckeielonpeklmjagmceh";

/// 「CSV 最新取得」の主ボタン。見出し右のボタン群と CSV 管理のパネルで使う。
/// ログイン中だけ出す(拡張機能が取る CSV は本人のデータのため)
#[component]
pub(crate) fn CsvSyncButton() -> impl IntoView {
    let session = use_session();
    let store = use_csv_sync();
    view! {
        <Show when=move || session.user.get().is_some()>
            <Button
                variant=ButtonVariant::Primary(ButtonSize::Sm)
                testid="csv-sync-button"
                disabled=move || store.running()
                on_click=move |_| store.start()
            >
                {move || {
                    if store.running() {
                        labels::CSV_SYNC_RUNNING
                    } else {
                        labels::CSV_FETCH_LATEST
                    }
                }}
            </Button>
        </Show>
    }
}

/// 資産管理・取引明細の本文上部に出す進行表示。何も動いていない間は何も出さない
#[component]
pub(crate) fn CsvSyncStatus() -> impl IntoView {
    let store = use_csv_sync();
    view! {
        {move || {
            store.visible().then(|| {
                view! {
                    <section
                        class="mb-3 rounded-lg border border-border bg-surface-sunken p-4"
                        data-testid="csv-sync-status"
                    >
                        <CsvSyncBody />
                    </section>
                }
            })
        }}
    }
}

/// CSV 管理に常設する「CSV 最新取得」の区画。ボタンと進行表示を一緒に置く
#[component]
pub(crate) fn CsvSyncPanel() -> impl IntoView {
    view! {
        <section
            class="rounded-lg border border-border bg-surface-sunken p-4"
            data-testid="csv-sync-panel"
        >
            <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div>
                    <h2 class="text-sm font-bold text-text">{labels::CSV_FETCH_LATEST}</h2>
                    <p class="mt-0.5 text-xs text-text-soft">{labels::CSV_SYNC_PANEL_HINT}</p>
                </div>
                <CsvSyncButton />
            </div>
            <CsvSyncBody />
        </section>
    }
}

/// 進行・結果・案内の本体。受け取った CSV の中身は表示しない(個人情報)
#[component]
fn CsvSyncBody() -> impl IntoView {
    let store = use_csv_sync();
    move || {
        let state = store.state();
        view! {
            {state.login_wait.then(|| {
                view! { <Alert variant=AlertVariant::Warning>{labels::CSV_SYNC_LOGIN_WAIT}</Alert> }
            })}
            {(!state.kinds.is_empty() && (state.running || state.finished)).then(|| {
                view! {
                    <ul class="mt-2 divide-y divide-border-subtle" data-testid="csv-sync-kinds">
                        {state
                            .kinds
                            .iter()
                            .map(|(kind, status)| {
                                let status_text = match status {
                                    KindStatus::Waiting => labels::CSV_SYNC_WAITING.to_string(),
                                    KindStatus::Fetching => labels::CSV_SYNC_RUNNING.to_string(),
                                    KindStatus::Done { lines } => labels::csv_received(*lines),
                                    KindStatus::Failed { reason } => {
                                        labels::csv_sync_failed(reason)
                                    }
                                };
                                let status_class = match status {
                                    KindStatus::Failed { .. } => "text-negative-vivid",
                                    _ => "text-text-muted",
                                };
                                view! {
                                    <li
                                        class="flex items-baseline justify-between gap-4 py-1.5 text-sm"
                                        data-testid=format!("csv-sync-kind-{}", kind.request_value())
                                    >
                                        <span class="text-text">{kind.label()}</span>
                                        <span class=status_class>{status_text}</span>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
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
