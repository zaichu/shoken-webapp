use super::chart::ChartList;
use super::search_bar::AssetBalanceSearchBar;
use super::summary::{ChartItem, PortfolioComposition, PortfolioSummary};
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::portfolio::chart_plan;
use crate::features::asset_balance::store::csv::csv_status_text;
use crate::features::asset_balance::store::{FilteredPortfolio, filtered_portfolio};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::Generation;
use crate::support::csv_flow::CsvTabMeta;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::ui::card::{Card, CardVariant};
use crate::ui::csv_preview::CsvPreviewBanner;
use crate::ui::csv_section::click_csv_input;
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use crate::ui::state::LoadingStrip;
use leptos::prelude::*;

#[component]
pub(crate) fn AssetBalanceMainContent(
    state: CsvTabMeta,
    rows: Vec<AssetBalanceRow>,
    fallback_totals: Option<AssetBalanceSummary>,
    has_csv_file: bool,
    csv_input_disabled: Memo<bool>,
    search_query: RwSignal<String>,
    dividends: RwSignal<DividendMaps>,
    show_all: RwSignal<bool>,
    composition_open: RwSignal<bool>,
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
    // 絞り込みはこの signal の1か所だけ。表示上限の適用より前に全取得行へかかる
    let filtered = RwSignal::new(filtered_portfolio(
        &rows,
        &search_query.get_untracked(),
        lookup,
        generation,
        has_csv_file,
    ));
    Effect::new(move |_| {
        filtered.set(filtered_portfolio(
            &rows,
            &search_query.get(),
            lookup,
            generation,
            has_csv_file,
        ));
    });
    let no_rows = move || total_count == 0;
    view! {
        {status.map(|text| view! { <LoadingStrip text=text.to_string() /> })}
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
            })
        }}
        {(preview_active && !no_rows()).then(|| {
            view! {
                <CsvPreviewBanner description="一覧は取り込むファイルの内容です。保存するまで登録済みのデータは変わりません。" />
            }
        })}
        // 入力欄は query 変更で作り直されるブロックの外に置く(再生成でフォーカスが落ちるため)
        <section id="assetbalance-list" hidden=no_rows>
            {move || {
                // query の変更は必ず Effect 経由で filtered の再通知を起こすので、ここでは追跡しない
                let is_filtered = !search_query.get_untracked().is_empty();
                view! {
                    <PortfolioSummary
                        filtered=filtered.get()
                        fallback_totals=fallback_totals.clone()
                        total_count=total_count
                        is_filtered=is_filtered
                        on_clear_filter=move || {
                            search_query.set(String::new())
                        }
                        dividends=dividends
                        show_all=show_all
                        preview=has_csv_file
                    />
                }
            }}
            <AssetBalanceSearchBar
                query=search_query
                filtered_count=Signal::derive(move || filtered.with(|f| f.views.len()))
                total_count=total_count
            />
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
