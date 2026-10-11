use super::chart::ChartList;
use super::summary::{ChartItem, PortfolioComposition, PortfolioSummary};
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::portfolio::chart_plan;
use crate::features::asset_balance::store::{FilteredPortfolio, filtered_portfolio};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::Generation;
use crate::ui::button::{ButtonSize, ButtonVariant, LinkButton};
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use leptos::prelude::*;

#[component]
pub(crate) fn AssetBalanceMainContent(
    rows: Vec<AssetBalanceRow>,
    fallback_totals: Option<AssetBalanceSummary>,
    search_query: RwSignal<String>,
    search_count: RwSignal<(usize, usize)>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
    composition_open: RwSignal<bool>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: Generation,
) -> impl IntoView {
    let total_count = rows.len();
    // 絞り込みはこの signal の1か所だけ。表示上限の適用より前に全取得行へかかる
    let initial = filtered_portfolio(&rows, &search_query.get_untracked(), lookup, generation);
    search_count.set((initial.views.len(), total_count));
    let filtered = RwSignal::new(initial);
    Effect::new(move |_| {
        let next = filtered_portfolio(&rows, &search_query.get(), lookup, generation);
        search_count.set((next.views.len(), total_count));
        filtered.set(next);
    });
    let no_rows = move || total_count == 0;
    view! {
        {move || {
            no_rows().then(|| {
                show_all.set(false);
                view! {
                    <Card variant=CardVariant::Soft class="mb-3">
                        <div>
                            <EmptyState
                                title="資産管理データがありません"
                                description="CSVファイルをインポートするか、データを登録してください。"
                                icon=EmptyStateIcon::Tray
                            >
                                <div class="mt-4">
                                    <LinkButton
                                        variant=ButtonVariant::Primary(ButtonSize::Md)
                                        href="/assetbalance/csv"
                                    >
                                        "CSVを取り込む"
                                    </LinkButton>
                                </div>
                            </EmptyState>
                        </div>
                    </Card>
                }
            })
        }}
        // 入力欄は query 変更で作り直されるブロックの外に置く(再生成でフォーカスが落ちるため)
        <section id="assetbalance-list" hidden=no_rows>
            {move || {
                // query の変更は必ず Effect 経由で filtered の再通知を起こすので、ここでは追跡しない
                view! {
                    <PortfolioSummary
                        filtered=filtered.get()
                        fallback_totals=fallback_totals.clone()
                        on_clear_filter=move || {
                            search_query.set(String::new())
                        }
                        dividends=dividends
                        show_all=show_all
                    />
                }
            }}
            {move || {
                let FilteredPortfolio { views, .. } = filtered.get();
                let chart_values: Vec<f64> = views.iter().map(|view| view.purchase).collect();
                let plan = chart_plan(&chart_values);
                let chart_items: Vec<ChartItem> = plan
                    .order
                    .iter()
                    .zip(&plan.percentages)
                    .map(|(&index, &percentage)| ChartItem {
                        view: views[index].clone(),
                        percentage,
                    })
                    .collect();
                let list_items = chart_items.clone();
                view! {
                    {(!chart_items.is_empty()).then(|| {
                        view! {
                            <div data-testid="portfolio-pie-chart">
                                <ChartList items=list_items dividends=dividends show_all=show_all />
                            </div>
                        }
                    })}
                    // 取得額構成比は一覧の後ろに置き、初期は折りたたむ
                    <PortfolioComposition
                        items=chart_items
                        open=composition_open
                        show_all=show_all
                    />
                }
            }}
        </section>
    }
    .into_any()
}
