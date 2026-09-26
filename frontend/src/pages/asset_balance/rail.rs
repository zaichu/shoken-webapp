use super::search_card::AssetBalanceSearchCard;
use crate::asset_balance::review_prompt::generate_asset_review_prompt;
use crate::asset_balance::search::asset_balance_search_options;
use crate::components::security_link::try_copy_to_clipboard;
use crate::dto::{AssetBalance, SearchFacets};
use leptos::prelude::*;

#[component]
pub(crate) fn AssetBalanceRailExtras(
    rows: Vec<AssetBalance>,
    facets: Option<SearchFacets>,
    has_csv_file: bool,
    warning: Option<String>,
    search_query: RwSignal<String>,
) -> impl IntoView {
    let options = asset_balance_search_options(&rows, facets.as_ref(), has_csv_file);
    let has_rows = !rows.is_empty();
    view! {
        {warning.map(|text| {
            view! {
                <div class="px-5 py-4" role="status" aria-live="polite">
                    <div
                        class="rounded-lg border border-amber-200 bg-amber-50 px-4 py-3 text-sm font-medium text-amber-900 shadow-sm"
                        role="alert"
                    >
                        {text}
                    </div>
                </div>
            }
        })}
        {has_rows.then(|| view! { <AssetBalanceSearchCard query=search_query options=options /> })}
        <AssetReviewPromptCard rows=rows />
    }
}

#[derive(Clone, Copy)]
enum ReviewCopyStatus {
    Idle,
    Success,
    Error,
}

#[component]
fn AssetReviewPromptCard(rows: Vec<AssetBalance>) -> impl IntoView {
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
            <p class="mb-2 text-xs font-medium text-secondary">"AI総評プロンプト"</p>
            <button
                type="button"
                class="review-prompt-button no-print"
                aria-label=label
                disabled=disabled
                on:click=on_click
            >
                {label}
            </button>
        </div>
    }
}
