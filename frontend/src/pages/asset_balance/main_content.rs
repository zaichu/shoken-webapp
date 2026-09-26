use super::csv::csv_status_text;
use super::data::{filtered_portfolio, FilteredPortfolio};
use super::summary::PortfolioSummary;
use crate::asset_balance::csv::AssetBalanceCsvRow;
use crate::asset_balance::lookup::AssetBalanceLookupStore;
use crate::asset_balance::search::clear_search_query;
use crate::csv_flow::CsvTabState;
use crate::dividend_per_share::DividendMaps;
use crate::dto::{AssetBalance, AssetBalanceSummary};
use leptos::prelude::*;

#[component]
pub(crate) fn AssetBalanceMainContent(
    state: CsvTabState<AssetBalanceCsvRow>,
    rows: Vec<AssetBalance>,
    summary: Option<AssetBalanceSummary>,
    has_csv_file: bool,
    search_query: RwSignal<String>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: u64,
) -> impl IntoView {
    if state.previewing {
        show_all.set(false);
        return view! { <CsvStatusMessage text="CSVファイルを解析しています..." /> }.into_any();
    }
    let total_count = rows.len();
    let status = csv_status_text(&state);
    view! {
        {status.map(|text| view! { <CsvStatusMessage text=text /> })}
        {move || {
            let query = search_query.get();
            if rows.is_empty() && query.is_empty() {
                show_all.set(false);
                return view! {
                    <div class="mb-3 overflow-hidden rounded-xl border border-slate-950/10 bg-white/90 shadow-[0_14px_38px_-32px_rgba(15,23,42,0.85)]">
                        <div>
                            <div class="empty-state">
                                <h3 class="text-base font-black text-slate-950">
                                    "資産管理データがありません"
                                </h3>
                                <p class="mt-1.5 max-w-md text-sm font-medium text-slate-600">
                                    "CSVファイルをインポートするか、データを登録してください。"
                                </p>
                            </div>
                        </div>
                    </div>
                }
                    .into_any();
            }
            let FilteredPortfolio { views, summary } = filtered_portfolio(
                &rows,
                summary.clone(),
                &query,
                lookup,
                generation,
                has_csv_file,
            );
            view! {
                <PortfolioSummary
                    views=views
                    total_count=total_count
                    is_filtered=!query.is_empty()
                    on_clear_filter=move || {
                        search_query.set(clear_search_query())
                    }
                    summary=summary
                    dividends=dividends
                    show_all=show_all
                />
            }
                .into_any()
        }}
    }
    .into_any()
}

#[component]
pub(crate) fn CsvStatusMessage(text: &'static str) -> impl IntoView {
    view! {
        <section class="px-5 py-4" role="status" aria-live="polite" aria-atomic="true">
            <div class="flex items-center gap-2 text-slate-600">
                <svg
                    class="animate-spin h-4 w-4"
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    aria-hidden="true"
                >
                    <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                    />
                    <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                </svg>
                <p class="text-sm">{text}</p>
            </div>
        </section>
    }
}
