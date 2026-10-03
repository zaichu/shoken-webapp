use super::search_card::AssetBalanceSearchCard;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::csv_store::AssetBalanceCsvStore;
use crate::features::asset_balance::review_prompt::generate_asset_review_prompt;
use crate::features::asset_balance::search::asset_balance_search_options;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::csv_section::CsvSection;
use crate::ui::elements::{Alert, AlertVariant};
use crate::ui::security_link::try_copy_to_clipboard;
use leptos::prelude::*;

#[derive(Clone, Copy)]
enum ReviewCopyStatus {
    Idle,
    Success,
    Error,
}

#[component]
pub(crate) fn AssetBalancePanelContent(
    view_csv: AssetBalanceCsvStore,
    search_query: RwSignal<String>,
    rows: impl Fn() -> Vec<AssetBalanceRow> + 'static + Send + Sync + Clone,
    facets: impl Fn() -> Option<crate::api::dto::SearchFacets> + 'static + Send + Sync,
    has_csv_file: impl Fn() -> bool + 'static + Send + Sync,
    warning: impl Fn() -> Option<String> + 'static + Send + Sync,
) -> impl IntoView {
    let rows_clone = rows.clone();
    let options = Memo::new(move |_| {
        asset_balance_search_options(&rows_clone(), facets().as_ref(), has_csv_file())
    });
    let review_rows = rows.clone();

    // パネル本体の直接の子は CSV → 検索 → 見直し促進カードの順で並べる
    view! {
        <div>
            <CsvSection source=view_csv />
            {move || {
                warning().map(|text| {
                    view! {
                        <div class="px-5 py-4">
                            <Alert variant=AlertVariant::Warning>{text}</Alert>
                        </div>
                    }
                })
            }}
            <AssetBalanceSearchCard query=search_query options=options />
            {move || {
                let has_rows = !review_rows().is_empty();
                has_rows.then(|| view! {
                    <AssetReviewPromptCard rows=review_rows() />
                })
            }}
        </div>
    }
}

#[component]
fn AssetReviewPromptCard(rows: Vec<AssetBalanceRow>) -> impl IntoView {
    let status = RwSignal::new(ReviewCopyStatus::Idle);
    let click_generation = RwSignal::new(0u64);
    let disabled = rows.is_empty();
    let label = move || match status.get() {
        ReviewCopyStatus::Success => "コピーしました！",
        ReviewCopyStatus::Error => "コピーに失敗しました",
        ReviewCopyStatus::Idle => "AI総評プロンプトをコピー",
    };
    let on_click = move |_| {
        let rows = rows.clone();
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
        <div class="px-5 py-4" data-testid="asset-review-prompt-card">
            <p class="mb-2 text-xs font-medium text-text-quiet">"AI総評プロンプト"</p>
            <Button
                variant=ButtonVariant::Prompt
                class="no-print"
                aria_label=move || label().to_string()
                disabled=disabled
                on_click=on_click
            >
                {label}
            </Button>
        </div>
    }
}
