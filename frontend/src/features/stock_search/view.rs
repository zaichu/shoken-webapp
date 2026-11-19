use crate::api::dto::Stock;
use crate::features::asset_balance;
use crate::features::asset_balance::{HoldingGroup, HoldingView, ValuationInput};
use crate::features::dividend_per_share::DividendMaps;
use crate::session;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::AccountBadge;
use crate::ui::format;
use crate::ui::holding_tiles::{
    HoldingDetailItem, HoldingDetails, HoldingSummary, HoldingSummaryItem,
};
use crate::ui::labels;
use crate::ui::page_frame::PageFrame;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use crate::ui::stock_search;
use leptos::prelude::*;
use shared::normalize;

const STOCK_LINKS: &[(&str, &str)] = &[
    (
        labels::STOCK_LINK_RAKUTEN,
        "https://www.rakuten-sec.co.jp/web/market/search/quote.html?ric={code}.T",
    ),
    (
        labels::STOCK_LINK_KABUTAN,
        "https://kabutan.jp/stock/?code={code}",
    ),
    (
        labels::STOCK_LINK_YAHOO,
        "https://finance.yahoo.co.jp/quote/{code}",
    ),
    (
        labels::STOCK_LINK_SBI,
        "https://site3.sbisec.co.jp/ETGate/?_ControlID=WPLETsiR001Control&_DataStoreID=DSWPLETsiR001Control&_PageID=WPLETsiR001Ilst10&_ActionID=getDetailOfStockPriceJP&s_rkbn=1&i_stock_sec=%94%43%93%56%93%B0&i_dom_flg=1&i_exchange_code=JPN&i_output_type=0&stock_sec_code_mul={code}",
    ),
    (
        labels::STOCK_LINK_NIKKEI,
        "https://www.nikkei.com/nkd/company/?scode={code}",
    ),
    (
        labels::STOCK_LINK_BUFFETT,
        "https://www.buffett-code.com/company/{code}",
    ),
    (
        labels::STOCK_LINK_MINKABU,
        "https://minkabu.jp/stock/{code}/",
    ),
    (labels::STOCK_LINK_IR_BANK, "https://irbank.net/{code}"),
    (
        labels::STOCK_LINK_SCOUTER,
        "https://monex.ifis.co.jp/index.php?sa=report_index&bcode={code}",
    ),
    (
        labels::STOCK_LINK_ZAIMANI,
        "https://zaimani.com/search/?_sf_s={code}",
    ),
    (
        labels::STOCK_LINK_JPX,
        "https://jpx-explorer.com/ja-JP/{code}-TSE",
    ),
];

#[component]
pub(crate) fn StockSearchPage() -> impl IntoView {
    let stock_search = super::use_stock_search();
    let session = session::use_session();
    let balance_state = asset_balance::use_asset_balance_state(session);
    let balances = balance_state.balances();
    let dividends = balance_state.dividends();
    let lookup = balance_state.lookup();
    let data_ops = balance_state.data_ops();
    if session.user.get_untracked().is_some() {
        asset_balance::load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            data_ops,
        );
    }
    let loading = stock_search.search.pending();
    let has_invalid = stock_search.has_invalid_code_param;

    // 検索した銘柄コードを URL に載せ、更新・戻る・リンクで同じ画面に戻れるようにする。
    // URL 遷移できたときは再マウント時の読み直しに任せ、できなかったときだけ直接取り直す
    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if loading.get_untracked() || stock_search.stock_code.get_untracked().is_empty() {
            return;
        }
        let code = stock_search.stock_code.get_untracked();
        if !super::sync_code_param(&code) {
            stock_search.search.dispatch(code);
        }
    };

    // 持っている銘柄だけ、結果カードの上に保有情報を出す。
    // 銘柄マスタに無い保有銘柄も URL のコードから解決し、検索結果と独立して出す。
    // ただし別名の検索失敗に前の銘柄を流用しないよう、フォールバックは
    // 入力がそのコードのとき(コード検索・URL直開き)だけ使う
    let holding_group = Memo::new(move |_| {
        session.user.get()?;
        let code = stock_search
            .stock_data()
            .map(|stock| stock.code)
            .filter(|code| !code.is_empty())
            .unwrap_or_else(|| {
                let url = super::url_stock_code();
                if !url.is_empty() && stock_search.stock_code.get() == url {
                    url
                } else {
                    String::new()
                }
            });
        if code.is_empty() {
            return None;
        }
        let generation = session.generation.get();
        let rows = balances.with(|slot| {
            asset_balance::resolve_asset_balance(generation, slot).map(|resolved| resolved.rows)
        })?;
        let views = asset_balance::all_portfolio_views(&rows, lookup, generation);
        asset_balance::group_holdings_by_code(&views)
            .into_iter()
            .find(|group| group.code == code)
    });
    let has_holding = Memo::new(move |_| holding_group.get().is_some());

    // 保有一覧の取得失敗は非保有と区別し、再試行できるようにする
    let balance_error = Memo::new(move |_| {
        session.user.get()?;
        let generation = session.generation.get();
        balances.with(|slot| match slot {
            Some((cached, Err(message))) if *cached == generation => Some(message.clone()),
            _ => None,
        })
    });
    let refresh_error = Memo::new(move |_| data_ops.with(|ops| ops.refresh_error.clone()));
    let reload_balances = move || {
        if session.user.get_untracked().is_none() {
            return;
        }
        balances.set(None);
        asset_balance::load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            data_ops,
        );
    };

    view! {
        {move || {
            // 持っているときだけ画面の上部に資産管理への戻りも出す
            if has_holding.get() {
                view! {
                    <PageFrame
                        title=labels::STOCK_SEARCH_TITLE
                        action_bar=view! {
                            <stock_search::SearchBar
                                stock_code=stock_search.stock_code
                                loading=loading.into()
                                mobile=true
                                on_submit=on_submit
                            />
                        }
                            .into_any()
                        action_bar_mobile_only=true
                        back_href="/assetbalance"
                        back_label=labels::ASSET_BALANCE_BACK
                    >
                        <SearchPageBody
                            stock_search=stock_search
                            loading=loading
                            on_submit=on_submit
                            has_invalid=has_invalid
                            holding_group=holding_group
                            dividends=dividends
                            balance_error=balance_error
                            refresh_error=refresh_error
                            on_retry_balances=reload_balances
                        />
                    </PageFrame>
                }
                    .into_any()
            } else {
                view! {
                    <PageFrame
                        title=labels::STOCK_SEARCH_TITLE
                        action_bar=view! {
                            <stock_search::SearchBar
                                stock_code=stock_search.stock_code
                                loading=loading.into()
                                mobile=true
                                on_submit=on_submit
                            />
                        }
                            .into_any()
                        action_bar_mobile_only=true
                    >
                        <SearchPageBody
                            stock_search=stock_search
                            loading=loading
                            on_submit=on_submit
                            has_invalid=has_invalid
                            holding_group=holding_group
                            dividends=dividends
                            balance_error=balance_error
                            refresh_error=refresh_error
                            on_retry_balances=reload_balances
                        />
                    </PageFrame>
                }
                    .into_any()
            }
        }}
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SearchBodyState {
    SearchFailed,
    BalanceFailed,
    Found,
    NotFound,
    Initial,
    Loading,
}

const fn resolve_search_body_state(
    has_search_error: bool,
    has_balance_error: bool,
    has_data: bool,
    is_not_found: bool,
    is_loading: bool,
) -> SearchBodyState {
    if has_search_error {
        SearchBodyState::SearchFailed
    } else if has_balance_error {
        SearchBodyState::BalanceFailed
    } else if has_data {
        SearchBodyState::Found
    } else if is_not_found {
        SearchBodyState::NotFound
    } else if !is_loading {
        SearchBodyState::Initial
    } else {
        SearchBodyState::Loading
    }
}
/// 検索欄(PC の本文直上)・結果・保有情報をまとめた本文。戻りの有無で枠を
/// 作り分けても中身は同じものを使う
#[component]
fn SearchPageBody(
    stock_search: super::StockSearch,
    loading: Memo<bool>,
    on_submit: impl Fn(web_sys::SubmitEvent) + Send + Sync + Copy + 'static,
    has_invalid: bool,
    holding_group: Memo<Option<HoldingGroup>>,
    dividends: RwSignal<DividendMaps>,
    balance_error: Memo<Option<String>>,
    refresh_error: Memo<Option<String>>,
    on_retry_balances: impl Fn() + Send + Sync + Copy + 'static,
) -> impl IntoView {
    // 名前検索の成功も銘柄コードで URL に載せ、更新・共有で同じ銘柄に戻れるようにする。
    // コード検索の送信と同じく履歴に積んで再マウントするため、戻るで前の銘柄に戻れる。
    // 載せ替え後は URL とコードが一致して何もしないため循環も起きない
    Effect::new(move |_| {
        let Some(stock) = stock_search.stock_data() else {
            return;
        };
        let code = stock.code.clone();
        if !crate::support::list_search::is_searchable_code(&code)
            || super::url_stock_code() == code
        {
            return;
        }
        super::sync_code_param(&code);
    });
    view! {
        <stock_search::SearchBar stock_code=stock_search.stock_code loading=loading.into() on_submit=on_submit />
        <Show when=move || has_invalid>
            <Alert variant=AlertVariant::Warning>{labels::STOCK_SEARCH_INVALID}</Alert>
        </Show>
        {move || {
            refresh_error
                .get()
                .map(|message| {
                    view! {
                        <stock_search::RefreshError message=message />
                    }
                })
        }}
        {move || {
            let search_error = stock_search.error_message();
            let balance_message = balance_error.get();
            let data = stock_search.stock_data();
            let state = resolve_search_body_state(
                search_error.is_some(),
                balance_message.is_some(),
                data.is_some(),
                stock_search.is_not_found(),
                loading.get(),
            );
            match state {
                SearchBodyState::SearchFailed => {
                    view! {
                        <SearchFailedBody
                            message=search_error.clone().unwrap_or_default()
                            stock_search=stock_search
                        />
                    }
                        .into_any()
                }
                SearchBodyState::BalanceFailed => {
                    view! {
                        <BalanceFailedBody
                            stock=data.clone()
                            message=balance_message.clone().unwrap_or_default()
                            on_retry_balances=on_retry_balances
                        />
                    }
                        .into_any()
                }
                SearchBodyState::Found => {
                    data.clone()
                        .map_or_else(
                            || {
                                view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
                                    .into_any()
                            },
                            |stock| {
                                view! {
                                    <FoundBody stock=stock holding=holding_group.get() dividends=dividends />
                                }
                                    .into_any()
                            },
                        )
                }
                SearchBodyState::NotFound => {
                    view! {
                        <NotFoundBody holding=holding_group.get() dividends=dividends />
                    }
                        .into_any()
                }
                SearchBodyState::Initial => {
                    view! { <EmptySearch title=labels::STOCK_SEARCH_INITIAL_TITLE /> }.into_any()
                }
                SearchBodyState::Loading => {
                    view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }.into_any()
                }
            }
        }}
    }
}

#[component]
fn SearchFailedBody(message: String, stock_search: super::StockSearch) -> impl IntoView {
    view! {
        <ListLoadError
            message=message
            on_retry=move || {
                let code = stock_search.stock_code.get_untracked();
                if !code.is_empty() && !stock_search.search.pending().get_untracked() {
                    stock_search.search.dispatch(code);
                }
            }
        />
    }
}

#[component]
fn BalanceFailedBody(
    stock: Option<Stock>,
    message: String,
    on_retry_balances: impl Fn() + Send + Sync + Copy + 'static,
) -> impl IntoView {
    view! {
        <stock_search::BalanceError message=message on_retry=on_retry_balances />
        {stock.map(|stock| view! { <StockInfo stock=stock /> })}
    }
}

#[component]
fn FoundBody(
    stock: Stock,
    holding: Option<HoldingGroup>,
    dividends: RwSignal<DividendMaps>,
) -> impl IntoView {
    view! {
        <StockInfo stock=stock />
        {holding.map(|group| view! { <HoldingSection group=group dividends=dividends /> })}
    }
}

#[component]
fn NotFoundBody(holding: Option<HoldingGroup>, dividends: RwSignal<DividendMaps>) -> impl IntoView {
    view! {
        {holding.map(|group| view! { <HoldingSection group=group dividends=dividends /> })}
        <EmptySearch title=labels::STOCK_SEARCH_NOT_FOUND_TITLE />
    }
}

fn empty_search_description(title: &str) -> &'static str {
    if title == labels::STOCK_SEARCH_INITIAL_TITLE {
        labels::STOCK_SEARCH_INITIAL_HINT
    } else {
        labels::STOCK_SEARCH_NO_RESULT_HINT
    }
}

#[component]
fn EmptySearch(title: &'static str) -> impl IntoView {
    let description = empty_search_description(title);
    view! {
        <stock_search::EmptySearch title=title description=description />
    }
}

#[component]
fn StockInfo(stock: Stock) -> impl IntoView {
    let meta = [
        (
            labels::STOCK_SEARCH_META_MARKET,
            stock.market_category.clone(),
        ),
        (
            labels::STOCK_SEARCH_META_INDUSTRY_33,
            stock.industry_category_33.clone().unwrap_or_default(),
        ),
        (
            labels::STOCK_SEARCH_META_INDUSTRY_17,
            stock.industry_category_17.clone().unwrap_or_default(),
        ),
        (
            labels::STOCK_SEARCH_META_SIZE,
            stock.size_category.clone().unwrap_or_default(),
        ),
    ];
    let name = normalize::normalize_display_name(&stock.name);
    let links = self::STOCK_LINKS
        .iter()
        .map(|(name, template)| {
            (
                *name,
                template.replace(&format!("{{{}}}", "code"), stock.code.as_str()),
            )
        })
        .collect();
    view! {
        <stock_search::StockInfoCard name=name code=stock.code meta=meta links=links />
    }
}

/// 結果カードの上に出す保有情報。口座が複数ある銘柄は合計の下に口座ごとを並べる。
/// 取得単価は口座ごとにだけ出す
#[component]
fn HoldingSection(group: HoldingGroup, dividends: RwSignal<DividendMaps>) -> impl IntoView {
    let multi = group.rows.len() > 1;
    let badges: Vec<Option<String>> = group.rows.iter().map(|row| row.account.clone()).collect();
    view! {
        <stock_search::HoldingSection>
            {move || {
                // 配当は保有一覧より遅れて届くため、ここで読み直してタイルを作り直す
                let maps = dividends.get();
                let total = HoldingTilesData::total(&group, &maps);
                if multi {
                    let accounts = group
                        .rows
                        .iter()
                        .map(|row| HoldingTilesData::account(row, &group, &maps))
                        .zip(badges.clone())
                        .map(|(data, badge)| {
                            view! {
                                <stock_search::HoldingAccount account=badge>
                                    <HoldingTilesView data=data />
                                </stock_search::HoldingAccount>
                            }
                        })
                        .collect_view();
                    view! {
                        <stock_search::HoldingTotalHeading />
                        <HoldingTilesView data=total />
                        <stock_search::HoldingAccounts>
                            {accounts}
                        </stock_search::HoldingAccounts>
                    }
                        .into_any()
                } else {
                    view! { <HoldingTilesView data=total /> }.into_any()
                }
            }}
        </stock_search::HoldingSection>
    }
}

/// タイルに載せる文言の集まり。表示の組み立て(`HoldingTilesView`)と分けて
/// 挙動の単体テストができるようにする
#[derive(Clone, Debug, PartialEq)]
struct HoldingTilesData {
    market: String,
    pnl: String,
    pnl_rate: String,
    pnl_negative: bool,
    pnl_positive: bool,
    pnl_rate_negative: bool,
    pnl_rate_positive: bool,
    current_price: String,
    price_as_of: String,
    quantity: String,
    average_price: Option<String>,
    purchase: String,
    /// 無配のときだけ真。1株配当・年間配当(予想)・取得額利回りの3項目を
    /// 「無配」の1項目にまとめる(値はどれも無配と出るため)
    no_dividend: bool,
    per_share: String,
    per_share_negative: bool,
    per_share_positive: bool,
    annual: String,
    annual_negative: bool,
    annual_positive: bool,
    yield_text: String,
    yield_negative: bool,
    yield_positive: bool,
    account: Option<String>,
    /// 銘柄共通の項目(現在値・価格基準日・1株配当)を出すか。合計だけに出し、口座ごとには出さない
    show_common: bool,
}

impl HoldingTilesData {
    fn of(input: TileInput<'_>) -> Self {
        let valuation = asset_balance::holding_valuation(&ValuationInput {
            current_price: input.current_price,
            shares: input.shares,
            total_purchase_amount: input.purchase,
        });
        let dividend = asset_balance::holding_dividend(
            input.code,
            input.shares,
            input.average_basis,
            input.maps,
        );
        Self {
            market: asset_balance::format_row_valuation_amount(valuation.market_value),
            pnl: asset_balance::format_signed_row_valuation_amount(valuation.profit_loss.amount),
            pnl_rate: asset_balance::format_signed_valuation_rate(valuation.profit_loss.rate, 1),
            pnl_negative: asset_balance::is_negative_valuation(valuation.profit_loss.amount),
            pnl_positive: asset_balance::is_positive_valuation(valuation.profit_loss.amount),
            pnl_rate_negative: asset_balance::is_negative_rate(valuation.profit_loss.rate),
            pnl_rate_positive: asset_balance::is_positive_rate(valuation.profit_loss.rate),
            current_price: format::format_currency(input.current_price),
            price_as_of: input.price_as_of.map_or_else(
                || labels::FETCH_UNAVAILABLE.to_string(),
                |date| format::format_iso_date(&date),
            ),
            quantity: labels::stock_shares_label(&format::format_number_value(input.shares)),
            average_price: input.average_tile.map(format::format_currency),
            purchase: format::format_currency(input.purchase),
            no_dividend: asset_balance::is_no_dividend(&dividend),
            per_share: asset_balance::format_dividend_per_share(
                &dividend,
                format::unit_price_decimals(dividend.per_share.into_iter()),
            ),
            per_share_negative: dividend.per_share.is_some_and(|value| value < 0.0),
            per_share_positive: dividend.per_share.is_some_and(|value| value > 0.0),
            annual: asset_balance::format_dividend_annual(&dividend),
            annual_negative: dividend.annual.is_some_and(|value| value < 0.0),
            annual_positive: dividend.annual.is_some_and(|value| value > 0.0),
            yield_text: asset_balance::format_dividend_yield(&dividend),
            yield_negative: asset_balance::is_negative_rate(dividend.yield_value),
            yield_positive: asset_balance::is_positive_rate(dividend.yield_value),
            account: input.account,
            show_common: input.show_common,
        }
    }

    fn total(group: &HoldingGroup, maps: &DividendMaps) -> Self {
        let single = group.rows.len() == 1;
        let account = single
            .then(|| group.rows.first().and_then(|row| row.account.clone()))
            .flatten();
        Self::of(TileInput {
            code: &group.code,
            current_price: group.current_price,
            price_as_of: group.price_as_of.clone(),
            shares: group.total_shares,
            average_tile: single.then(|| group.average_price()),
            average_basis: group.average_price(),
            purchase: group.total_purchase,
            maps,
            account,
            show_common: true,
        })
    }

    fn account(row: &HoldingView, group: &HoldingGroup, maps: &DividendMaps) -> Self {
        Self::of(TileInput {
            code: &group.code,
            current_price: row.current_price,
            price_as_of: group.price_as_of.clone(),
            shares: row.shares,
            average_tile: Some(row.average_price),
            average_basis: row.average_price,
            purchase: row.purchase,
            maps,
            account: row.account.clone(),
            show_common: false,
        })
    }
}

/// `HoldingTilesData::of` への入力。引数が増えすぎないよう1つにまとめる
struct TileInput<'a> {
    code: &'a str,
    current_price: f64,
    price_as_of: Option<String>,
    shares: f64,
    /// 取得単価のタイルに出す値。合計では出さないため None
    average_tile: Option<f64>,
    /// 配当利回りの分母。合計では加重平均を使う
    average_basis: f64,
    purchase: f64,
    maps: &'a DividendMaps,
    account: Option<String>,
    /// 銘柄共通の項目(現在値・価格基準日・1株配当)を出すか
    show_common: bool,
}

/// 詳細が奇数項目だと末尾が1つ残るため利回りだけ全幅にする。
/// 無配で畳んだ分(合計は2項目・口座ごとは1項目)も引いて数える
fn detail_item_count(
    show_common: bool,
    has_average: bool,
    has_account: bool,
    collapsed: bool,
) -> usize {
    let count = 4
        + usize::from(show_common) * 3
        + usize::from(has_average)
        + usize::from(show_common && has_account);
    count - usize::from(collapsed) * if show_common { 2 } else { 1 }
}

fn yield_tile_wide(
    show_common: bool,
    has_average: bool,
    has_account: bool,
    collapsed: bool,
) -> bool {
    detail_item_count(show_common, has_average, has_account, collapsed) % 2 == 1
}

#[component]
fn HoldingTilesView(data: HoldingTilesData) -> impl IntoView {
    let summary = data.clone();
    view! {
        <HoldingSummaryTiles data=summary />
        <HoldingDetailTiles data=data />
    }
}

#[component]
fn HoldingSummaryTiles(data: HoldingTilesData) -> impl IntoView {
    view! {
        <HoldingSummary>
            <HoldingSummaryItem label=labels::MARKET_VALUE.to_string()>
                <Amount text=data.market />
            </HoldingSummaryItem>
            <HoldingSummaryItem label=labels::VALUATION_PROFIT_LOSS.to_string()>
                <stock_search::HoldingProfitLoss
                    text=data.pnl
                    negative=data.pnl_negative
                    positive=data.pnl_positive
                    rate=data.pnl_rate
                    rate_negative=data.pnl_rate_negative
                    rate_positive=data.pnl_rate_positive
                />
            </HoldingSummaryItem>
        </HoldingSummary>
    }
}

#[component]
fn CommonDetailTiles(current_price: String, price_as_of: String) -> impl IntoView {
    view! {
        <HoldingDetailItem label=labels::CURRENT_PRICE.to_string()>
            <Amount text=current_price />
        </HoldingDetailItem>
        <HoldingDetailItem label=labels::PRICE_AS_OF.to_string()>
            {price_as_of}
        </HoldingDetailItem>
    }
}

#[component]
fn PerShareDetailTile(per_share: String, negative: bool, positive: bool) -> impl IntoView {
    view! {
        <HoldingDetailItem label=labels::PER_SHARE_DIVIDEND.to_string()>
            <Amount
                kind=AmountKind::Delta
                text=per_share
                negative=negative
                positive=positive
            />
        </HoldingDetailItem>
    }
}

#[component]
fn HoldingDetailTiles(data: HoldingTilesData) -> impl IntoView {
    let show_common = data.show_common;
    let show_account = show_common && data.account.is_some();
    let collapsed = data.no_dividend;
    let wide_dividend = yield_tile_wide(
        show_common,
        data.average_price.is_some(),
        data.account.is_some(),
        collapsed,
    );
    let price_tiles = show_common.then(|| {
        view! {
            <CommonDetailTiles
                current_price=data.current_price.clone()
                price_as_of=data.price_as_of.clone()
            />
        }
    });
    let dividend_tiles = if collapsed {
        view! {
            <HoldingDetailItem
                label=labels::ANNUAL_DIVIDEND_FORECAST.to_string()
                title=labels::ANNUAL_DIVIDEND_NOTE
                wide=wide_dividend
            >
                <Amount
                    kind=AmountKind::Delta
                    text=data.annual.clone()
                    negative=data.annual_negative
                    positive=data.annual_positive
                />
            </HoldingDetailItem>
        }
        .into_any()
    } else {
        let per_share_tile = show_common.then(|| {
            view! {
                <PerShareDetailTile
                    per_share=data.per_share.clone()
                    negative=data.per_share_negative
                    positive=data.per_share_positive
                />
            }
        });
        view! {
            {per_share_tile}
            <HoldingDetailItem
                label=labels::ANNUAL_DIVIDEND_FORECAST.to_string()
                title=labels::ANNUAL_DIVIDEND_NOTE
            >
                <Amount
                    kind=AmountKind::Delta
                    text=data.annual.clone()
                    negative=data.annual_negative
                    positive=data.annual_positive
                />
            </HoldingDetailItem>
            <HoldingDetailItem
                label=labels::ACQUISITION_YIELD.to_string()
                wide=wide_dividend
                title=labels::ACQUISITION_YIELD_NOTE
            >
                <Amount
                    kind=AmountKind::Delta
                    text=data.yield_text.clone()
                    negative=data.yield_negative
                    positive=data.yield_positive
                />
            </HoldingDetailItem>
        }
        .into_any()
    };
    view! {
        <HoldingDetails>
            {price_tiles}
            <HoldingDetailItem label=labels::QUANTITY.to_string()>
                <span class="tabular-nums">{data.quantity.clone()}</span>
            </HoldingDetailItem>
            {data
                .average_price
                .clone()
                .map(|average| {
                    view! {
                        <HoldingDetailItem label=labels::AVERAGE_PRICE.to_string()>
                            <Amount text=average />
                        </HoldingDetailItem>
                    }
                })}
            <HoldingDetailItem label=labels::TOTAL_ACQUISITION.to_string()>
                <Amount text=data.purchase.clone() />
            </HoldingDetailItem>
            {show_account
                .then(|| {
                    data.account
                        .clone()
                        .map(|account| {
                            view! {
                                <HoldingDetailItem label=labels::ACCOUNT.to_string()>
                                    <AccountBadge label=account />
                                </HoldingDetailItem>
                            }
                        })
                })}
            {dividend_tiles}
        </HoldingDetails>
    }
}

#[cfg(test)]
mod tests;
