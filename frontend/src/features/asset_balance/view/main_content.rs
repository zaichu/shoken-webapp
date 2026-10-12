use super::holdings_list::HoldingsList;
use super::sort_controls::SortControls;
use super::summary::PortfolioSummary;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::holdings::holding_dividend;
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::portfolio::{SortKey, SortOrder, sort_holdings};
use crate::features::asset_balance::store::{FilteredPortfolio, filtered_portfolio};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::Generation;
use leptos::prelude::*;

#[component]
pub(crate) fn AssetBalanceMainContent(
    rows: Vec<AssetBalanceRow>,
    fallback_totals: Option<crate::api::dto::AssetBalanceSummary>,
    search_query: RwSignal<String>,
    search_count: RwSignal<(usize, usize)>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: Generation,
) -> impl IntoView {
    let total_count = rows.len();
    let initial = filtered_portfolio(&rows, &search_query.get_untracked(), lookup, generation);
    search_count.set((initial.views.len(), total_count));
    let filtered = RwSignal::new(initial);
    Effect::new(move |_| {
        let next = filtered_portfolio(&rows, &search_query.get(), lookup, generation);
        search_count.set((next.views.len(), total_count));
        filtered.set(next);
    });
    let sort_key = RwSignal::new(SortKey::MarketValue);
    let sort_order = RwSignal::new(SortOrder::Desc);
    let no_rows = move || total_count == 0;
    view! {
        {move || {
            no_rows().then(|| {
                view! {
                    <div class="rounded-lg border border-border bg-surface p-6">
                        <div class="text-center">
                            <p class="font-medium text-text">"資産管理データがありません"</p>
                            <p class="mt-1 text-sm text-text-muted">
                                "CSVファイルをインポートするか、データを登録してください。"
                            </p>
                            <a
                                href="/assetbalance/csv"
                                class="mt-4 inline-flex items-center justify-center rounded-md border border-border-strong bg-surface px-4 py-2 text-sm font-bold text-text no-underline hover:border-border-bold hover:bg-surface-sunken"
                            >
                                "CSVを取り込む"
                            </a>
                        </div>
                    </div>
                }
            })
        }}
        <section id="assetbalance-list" hidden=no_rows>
            {move || {
                view! {
                    <PortfolioSummary
                        filtered=filtered.get()
                        fallback_totals=fallback_totals.clone()
                        on_clear_filter=move || {
                            search_query.set(String::new())
                        }
                        dividends=dividends
                    />
                }
            }}
            <div class="rounded-lg border border-border bg-surface">
                <div class="border-b border-border p-3">
                    <SortControls sort_key=sort_key sort_order=sort_order />
                </div>
                {move || {
                    let FilteredPortfolio { views, .. } = filtered.get();
                    let dividends_map = dividends.get();
                    let dividends_vec: Vec<_> = views
                        .iter()
                        .map(|view| holding_dividend(
                            &view.code,
                            view.shares,
                            view.average_price,
                            &dividends_map,
                        ))
                        .collect();
                    let inputs: Vec<_> = views
                        .iter()
                        .zip(dividends_vec.iter())
                        .map(|(view, dividend)| super::super::portfolio::SortInput {
                            code: view.code.as_str(),
                            valuation: view.into(),
                            dividend,
                        })
                        .collect();
                    let order = sort_holdings(inputs, sort_key.get(), sort_order.get());
                    let sorted: Vec<_> = order
                        .into_iter()
                        .map(|index| views[index].clone())
                        .collect();
                    view! { <HoldingsList views=sorted dividends=dividends /> }
                }}
            </div>
        </section>
    }
    .into_any()
}
