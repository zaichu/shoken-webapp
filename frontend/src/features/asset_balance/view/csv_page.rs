use super::{AssetBalanceState, use_asset_balance_state};
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::review_prompt::generate_asset_review_prompt;
use crate::features::asset_balance::store::csv::{
    AssetBalanceCsvStore, csv_preview_rows, resolve_asset_balance,
};
use crate::features::asset_balance::store::filtered_portfolio;
use crate::session::{SessionStore, use_session};
use crate::support::csv_flow::CsvTabMeta;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::csv_page::{CsvManagePage, CsvSource};
use crate::ui::security_link::try_copy_to_clipboard;
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

    fn mode_label(&self) -> &'static str {
        "全件置換"
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

    fn discard_csv(&self) {
        AssetBalanceCsvStore::discard_csv(self)
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
                subject="資産管理"
                back_href="/assetbalance"
    delete_href=String::new()
                preview=view! { <AssetCsvPreview state=state session=session /> }.into_any()
                extra=view! {
                    <AssetReviewPromptCard rows=move || review_rows(review_state, session) />
                }
                .into_any()
            />
        }
}

/// 旧削除ページ URL へのアクセスを CSV 管理ページへリダイレクトする
#[component]
fn CsvPreviewWrapper(state: AssetBalanceState, _session: SessionStore) -> impl IntoView {
    move || {
        let csv_state = state.csv.csv_state();
        let rows = csv_preview_rows(&csv_state);
        if rows.is_empty() {
            return None;
        }
        Some(view! { <AssetBalanceCsvPreview rows=rows /> })
    }
}

#[component]
pub fn AssetBalanceCsvDeletePage() -> impl IntoView {
    leptos::task::spawn_local(async move {
        if let Some(window) = web_sys::window() {
            window.location().set_href("/assetbalance/csv").ok();
        }
    });
    view! { <div>"CSV管理ページへ移動します..."</div> }
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
        let generation = click_generation.get() + 1;
        click_generation.set(generation);
        leptos::task::spawn_local(async move {
            let ok = try_copy_to_clipboard(generate_asset_review_prompt(&rows)).await;
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
