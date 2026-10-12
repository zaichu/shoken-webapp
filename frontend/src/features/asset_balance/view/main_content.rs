use super::asset_table::AssetHoldingsTable;
use super::holdings_list::HoldingsList;
use super::sort_controls::SortControls;
use super::summary::PortfolioSummary;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::holdings::{HoldingView, holding_dividend};
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::portfolio::{SortInput, SortKey, SortOrder, sort_holdings};
use crate::features::asset_balance::store::{FilteredPortfolio, filtered_portfolio};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::Generation;
use crate::ui::button::{Button, ButtonSize, ButtonVariant};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

fn sorted_views(
    views: &[HoldingView],
    dividends: &DividendMaps,
    key: SortKey,
    order: SortOrder,
) -> Vec<usize> {
    let dividends_vec: Vec<_> = views
        .iter()
        .map(|view| holding_dividend(&view.code, view.shares, view.average_price, dividends))
        .collect();
    let inputs: Vec<_> = views
        .iter()
        .zip(dividends_vec.iter())
        .map(|(view, dividend)| SortInput {
            code: view.code.as_str(),
            valuation: view.into(),
            dividend,
        })
        .collect();
    sort_holdings(inputs, key, order)
}

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
                    <div class="asset-panel-padded mx-4 lg:mx-0">
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
            <div class="asset-list-surface">
                <div class="asset-pc-toolbar no-print" role="search" aria-label="保有銘柄の検索">
                    <div class="search-input-frame w-80">
                        <svg aria-hidden="true" class="mx-2 h-4 w-4 shrink-0 self-center text-text-muted" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                            <circle cx="11" cy="11" r="7"></circle>
                            <path d="M20 20l-3.5-3.5"></path>
                        </svg>
                        <input
                            type="search"
                            class="search-input"
                            placeholder="銘柄名・コードで絞り込み"
                            aria-label="銘柄名・コードで絞り込み"
                            autocomplete="off"
                            data-testid="asset-search-input-pc"
                            enterkeyhint="search"
                            prop:value=move || search_query.get()
                            // IME 変換中の確定前の文字では絞り込まない。確定は compositionend で受ける
                            on:input=move |ev| {
                                if ev.unchecked_ref::<web_sys::InputEvent>().is_composing() {
                                    return;
                                }
                                search_query.set(event_target_value(&ev));
                            }
                            on:compositionend=move |ev| {
                                search_query.set(event_target_value(&ev))
                            }
                        />
                        {move || {
                            (!search_query.get().is_empty()).then(|| {
                                view! {
                                    <Button
                                        variant=ButtonVariant::SecondarySoft(ButtonSize::Xs)
                                        class="shrink-0 self-center".to_string()
                                        aria_label="検索を解除".to_string()
                                        testid="asset-search-clear-pc"
                                        on_click=move |_| search_query.set(String::new())
                                    >
                                        "消去"
                                    </Button>
                                }
                            })
                        }}
                    </div>
                    {move || {
                        if search_query.get().is_empty() {
                            view! {
                                <span class="shrink-0 text-xs text-text-muted">
                                    "列名を押すと並べ替え"
                                </span>
                            }
                            .into_any()
                        } else {
                            let (filtered, total) = search_count.get();
                            view! {
                                <span
                                    class="shrink-0 text-xs text-text-muted"
                                    data-testid="asset-search-count-pc"
                                >
                                    {format!("検索結果 {filtered} / {total}件")}
                                </span>
                            }
                            .into_any()
                        }
                    }}
                </div>
                <SortControls sort_key=sort_key sort_order=sort_order />
                <div class="hidden lg:block print:block">
                    {move || {
                        let FilteredPortfolio { views, .. } = filtered.get();
                        let order = sorted_views(
                            &views,
                            &dividends.get(),
                            sort_key.get(),
                            sort_order.get(),
                        );
                        let sorted: Vec<_> = order
                            .into_iter()
                            .map(|index| views[index].clone())
                            .collect();
                        view! {
                            <AssetHoldingsTable
                                views=sorted
                                dividends=dividends
                                sort_key=sort_key
                                sort_order=sort_order
                            />
                        }
                    }}
                </div>
                <div class="lg:hidden print:hidden">
                    {move || {
                        let FilteredPortfolio { views, .. } = filtered.get();
                        let key = sort_key.get();
                        let order = sorted_views(&views, &dividends.get(), key, sort_order.get());
                        let sorted: Vec<_> = order
                            .into_iter()
                            .map(|index| views[index].clone())
                            .collect();
                        let show_dividend = matches!(
                            key,
                            SortKey::Dividend | SortKey::Yield
                        );
                        view! {
                            <HoldingsList
                                views=sorted
                                dividends=dividends
                                show_dividend=show_dividend
                            />
                        }
                    }}
                </div>
            </div>
        </section>
    }
    .into_any()
}
