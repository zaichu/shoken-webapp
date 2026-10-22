use super::{AssetBalanceState, use_asset_balance_state};
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::fund::AssetKind;
use crate::features::asset_balance::review_prompt::generate_asset_review_prompt;
use crate::features::asset_balance::store::csv::{
    AssetBalanceCsvStore, csv_preview_rows, resolve_asset_balance,
};
use crate::features::asset_balance::view::{AssetBalanceCsvPreview, FundBalanceCsvPreview};
use crate::session::{SessionStore, use_session};
use crate::support::csv_flow::CsvTabMeta;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::confirm_modal::ConfirmDeleteModal;
use crate::ui::csv_page::{CsvManagePage, CsvSource};
use crate::ui::security_link::try_copy_to_clipboard;
use crate::ui::state::{Alert, AlertVariant};
use leptos::prelude::*;

impl CsvSource for AssetBalanceCsvStore {
    fn input_id(&self) -> &'static str {
        "csv-file-input-assetbalance"
    }

    fn save_action(&self) -> &'static str {
        "全件置換で保存"
    }

    fn save_mode_label(&self) -> &'static str {
        "全件置換"
    }

    // 追加保存と違い、置換の対象が全件だとボタン上でも分かるようにする
    fn save_button_label(&self, valid_rows: Option<usize>) -> String {
        match valid_rows {
            Some(rows) => format!("{rows}件 全件置換で保存"),
            None => "全件置換で保存".to_string(),
        }
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

    fn errors_block_save(&self) -> bool {
        true
    }

    fn csv_format_hint(&self) -> &'static str {
        "対応形式: 資産残高CSV(見出し行の先頭は「銘柄コード」)"
    }

    fn db_count(&self) -> usize {
        AssetBalanceCsvStore::db_count(self)
    }

    fn delete_count_ready(&self) -> bool {
        self.saved_count_ready()
    }

    fn delete_count_error(&self) -> Option<String> {
        self.saved_count_error()
    }

    fn date_range(&self) -> Option<(String, String)> {
        AssetBalanceCsvStore::date_range(self)
    }

    fn refresh_for_delete(&self) {
        AssetBalanceCsvStore::refresh_for_delete(self)
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

    fn confirm_delete_all(&self) {
        AssetBalanceCsvStore::confirm_delete_all(self)
    }
}

#[component]
pub fn AssetBalanceCsvPage() -> impl IntoView {
    let session = use_session();
    let state = use_asset_balance_state(session);
    let source = state.csv;
    let review_state = state;
    view! {
        <CsvManagePage
            source=source
            subject="資産管理(国内株式)"
            delete_button_label="国内株式をすべて削除…"
            back_href="/assetbalance"
            preview=view! { <CsvPreviewWrapper state=state _session=session /> }.into_any()
            extra=view! {
                <AssetReviewPromptCard rows=move || review_rows(review_state, session) />
            }
            .into_any()
            saved_extra=view! { <FundSavedCard source=source /> }.into_any()
            top_extra=view! { <crate::features::csv_sync::CsvSyncPanel /> }.into_any()
        />
    }
}

#[component]
fn CsvPreviewWrapper(state: AssetBalanceState, _session: SessionStore) -> impl IntoView {
    move || {
        if state.csv.csv_kind.get() == AssetKind::Fund {
            let rows = csv_preview_rows(&state.csv.fund_csv_state());
            return (!rows.is_empty())
                .then(|| view! { <FundBalanceCsvPreview rows=rows /> }.into_any());
        }
        let rows = csv_preview_rows(&state.csv.csv_state());
        (!rows.is_empty()).then(|| view! { <AssetBalanceCsvPreview rows=rows /> }.into_any())
    }
}

/// 投資信託の保存済みデータカード。株式側と同じ構造で、削除は投資信託だけを対象にする
#[component]
fn FundSavedCard(source: AssetBalanceCsvStore) -> impl IntoView {
    let db_count = Memo::new(move |_| source.fund_db_count());
    let count_ready = Memo::new(move |_| source.fund_saved_count_ready());
    let count_error = Memo::new(move |_| source.fund_saved_count_error());
    let date_range = Memo::new(move |_| source.fund_date_range());
    let show_confirm = RwSignal::new(false);
    let delete_error = RwSignal::new(None::<String>);
    let deleting = Memo::new(move |_| source.fund_csv_state().meta().deleting);
    let modal_error = Signal::derive(move || delete_error.get().or_else(|| count_error.get()));
    let open_confirm = move |_| {
        delete_error.set(None);
        source.refresh_fund_for_delete();
        source.fund_open_delete_confirm();
        show_confirm.set(true);
    };
    let on_confirm = move || {
        delete_error.set(None);
        source.fund_open_delete_confirm();
        source.fund_confirm_delete_all();
    };
    let on_cancel = move || {
        show_confirm.set(false);
        delete_error.set(None);
    };
    // 削除要求が終わったときにモーダルを閉じ、失敗はエラー表示に回す
    let was_deleting = RwSignal::new(false);
    Effect::new(move |_| {
        let now = deleting.get();
        let was = was_deleting.get_untracked();
        was_deleting.set(now);
        if !was || now || !show_confirm.get() {
            return;
        }
        match source.fund_csv_state().meta().error {
            Some(err) => delete_error.set(Some(err)),
            None => show_confirm.set(false),
        }
    });
    view! {
        <section>
            <div class="bg-surface-sunken rounded-lg border border-border p-4">
                <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                    <div>
                        <p class="text-xs text-text-soft">"保存済みの投資信託"</p>
                        <div class="flex items-baseline gap-2 mt-0.5">
                            <span class="text-2xl font-bold text-text" data-testid="fund-saved-count">
                                {move || {
                                    if count_ready.get() {
                                        format!("{}件", db_count.get())
                                    } else if count_error.get().is_some() {
                                        "取得できませんでした".to_string()
                                    } else {
                                        "取得中...".to_string()
                                    }
                                }}
                            </span>
                            {move || {
                                date_range.get().map(|(min, max)| {
                                    view! {
                                        <span class="text-sm font-medium text-text-muted ml-2">
                                            {format!("{} 〜 {}", min, max)}
                                        </span>
                                    }
                                })
                            }}
                        </div>
                    </div>
                    <Button
                        variant=ButtonVariant::Secondary(crate::ui::button::ButtonSize::Md)
                        class="text-negative border-negative border"
                        disabled=move || {
                            !count_ready.get()
                                || db_count.get() == 0
                                || source.csv_busy()
                                || source.fund_csv_state().meta().busy()
                        }
                        on_click=open_confirm
                    >
                        "投資信託をすべて削除…"
                    </Button>
                </div>
                {move || {
                    count_error.get().map(|message| {
                        view! {
                            <div class="mt-3">
                                <Alert variant=AlertVariant::Danger>
                                    <strong>"エラー:"</strong>
                                    " "
                                    {message}
                                    <Button
                                        variant=ButtonVariant::Ghost
                                        class="ml-2 text-xs underline"
                                        on_click=move |_| source.refresh_fund_for_delete()
                                    >
                                        "再読み込み"
                                    </Button>
                                </Alert>
                            </div>
                        }
                    })
                }}
            </div>
        </section>
        {move || {
            show_confirm.get().then(|| {
                let count = db_count.get();
                let range_text = date_range
                    .get()
                    .map(|(min, max)| format!("{} 〜 {} の", min, max))
                    .unwrap_or_default();
                view! {
                    <ConfirmDeleteModal
                        title="投資信託を削除しますか?".to_string()
                        description=format!("{}データが、一覧と集計から消えます。", range_text)
                        item_count=count
                        confirm_label=format!("{}件を削除する", count)
                        loading=deleting
                        confirm_ready=Memo::new(move |_| deleting.get() || count_ready.get())
                        error=modal_error
                        on_confirm=on_confirm
                        on_cancel=on_cancel
                    />
                }
            })
        }}
    }
}

// プロンプトは表示対象と一致させる: CSV選択中はプレビュー行、なければ保存済み一覧
fn review_rows(state: AssetBalanceState, session: SessionStore) -> Vec<AssetBalanceRow> {
    let csv_state = state.csv.csv_state();
    if csv_state.file_name.is_some() {
        csv_preview_rows(&csv_state)
    } else {
        resolve_asset_balance(session.generation.get(), &state.balances.get())
            .map(|resolved| resolved.rows)
            .unwrap_or_default()
    }
}

#[derive(Clone, Copy)]
enum ReviewCopyStatus {
    Idle,
    Success,
    Error,
}

#[component]
fn AssetReviewPromptCard(
    rows: impl Fn() -> Vec<AssetBalanceRow> + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let status = RwSignal::new(ReviewCopyStatus::Idle);
    let click_generation = RwSignal::new(0u64);
    let disabled = Memo::new({
        let rows = rows.clone();
        move |_| rows().is_empty()
    });
    let label = move || match status.get() {
        ReviewCopyStatus::Success => "コピーしました！",
        ReviewCopyStatus::Error => "コピーに失敗しました",
        ReviewCopyStatus::Idle => "AI総評プロンプトをコピー",
    };
    let on_click = move |_| {
        let rows = rows();
        let pref = crate::settings::try_use_settings()
            .review_prompt
            .get_untracked();
        let generation = click_generation.get() + 1;
        click_generation.set(generation);
        leptos::task::spawn_local(async move {
            let ok = try_copy_to_clipboard(generate_asset_review_prompt(&rows, &pref)).await;
            if click_generation.get() != generation {
                return;
            }
            status.set(if ok {
                ReviewCopyStatus::Success
            } else {
                ReviewCopyStatus::Error
            });
            gloo_timers::future::TimeoutFuture::new(3_000).await;
            if click_generation.get() == generation {
                status.set(ReviewCopyStatus::Idle);
            }
        });
    };
    view! {
        <div data-testid="asset-review-prompt-card">
            <p class="mb-2 text-xs font-medium text-text-quiet">"AI総評プロンプト"</p>
            <Button
                variant=ButtonVariant::Prompt
                class="no-print"
                aria_label=move || label().to_string()
                disabled=move || disabled.get()
                on_click=on_click
            >
                {label}
            </Button>
        </div>
    }
}
