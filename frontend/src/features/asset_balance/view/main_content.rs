use super::chart::ChartList;
use super::summary::{ChartItem, PortfolioSummary};
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::{AssetBalanceCsvRow, AssetBalanceRow};
use crate::features::asset_balance::csv_store::csv_status_text;
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::portfolio::chart_plan;
use crate::features::asset_balance::search::clear_search_query;
use crate::features::asset_balance::store::{filtered_portfolio, FilteredPortfolio};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::Generation;
use crate::support::csv_flow::CsvTabState;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::csv_preview::CsvPreviewBanner;
use crate::ui::csv_section::click_csv_input;
use crate::ui::elements::LoadingStrip;
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use leptos::prelude::*;

#[component]
pub(crate) fn AssetBalanceMainContent(
    state: CsvTabState<AssetBalanceCsvRow>,
    rows: Vec<AssetBalanceRow>,
    summary: Option<AssetBalanceSummary>,
    has_csv_file: bool,
    csv_input_disabled: Memo<bool>,
    search_query: RwSignal<String>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: Generation,
) -> impl IntoView {
    if state.previewing {
        show_all.set(false);
        return view! {
            <LoadingStrip text="CSVファイルを解析しています...".to_string() />
        }
        .into_any();
    }
    let total_count = rows.len();
    let status = csv_status_text(&state);
    let preview_active = state.has_preview_rows();
    view! {
        {status.map(|text| view! { <LoadingStrip text=text.to_string() /> })}
        {move || {
            let query = search_query.get();
            if rows.is_empty() && query.is_empty() {
                show_all.set(false);
                return view! {
                    <Card variant=CardVariant::Soft class="mb-3">
                        <div>
                            <EmptyState
                                title="資産管理データがありません"
                                description="CSVファイルをインポートするか、データを登録してください。"
                                icon=EmptyStateIcon::Tray
                            >
                                <div class="mt-4">
                                    <Button
                                        variant=ButtonVariant::Primary(ButtonSize::Md)
                                        disabled=move || csv_input_disabled.get()
                                        aria_disabled=move || csv_input_disabled.get()
                                        on_click=move |_| {
                                            click_csv_input("csv-file-input-assetbalance")
                                        }
                                    >
                                        "CSVを取り込む"
                                    </Button>
                                </div>
                            </EmptyState>
                        </div>
                    </Card>
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
            view! {
                {preview_active.then(|| {
                    view! {
                        <CsvPreviewBanner description="一覧は取り込むファイルの内容です。保存するまで登録済みのデータは変わりません。" />
                    }
                })}
                <section id="assetbalance-list">
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
                    {(!chart_items.is_empty()).then(|| {
                        view! {
                            <div data-testid="portfolio-pie-chart">
                                <ChartList items=chart_items dividends=dividends show_all=show_all />
                            </div>
                        }
                    })}
                </section>
            }
                .into_any()
        }}
    }
    .into_any()
}
