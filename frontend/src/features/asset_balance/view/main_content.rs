use super::all_list::{
    AssetMergedList, AssetMergedTable, UnifiedRow, sort_unified, unified_from_fund,
    unified_from_stock,
};
use super::asset_table::AssetHoldingsTable;
use super::fund_table::{FundHoldingsList, FundHoldingsTable};
use super::holdings_list::HoldingsList;
use super::search_bar::{AssetMobileSearchEntry, AssetSearchBar, ConditionKind};
use super::summary::{
    AssetSummaryBand, fund_has_pending, oldest_combined_as_of, oldest_fund_as_of,
    oldest_stock_as_of, purchase_summary_tag, stock_pending_from_parts, stock_price_dates,
    stock_summary_values,
};
use crate::api::dto::{AssetBalanceSummary, FundBalanceSummary};
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
use crate::features::asset_balance::valuation_has_pending;
use crate::features::dividend_per_share::DividendMaps;
use crate::session::Generation;
use crate::ui::button::{Button, ButtonVariant};
use crate::ui::empty_state::EmptyState;
use crate::ui::labels;
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
            filtering: !search.with(super::super::fund::AssetSearch::is_empty),
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

/// 並べ替え結果の添字を行データに適用する。PC/モバイルの両表示で同じ順序を使う
fn apply_order<T: Clone>(views: &[T], order: Vec<usize>) -> Vec<T> {
    order
        .into_iter()
        .map(|index| views[index].clone())
        .collect()
}

/// 株式行を並べ替えた結果。PC表とモバイル一覧で共有する
fn sorted_stock_views(
    views: &[HoldingView],
    dividends: &DividendMaps,
    key: SortKey,
    order: SortOrder,
) -> Vec<HoldingView> {
    apply_order(views, sorted_views(views, dividends, key, order))
}

/// 投信行を並べ替えた結果。PC表とモバイル一覧で共有する
fn sorted_fund_views(views: &[FundView], key: FundSortKey, order: SortOrder) -> Vec<FundView> {
    apply_order(views, sort_fund_views(views, key, order))
}

/// 損益率。取得総額が0のときは出さない
fn profit_rate(profit: f64, purchase: f64) -> Option<f64> {
    (purchase != 0.0).then_some(profit / purchase * 100.0)
}

/// 株と投信の前日比を合算する。片方だけあるときはある方を使う
fn combined_daily(stock: Option<f64>, fund: Option<f64>) -> Option<f64> {
    match (stock, fund) {
        (Some(stock), Some(fund)) => Some(stock + fund),
        (stock, fund) => stock.or(fund),
    }
}

/// 投信の集計値。API集計があるときはそれを使い、無ければ行から求める
fn fund_summary(
    views: &[FundView],
    fallback: Option<&FundBalanceSummary>,
) -> (f64, f64, f64, Option<f64>) {
    let totals = fund_totals(views);
    fallback.map_or_else(
        || {
            (
                totals.market,
                totals.purchase,
                totals.profit,
                totals.has_daily_change.then_some(totals.daily_change),
            )
        },
        |summary| {
            (
                crate::features::asset_balance::dec_to_f64(&summary.total_market_value),
                crate::features::asset_balance::dec_to_f64(&summary.total_purchase_amount),
                crate::features::asset_balance::dec_to_f64(&summary.total_market_value)
                    - crate::features::asset_balance::dec_to_f64(&summary.total_purchase_amount),
                (summary.latest_nav_count > 0).then(|| {
                    crate::features::asset_balance::dec_to_f64(&summary.total_daily_change)
                }),
            )
        },
    )
}

#[component]
fn EmptyAssetPanel() -> impl IntoView {
    view! {
        <div class="px-4 py-4 lg:px-0">
            <EmptyState
                title=labels::ASSET_EMPTY_TITLE
                description=labels::ASSET_EMPTY_DESCRIPTION
            >
                <a
                    href="/csv"
                    class="mt-4 inline-flex items-center justify-center rounded-md border border-border-strong bg-surface px-4 py-2 text-sm font-bold text-text no-underline hover:border-border-bold hover:bg-surface-sunken"
                >
                    {labels::CSV_IMPORT}
                </a>
            </EmptyState>
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
                        {labels::CLEAR_SEARCH_CONDITIONS}
                    </Button>
                </div>
            </div>
        </div>
    }
}

/// 検索で全行が消えたときの文面。3タブで同じ文言を使う
const EMPTY_FILTERED_TITLE: &str = labels::NO_MATCH_TITLE;

/// 検索で全行が消えたときの表示。クリア操作は3タブで同じ
#[component]
fn EmptyFilteredSearch(search: RwSignal<AssetSearch>) -> impl IntoView {
    view! {
        <EmptyFilteredPanel
            title=EMPTY_FILTERED_TITLE
            on_clear=move || {
                search.update(|search| *search = AssetSearch::default());
            }
        />
    }
}

#[component]
fn StockContent(
    rows: Vec<AssetBalanceRow>,
    fallback_totals: Option<AssetBalanceSummary>,
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
        search_count.set((filtered_views.with(std::vec::Vec::len), total_count));
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
    let summary_fallback = fallback_totals.clone();
    let summary = Memo::new(move |_| {
        let (values, tags) = all_views
            .with(|views| stock_summary_values(views, summary_fallback.as_ref(), &dividends.get()));
        let purchase = summary_fallback
            .as_ref()
            .map(|summary| {
                crate::features::asset_balance::dec_to_f64(&summary.total_purchase_amount)
            })
            .or_else(|| {
                all_views.with(|views| {
                    let total: f64 = views.iter().map(|view| view.purchase).sum();
                    Some(total)
                })
            });
        (values, tags, purchase)
    });
    view! {
        {empty.then(|| view! { <EmptyAssetPanel /> }.into_any())}
        <section id="assetbalance-stock-list" hidden=empty>
            {move || {
                (!empty).then(|| {
                    let (values, tags, purchase) = summary.get();
                    let as_of = oldest_stock_as_of(&rows).unwrap_or_default();
                    let pending = valuation_has_pending(
                        stock_pending_from_parts(&rows, fallback_totals.as_ref()),
                        false,
                    );
                    view! {
                        <SkeletonSummary>
                            <AssetSummaryBand
                                market=values.market
                                profit=values.profit
                                profit_rate=values.profit_rate
                                daily_change=values.daily_change
                                tags=tags
                                purchase=purchase
                                as_of=as_of
                                show_pending=pending
                            />
                        </SkeletonSummary>
                        <SkeletonNote filtered=filtered_count />
                    }
                })
            }}
            <SkeletonSearch>
                <AssetSearchBar
                    search=search
                    search_count=search_count
                    placeholder=labels::search_placeholder(labels::ASSET_SEARCH_TARGET)
                    conditions=&[ConditionKind::Account]
                    account_options=account_options
                    course_options=course_options
                />
                <AssetMobileSearchEntry
                    search=search
                    search_count=search_count
                    placeholder=labels::search_placeholder(labels::ASSET_SEARCH_TARGET)
                    conditions=&[ConditionKind::Account]
                    account_options=account_options
                    course_options=course_options
                    sort_options=&SortKey::ALL
                    key_label=|key: SortKey| key.label()
                    key_value=|key: SortKey| key.value()
                    key_from_value=|value: &str| SortKey::from_value(value)
                    sort_key=sort_key
                    sort_order=sort_order
                    composition=true
                />
            </SkeletonSearch>
            <div class="asset-list-surface">
                <div class="hidden lg:block print:block">
                    {move || {
                        let views = filtered_views.get();
                        let sorted = sorted_stock_views(
                            &views,
                            &dividends.get(),
                            sort_key.get(),
                            sort_order.get(),
                        );
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
                        let sorted = sorted_stock_views(
                            &views,
                            &dividends.get(),
                            sort_key.get(),
                            sort_order.get(),
                        );
                        view! { <HoldingsList views=sorted dividends=dividends /> }
                    }}
                </div>
            </div>
            {move || {
                (!empty && filtered_views.with(std::vec::Vec::is_empty)).then(|| {
                    view! { <EmptyFilteredSearch search=search /> }
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
    let (market, purchase, profit, daily) =
        fund_summary(&all_views, resolved.fallback_totals.as_ref());
    let profit_rate = profit_rate(profit, purchase);
    let filtered_count = filtered_count(search, search_count);
    let empty = total_count == 0;
    let as_of = oldest_fund_as_of(&resolved).unwrap_or_default();
    let pending = valuation_has_pending(false, fund_has_pending(&resolved));
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
                    purchase=Some(purchase)
                    as_of=as_of
                    show_pending=pending
                />
            </SkeletonSummary>
            <SkeletonNote filtered=filtered_count />
            <SkeletonSearch>
                <AssetSearchBar
                    search=search
                    search_count=search_count
                    placeholder=labels::search_placeholder(labels::ASSET_SEARCH_TARGET)
                    conditions=&[ConditionKind::Account, ConditionKind::Course]
                    account_options=account_options
                    course_options=course_options
                />
                <AssetMobileSearchEntry
                    search=search
                    search_count=search_count
                    placeholder=labels::search_placeholder(labels::ASSET_SEARCH_TARGET)
                    conditions=&[ConditionKind::Account, ConditionKind::Course]
                    account_options=account_options
                    course_options=course_options
                    sort_options=&FundSortKey::ALL
                    key_label=|key: FundSortKey| key.label()
                    key_value=|key: FundSortKey| key.value()
                    key_from_value=|value: &str| FundSortKey::from_value(value)
                    sort_key=sort_key
                    sort_order=sort_order
                    composition=true
                />
            </SkeletonSearch>
            <div class="asset-list-surface">
                <div class="hidden lg:block print:block">
                    {move || {
                        let views = filtered.get();
                        let sorted = sorted_fund_views(&views, sort_key.get(), sort_order.get());
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
                        let sorted = sorted_fund_views(&views, sort_key.get(), sort_order.get());
                        view! { <FundHoldingsList views=sorted /> }
                    }}
                </div>
            </div>
            {move || {
                (filtered.with(std::vec::Vec::is_empty) && total_count > 0).then(|| {
                    view! { <EmptyFilteredSearch search=search /> }
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
    let stock_market = stock_fallback.as_ref().map_or_else(
        || stock_valuation.market_value.unwrap_or(0.0),
        |summary| crate::features::asset_balance::dec_to_f64(&summary.total_market_value),
    );
    let stock_purchase = stock_fallback
        .as_ref()
        .map_or(stock_rows_purchase, |summary| {
            crate::features::asset_balance::dec_to_f64(&summary.total_purchase_amount)
        });
    let stock_daily = stock_fallback
        .as_ref()
        .map(|summary| crate::features::asset_balance::dec_to_f64(&summary.total_daily_change))
        .or_else(|| stock_daily_change_total(&stock_all_views));
    let fund_rows = fund_resolved.rows.clone();
    let fund_views: Vec<_> = fund_rows.iter().map(fund_view).collect();
    let (fund_market, fund_purchase, fund_profit, fund_daily) =
        fund_summary(&fund_views, fund_resolved.fallback_totals.as_ref());
    let pending = valuation_has_pending(
        stock_pending_from_parts(&stock_rows, stock_fallback.as_ref()),
        fund_has_pending(&fund_resolved),
    );
    let as_of = oldest_combined_as_of(&stock_rows, &fund_resolved).unwrap_or_default();
    let stock_profit = stock_market - stock_purchase;
    let market = stock_market + fund_market;
    let purchase = stock_purchase + fund_purchase;
    let profit = stock_profit + fund_profit;
    let daily = combined_daily(stock_daily, fund_daily);
    let profit_rate = profit_rate(profit, purchase);
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
                    purchase=Some(purchase)
                    as_of=as_of
                    show_pending=pending
                />
            </SkeletonSummary>
            <SkeletonNote filtered=filtered_count />
            <SkeletonSearch>
                <AssetSearchBar
                    search=search
                    search_count=search_count
                    placeholder=labels::search_placeholder(labels::ASSET_SEARCH_TARGET)
                    conditions=&[]
                    account_options=account_options
                    course_options=course_options
                />
                <AssetMobileSearchEntry
                    search=search
                    search_count=search_count
                    placeholder=labels::search_placeholder(labels::ASSET_SEARCH_TARGET)
                    conditions=&[]
                    account_options=account_options
                    course_options=course_options
                    sort_options=&FundSortKey::MERGED
                    key_label=|key: FundSortKey| key.label()
                    key_value=|key: FundSortKey| key.value()
                    key_from_value=|value: &str| FundSortKey::from_value(value)
                    sort_key=sort_key
                    sort_order=sort_order
                    composition=true
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
                        let sorted = apply_order(&rows, order);
                        view! { <AssetMergedList rows=sorted /> }
                    }}
                </div>
            </div>
            {move || {
                (filtered.with(std::vec::Vec::is_empty) && total_count > 0).then(|| {
                    view! { <EmptyFilteredSearch search=search /> }
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
    view! {
        {move || match kind.get() {
            AssetKind::Stock => {
                view! {
                    <StockContent
                        rows=stock_rows.clone()
                        fallback_totals=fallback_totals.clone()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn fund_row(market: &str, profit: &str, day_change: Option<i64>) -> FundBalanceRow {
        serde_json::from_value(serde_json::json!({
            "id": format!("fund-{market}-{profit}"),
            "fund_name": format!("ファンド{market}"),
            "dividends_course": null,
            "account": "特定",
            "shares": 10000,
            "average_purchase_price": 10000,
            "total_purchase_amount": 1000,
            "base_price": 11000,
            "base_price_day_change": day_change,
            "base_price_month_change": null,
            "market_value": market.parse::<rust_decimal::Decimal>().unwrap(),
            "profit_loss": profit.parse::<rust_decimal::Decimal>().unwrap(),
            "profit_loss_rate": null,
            "total_return": null,
            "price_as_of": "2026-10-10",
            "price_source": "csv",
            "nav_is_latest": true,
            "expected_nav_date": null,
            "created_at": "2026-10-17T00:00:00Z",
            "updated_at": "2026-10-18T00:00:00Z"
        }))
        .unwrap()
    }

    #[test]
    fn apply_order_maps_sorted_index_to_rows() {
        let rows = vec!["a", "b", "c"];
        assert_eq!(apply_order(&rows, vec![2, 0]), vec!["c", "a"]);
        let empty: Vec<String> = Vec::new();
        assert!(apply_order(&empty, Vec::new()).is_empty());
    }

    #[test]
    fn profit_rate_returns_none_when_purchase_is_zero() {
        assert_eq!(profit_rate(100.0, 1000.0), Some(10.0));
        assert_eq!(profit_rate(-50.0, 1000.0), Some(-5.0));
        assert_eq!(profit_rate(100.0, 0.0), None);
    }

    #[test]
    fn combined_daily_prefers_sum_then_either_side() {
        assert_eq!(combined_daily(Some(10.0), Some(20.0)), Some(30.0));
        assert_eq!(combined_daily(Some(10.0), None), Some(10.0));
        assert_eq!(combined_daily(None, Some(20.0)), Some(20.0));
        assert_eq!(combined_daily(None, None), None);
    }

    #[test]
    fn fund_summary_uses_rows_when_no_fallback() {
        let rows = [fund_row("100", "10", None), fund_row("300", "-5", None)];
        let views: Vec<_> = rows.iter().map(fund_view).collect();
        let (market, purchase, profit, daily) = fund_summary(&views, None);
        assert_eq!(market, 400.0);
        assert_eq!(purchase, 2000.0);
        assert_eq!(profit, 5.0);
        assert_eq!(daily, None);
    }

    #[test]
    fn sorted_fund_views_keeps_table_and_list_order_same() {
        let rows = [fund_row("100", "10", None), fund_row("300", "-5", None)];
        let views: Vec<_> = rows.iter().map(fund_view).collect();
        let sorted = sorted_fund_views(&views, FundSortKey::MarketValue, SortOrder::Desc);
        assert_eq!(sorted[0].market_value, 300.0);
        assert_eq!(sorted[1].market_value, 100.0);
        let ascending = sorted_fund_views(&views, FundSortKey::MarketValue, SortOrder::Asc);
        assert_eq!(ascending[0].market_value, 100.0);
    }
}
