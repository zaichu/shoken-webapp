use super::all_list::{
    AssetMergedList, AssetMergedTable, UnifiedRow, sort_unified, unified_from_fund,
    unified_from_stock,
};
use super::asset_table::AssetHoldingsTable;
use super::fund_table::{FundHoldingsList, FundHoldingsTable};
use super::holdings_list::HoldingsList;
use super::search_bar::{AssetSearchBar, ConditionKind};
use super::summary::{
    AssetSummaryBand, all_basis_lines, fund_basis_lines, fund_notes, purchase_summary_tag,
    stock_basis_lines, stock_price_dates, stock_summary_values,
};
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::csv::AssetBalanceRow;
use crate::features::asset_balance::format::{price_date_is_outlier, price_date_representative};
use crate::features::asset_balance::fund::{
    AssetKind, AssetSearch, FundBalanceRow, FundSortKey, FundView, filter_fund_rows, fund_accounts,
    fund_courses, fund_totals, fund_view, sort_fund_views, text_only_search,
};
use crate::features::asset_balance::holdings::{
    HoldingView, holding_dividend, stock_daily_change_total,
};
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::model::ValuationInput;
use crate::features::asset_balance::portfolio::{SortInput, SortKey, SortOrder, sort_holdings};
use crate::features::asset_balance::search::stock_accounts;
use crate::features::asset_balance::store::csv::ResolvedFundBalance;
use crate::features::asset_balance::store::{all_portfolio_views, filtered_portfolio_views};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::Generation;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::labels::{self, CLEAR_SEARCH_CONDITIONS, NO_MATCH_TITLE};
use crate::ui::page_skeleton::{FilteredCount, SkeletonNote, SkeletonSearch, SkeletonSummary};
use leptos::prelude::*;

/// 絞り込みの件数。検索条件が無いときは注記に出さない
fn filtered_count(
    search: RwSignal<AssetSearch>,
    search_count: RwSignal<(usize, usize)>,
) -> Signal<FilteredCount> {
    Signal::derive(move || {
        let (filtered, total) = search_count.get();
        FilteredCount {
            filtered,
            total,
            filtering: !search.with(|search| search.is_empty()),
        }
    })
}

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
fn EmptyAssetPanel() -> impl IntoView {
    view! {
        <div class="asset-panel-padded mx-4 lg:mx-0">
            <div class="text-center">
                <p class="font-medium text-text">"資産管理データがありません"</p>
                <p class="mt-1 text-sm text-text-muted">
                    "CSVファイルをインポートするか、データを登録してください。"
                </p>
                <a
                    href="/csv"
                    class="mt-4 inline-flex items-center justify-center rounded-md border border-border-strong bg-surface px-4 py-2 text-sm font-bold text-text no-underline hover:border-border-bold hover:bg-surface-sunken"
                >
                    "CSVを取り込む"
                </a>
            </div>
        </div>
    }
}

/// 検索で全行が消えたときの表示
#[component]
fn EmptyFilteredPanel(
    title: &'static str,
    on_clear: impl Fn() + Copy + Send + 'static,
) -> impl IntoView {
    view! {
        <div class="asset-panel-padded mx-4 mt-3 lg:mx-0">
            <div class="text-center">
                <EmptyState title=title />
                <div class="mt-3 text-center">
                    <Button variant=ButtonVariant::Ghost on_click=move |_| on_clear()>
                        {CLEAR_SEARCH_CONDITIONS}
                    </Button>
                </div>
            </div>
        </div>
    }
}

/// 検索欄の入力欄に出す文言。3タブで同じ文言・同じ動詞(検索)を使う
const SEARCH_TARGET: &str = "銘柄・ファンド名・コード・口座";

/// 検索で全行が消えたときの文面。3タブで同じ文言を使う
const EMPTY_FILTERED_TITLE: &str = NO_MATCH_TITLE;

#[component]
fn StockContent(
    rows: Vec<AssetBalanceRow>,
    fallback_totals: Option<AssetBalanceSummary>,
    stock_note: String,
    search: RwSignal<AssetSearch>,
    search_count: RwSignal<(usize, usize)>,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: Generation,
) -> impl IntoView {
    let total_count = rows.len();
    let empty = rows.is_empty();
    let option_rows = rows.clone();
    // 全保有分の変換は検索語に依存させない。検索入力では絞り込みだけ再計算する
    let rows_for_all = rows.clone();
    let all_views = Memo::new(move |_| all_portfolio_views(&rows_for_all, lookup, generation));
    let rows_for_filtered = rows.clone();
    let filtered_views = Memo::new(move |_| {
        filtered_portfolio_views(&rows_for_filtered, &search.get(), lookup, generation)
    });
    search_count.set((filtered_views.get_untracked().len(), total_count));
    Effect::new(move |_| {
        search_count.set((filtered_views.with(|views| views.len()), total_count));
    });
    // 見出しの基準日と同じ代表を表の日付出し分けにも使う(絞り込み前の全行で決める)
    let date_rep = {
        let dates = stock_price_dates(&rows);
        price_date_representative(dates.into_iter()).map(str::to_string)
    };
    let account_options = Memo::new(move |_| stock_accounts(&option_rows));
    let course_options = Memo::new(|_| Vec::<String>::new());
    let filtered_count = filtered_count(search, search_count);
    // 集計は絞り込み前の全保有分で求める(検索中も全件のまま)
    let summary = Memo::new(move |_| {
        all_views
            .with(|views| stock_summary_values(views, fallback_totals.as_ref(), &dividends.get()))
    });
    view! {
        {empty.then(|| view! { <EmptyAssetPanel /> }.into_any())}
        <section id="assetbalance-stock-list" hidden=empty>
            {move || {
                (!empty).then(|| {
                    let (values, tags) = summary.get();
                    view! {
                        <SkeletonSummary>
                            <AssetSummaryBand
                                market=values.market
                                profit=values.profit
                                profit_rate=values.profit_rate
                                daily_change=values.daily_change
                                tags=tags
                                composition_link=true
                            />
                        </SkeletonSummary>
                        <SkeletonNote
                            notes=vec![stock_note.clone()]
                            filtered=filtered_count
                            basis=stock_basis_lines()
                        />
                    }
                })
            }}
            <SkeletonSearch>
                <AssetSearchBar
                    search=search
                    search_count=search_count
                    placeholder=labels::search_placeholder(SEARCH_TARGET)
                    conditions=&[ConditionKind::Account]
                    account_options=account_options
                    course_options=course_options
                />
            </SkeletonSearch>
            <div class="asset-list-surface">
                <div class="hidden lg:block print:block">
                    {move || {
                        let views = filtered_views.get();
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
                                price_date_rep=date_rep.clone()
                                dividends=dividends
                                sort_key=sort_key
                                sort_order=sort_order
                            />
                        }
                    }}
                </div>
                <div class="lg:hidden print:hidden">
                    {move || {
                        let views = filtered_views.get();
                        let key = sort_key.get();
                        let order = sorted_views(&views, &dividends.get(), key, sort_order.get());
                        let sorted: Vec<_> = order
                            .into_iter()
                            .map(|index| views[index].clone())
                            .collect();
                        let show_dividend = matches!(key, SortKey::Dividend | SortKey::Yield);
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
            {move || {
                (!empty && filtered_views.with(|views| views.is_empty())).then(|| {
                    view! {
                        <EmptyFilteredPanel
                            title=EMPTY_FILTERED_TITLE
                            on_clear=move || {
                                search.update(|s| *s = AssetSearch::default())
                            }
                        />
                    }
                })
            }}
        </section>
    }
    .into_any()
}

#[component]
fn FundContent(
    resolved: ResolvedFundBalance,
    search: RwSignal<AssetSearch>,
    search_count: RwSignal<(usize, usize)>,
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    let total_count = resolved.rows.len();
    let account_rows = resolved.rows.clone();
    let account_options = Memo::new(move |_| fund_accounts(&account_rows));
    let course_rows = resolved.rows.clone();
    let course_options = Memo::new(move |_| fund_courses(&course_rows));
    let rows_for_filter = resolved.rows.clone();
    let initial = filtered_fund_views(&rows_for_filter, &search.get_untracked());
    search_count.set((initial.len(), total_count));
    let filtered = RwSignal::new(initial);
    Effect::new(move |_| {
        let next = filtered_fund_views(&rows_for_filter, &search.get());
        search_count.set((next.len(), total_count));
        filtered.set(next);
    });
    // 集計は絞り込み前の全行で求め、検索中も全保有分を維持する
    let all_views: Vec<_> = resolved.rows.iter().map(fund_view).collect();
    let totals = fund_totals(&all_views);
    let fallback = resolved.fallback_totals.clone();
    let (market, purchase, profit, daily) = match &fallback {
        Some(summary) => (
            crate::features::asset_balance::dec_to_f64(&summary.total_market_value),
            crate::features::asset_balance::dec_to_f64(&summary.total_purchase_amount),
            crate::features::asset_balance::dec_to_f64(&summary.total_market_value)
                - crate::features::asset_balance::dec_to_f64(&summary.total_purchase_amount),
            (summary.latest_nav_count > 0)
                .then(|| crate::features::asset_balance::dec_to_f64(&summary.total_daily_change)),
        ),
        None => (
            totals.market,
            totals.purchase,
            totals.profit,
            totals.has_daily_change.then_some(totals.daily_change),
        ),
    };
    let profit_rate = (purchase != 0.0).then_some(profit / purchase * 100.0);
    let filtered_count = filtered_count(search, search_count);
    let empty = total_count == 0;
    let notes = fund_notes(&resolved, "");
    view! {
        {empty.then(|| view! { <EmptyAssetPanel /> }.into_any())}
        <section id="assetbalance-fund-list" hidden=empty>
            <SkeletonSummary>
                <AssetSummaryBand
                    market=Some(market)
                    profit=Some(profit)
                    profit_rate=profit_rate
                    daily_change=daily
                    tags=vec![purchase_summary_tag(purchase)]
                    composition_link=false
                />
            </SkeletonSummary>
            <SkeletonNote notes=notes filtered=filtered_count basis=fund_basis_lines() />
            <SkeletonSearch>
                <AssetSearchBar
                    search=search
                    search_count=search_count
                    placeholder=labels::search_placeholder(SEARCH_TARGET)
                    conditions=&[ConditionKind::Account, ConditionKind::Course]
                    account_options=account_options
                    course_options=course_options
                />
            </SkeletonSearch>
            <div class="asset-list-surface">
                <div class="hidden lg:block print:block">
                    {move || {
                        let views = filtered.get();
                        let order = sort_fund_views(&views, sort_key.get(), sort_order.get());
                        let sorted: Vec<_> = order
                            .into_iter()
                            .map(|index| views[index].clone())
                            .collect();
                        view! {
                            <FundHoldingsTable
                                views=sorted
                                sort_key=sort_key
                                sort_order=sort_order
                            />
                        }
                    }}
                </div>
                <div class="lg:hidden print:hidden">
                    {move || {
                        let views = filtered.get();
                        let order = sort_fund_views(&views, sort_key.get(), sort_order.get());
                        let sorted: Vec<_> = order
                            .into_iter()
                            .map(|index| views[index].clone())
                            .collect();
                        view! { <FundHoldingsList views=sorted /> }
                    }}
                </div>
            </div>
            {move || {
                (filtered.with(|views| views.is_empty()) && total_count > 0).then(|| {
                    view! {
                        <EmptyFilteredPanel
                            title=EMPTY_FILTERED_TITLE
                            on_clear=move || {
                                search.update(|s| *s = AssetSearch::default())
                            }
                        />
                    }
                })
            }}
        </section>
    }
    .into_any()
}

fn filtered_fund_views(rows: &[FundBalanceRow], search: &AssetSearch) -> Vec<FundView> {
    filter_fund_rows(rows, search)
        .into_iter()
        .map(fund_view)
        .collect()
}

#[component]
fn AllContent(
    stock_rows: Vec<AssetBalanceRow>,
    stock_fallback: Option<AssetBalanceSummary>,
    stock_note: String,
    fund_resolved: ResolvedFundBalance,
    search: RwSignal<AssetSearch>,
    search_count: RwSignal<(usize, usize)>,
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: Generation,
) -> impl IntoView {
    let stock_total = stock_rows.len();
    let fund_total = fund_resolved.rows.len();
    let total_count = stock_total + fund_total;
    let stock_rows_for_search = stock_rows.clone();
    let fund_rows_for_search = fund_resolved.rows.clone();
    // 株の見出しと同じ代表を行の日付出し分けに使う(絞り込み前の全行)。
    // 投信の行は日付を出さない(基準日は title にだけ入れる)
    let stock_date_rep = {
        let dates = stock_price_dates(&stock_rows_for_search);
        price_date_representative(dates.into_iter()).map(str::to_string)
    };
    let build_rows = move |search: &AssetSearch| -> Vec<UnifiedRow> {
        // 「すべて」はテキストのみ。口座・コースの選択条件はタブごとに語彙が
        // 違うためここでは使わない(テキストは株の口座・ファンド口座も部分一致する)
        let text_search = text_only_search(search);
        let views =
            filtered_portfolio_views(&stock_rows_for_search, &text_search, lookup, generation);
        let dividend_maps = dividends.get();
        let mut rows: Vec<UnifiedRow> = views
            .iter()
            .map(|view| unified_from_stock(view, &dividend_maps))
            .collect();
        for row in &mut rows {
            row.date_outlier =
                price_date_is_outlier(row.price_as_of.as_deref(), stock_date_rep.as_deref());
        }
        let fund_rows: Vec<UnifiedRow> = filter_fund_rows(&fund_rows_for_search, &text_search)
            .into_iter()
            .map(fund_view)
            .map(|view| unified_from_fund(&view))
            .collect();
        rows.extend(fund_rows);
        rows
    };
    let initial = build_rows(&search.get_untracked());
    search_count.set((initial.len(), total_count));
    let filtered = RwSignal::new(initial);
    Effect::new(move |_| {
        let next = build_rows(&search.get());
        search_count.set((next.len(), total_count));
        filtered.set(next);
    });
    // 集計は両種別を合算。株側は銘柄ごとの現在値評価を使う
    let stock_all_views = all_portfolio_views(&stock_rows, lookup, generation);
    let stock_inputs: Vec<ValuationInput> =
        stock_all_views.iter().map(ValuationInput::from).collect();
    let stock_valuation = crate::features::asset_balance::model::portfolio_valuation(&stock_inputs);
    let stock_rows_purchase: f64 = stock_inputs.iter().map(|i| i.total_purchase_amount).sum();
    let stock_market = stock_fallback
        .as_ref()
        .map(|summary| crate::features::asset_balance::dec_to_f64(&summary.total_market_value))
        .unwrap_or_else(|| stock_valuation.market_value.unwrap_or(0.0));
    let stock_purchase = stock_fallback
        .as_ref()
        .map(|summary| crate::features::asset_balance::dec_to_f64(&summary.total_purchase_amount))
        .unwrap_or(stock_rows_purchase);
    let stock_daily = stock_fallback
        .as_ref()
        .map(|summary| crate::features::asset_balance::dec_to_f64(&summary.total_daily_change))
        .or_else(|| stock_daily_change_total(&stock_all_views));
    let fund_rows = fund_resolved.rows.clone();
    let fund_views: Vec<_> = fund_rows.iter().map(fund_view).collect();
    let fund_totals_value = fund_totals(&fund_views);
    let (fund_market, fund_purchase, fund_profit, fund_daily) = match &fund_resolved.fallback_totals
    {
        Some(summary) => (
            crate::features::asset_balance::dec_to_f64(&summary.total_market_value),
            crate::features::asset_balance::dec_to_f64(&summary.total_purchase_amount),
            crate::features::asset_balance::dec_to_f64(&summary.total_market_value)
                - crate::features::asset_balance::dec_to_f64(&summary.total_purchase_amount),
            (summary.latest_nav_count > 0)
                .then(|| crate::features::asset_balance::dec_to_f64(&summary.total_daily_change)),
        ),
        None => (
            fund_totals_value.market,
            fund_totals_value.purchase,
            fund_totals_value.profit,
            fund_totals_value
                .has_daily_change
                .then_some(fund_totals_value.daily_change),
        ),
    };
    let mut notes = vec![stock_note];
    notes.extend(fund_notes(&fund_resolved, "投資信託の"));
    let stock_profit = stock_market - stock_purchase;
    let market = stock_market + fund_market;
    let purchase = stock_purchase + fund_purchase;
    let profit = stock_profit + fund_profit;
    let daily = match (stock_daily, fund_daily) {
        (Some(a), Some(b)) => Some(a + b),
        (a, b) => a.or(b),
    };
    let profit_rate = (purchase != 0.0).then_some(profit / purchase * 100.0);
    let empty = total_count == 0;
    let filtered_count = filtered_count(search, search_count);
    let account_options = Memo::new(|_| Vec::<String>::new());
    let course_options = Memo::new(|_| Vec::<String>::new());
    view! {
        {empty.then(|| view! { <EmptyAssetPanel /> }.into_any())}
        <section id="assetbalance-all-list" hidden=empty>
            <SkeletonSummary>
                <AssetSummaryBand
                    market=Some(market)
                    profit=Some(profit)
                    profit_rate=profit_rate
                    daily_change=daily
                    tags=vec![purchase_summary_tag(purchase)]
                    composition_link=false
                />
            </SkeletonSummary>
            <SkeletonNote notes=notes filtered=filtered_count basis=all_basis_lines() />
            <SkeletonSearch>
                <AssetSearchBar
                    search=search
                    search_count=search_count
                    placeholder=labels::search_placeholder(SEARCH_TARGET)
                    conditions=&[]
                    account_options=account_options
                    course_options=course_options
                />
            </SkeletonSearch>
            <div class="asset-list-surface">
                <div class="hidden lg:block print:block">
                    {move || {
                        let rows = filtered.get();
                        view! {
                            <AssetMergedTable
                                rows=rows
                                sort_key=sort_key
                                sort_order=sort_order
                            />
                        }
                    }}
                </div>
                <div class="lg:hidden print:hidden">
                    {move || {
                        let rows = filtered.get();
                        let order = sort_unified(&rows, sort_key.get(), sort_order.get());
                        let sorted: Vec<_> = order
                            .into_iter()
                            .map(|index| rows[index].clone())
                            .collect();
                        view! { <AssetMergedList rows=sorted /> }
                    }}
                </div>
            </div>
            {move || {
                (filtered.with(|rows| rows.is_empty()) && total_count > 0).then(|| {
                    view! {
                        <EmptyFilteredPanel
                            title=EMPTY_FILTERED_TITLE
                            on_clear=move || {
                                search.update(|s| *s = AssetSearch::default())
                            }
                        />
                    }
                })
            }}
        </section>
    }
    .into_any()
}

#[component]
pub(crate) fn AssetBalanceMainContent(
    stock_rows: Vec<AssetBalanceRow>,
    fallback_totals: Option<AssetBalanceSummary>,
    stock_note: String,
    fund_resolved: ResolvedFundBalance,
    search: RwSignal<AssetSearch>,
    search_count: RwSignal<(usize, usize)>,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
    fund_sort_key: RwSignal<FundSortKey>,
    fund_sort_order: RwSignal<SortOrder>,
    kind: RwSignal<AssetKind>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    generation: Generation,
) -> impl IntoView {
    // 口座・コースの選択肢はタブごとに語彙が違うため、切替時は選択条件だけ消す
    // (テキストは全タブ共通で引き継ぐ)
    let prev_kind = RwSignal::new(kind.get_untracked());
    Effect::new(move |_| {
        let next = kind.get();
        if prev_kind.get_untracked() == next {
            return;
        }
        prev_kind.set(next);
        search.update(|s| *s = text_only_search(s));
    });
    view! {
        {move || match kind.get() {
            AssetKind::Stock => {
                view! {
                    <StockContent
                        rows=stock_rows.clone()
                        fallback_totals=fallback_totals.clone()
                        stock_note=stock_note.clone()
                        search=search
                        search_count=search_count
                        sort_key=sort_key
                        sort_order=sort_order
                        dividends=dividends
                        lookup=lookup
                        generation=generation
                    />
                }
                    .into_any()
            }
            AssetKind::Fund => {
                view! {
                    <FundContent
                        resolved=fund_resolved.clone()
                        search=search
                        search_count=search_count
                        sort_key=fund_sort_key
                        sort_order=fund_sort_order
                    />
                }
                    .into_any()
            }
            AssetKind::All => {
                view! {
                    <AllContent
                        stock_rows=stock_rows.clone()
                        stock_fallback=fallback_totals.clone()
                        stock_note=stock_note.clone()
                        fund_resolved=fund_resolved.clone()
                        search=search
                        search_count=search_count
                        sort_key=fund_sort_key
                        sort_order=fund_sort_order
                        dividends=dividends
                        lookup=lookup
                        generation=generation
                    />
                }
                    .into_any()
            }
        }}
    }
    .into_any()
}
