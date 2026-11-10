use super::{sync_code_param, use_stock_search};
use crate::api::dto::Stock;
use crate::features::asset_balance::{
    HoldingGroup, HoldingView, ValuationInput, all_portfolio_views, format_dividend_annual,
    format_dividend_per_share, format_dividend_yield, format_row_valuation_amount,
    format_signed_row_valuation_amount, format_signed_valuation_rate, group_holdings_by_code,
    holding_dividend, holding_valuation, is_negative_valuation, is_positive_valuation,
    load_asset_balances, resolve_asset_balance, use_asset_balance_state,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::use_session;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::badge::{AccountBadge, CodeBadge};
use crate::ui::card::{Card, CardVariant};
use crate::ui::empty_state::{EmptyState, EmptyStateIcon};
use crate::ui::format::{
    format_currency, format_iso_date, format_number_value, unit_price_decimals,
};
use crate::ui::holding_tiles::{
    HoldingDetailItem, HoldingDetails, HoldingSummary, HoldingSummaryItem,
};
use crate::ui::labels;
use crate::ui::page_frame::PageFrame;
use crate::ui::search_controls::{SearchBox, SearchSubmitButton, SearchTextInput};
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use leptos::prelude::*;
use shared::normalize::normalize_display_name;

const STOCK_LINKS: &[(&str, &str)] = &[
    (
        labels::STOCK_LINK_RAKUTEN,
        "https://www.rakuten-sec.co.jp/web/market/search/quote.html?ric={code}.T",
    ),
    (
        labels::STOCK_LINK_SBI,
        "https://site3.sbisec.co.jp/ETGate/?_ControlID=WPLETsiR001Control&_DataStoreID=DSWPLETsiR001Control&_PageID=WPLETsiR001Ilst10&_ActionID=getDetailOfStockPriceJP&s_rkbn=1&i_stock_sec=%94%43%93%56%93%B0&i_dom_flg=1&i_exchange_code=JPN&i_output_type=0&stock_sec_code_mul={code}",
    ),
    (
        labels::STOCK_LINK_KABUTAN,
        "https://kabutan.jp/stock/?code={code}",
    ),
    ("Yahoo! Finance", "https://finance.yahoo.co.jp/quote/{code}"),
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
    ("IR BANK", "https://irbank.net/{code}"),
    (
        labels::STOCK_LINK_SCOUTER,
        "https://monex.ifis.co.jp/index.php?sa=report_index&bcode={code}",
    ),
    (
        labels::STOCK_LINK_ZAIMANI,
        "https://zaimani.com/search/?_sf_s={code}",
    ),
    ("JPX Explorer", "https://jpx-explorer.com/ja-JP/{code}-TSE"),
];

#[component]
pub(crate) fn StockSearchPage() -> impl IntoView {
    let stock_search = use_stock_search();
    let session = use_session();
    let balance_state = use_asset_balance_state(session);
    let balances = balance_state.balances();
    let dividends = balance_state.dividends();
    let lookup = balance_state.lookup();
    let data_ops = balance_state.data_ops();
    if session.user.get_untracked().is_some() {
        load_asset_balances(
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
        if !sync_code_param(&code) {
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
        let rows = balances
            .with(|slot| resolve_asset_balance(generation, slot).map(|resolved| resolved.rows))?;
        let views = all_portfolio_views(&rows, lookup, generation);
        group_holdings_by_code(&views)
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
        load_asset_balances(
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
                            <div class="py-2" role="search" aria-label=labels::STOCK_SEARCH_TITLE>
                                <SearchForm
                                    stock_code=stock_search.stock_code
                                    loading=loading.into()
                                    icon_size="h-5 w-5"
                                    on_submit=on_submit
                                />
                            </div>
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
                            <div class="py-2" role="search" aria-label=labels::STOCK_SEARCH_TITLE>
                                <SearchForm
                                    stock_code=stock_search.stock_code
                                    loading=loading.into()
                                    icon_size="h-5 w-5"
                                    on_submit=on_submit
                                />
                            </div>
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
        sync_code_param(&code);
    });
    view! {
        <div class="hidden lg:block lg:pb-4" role="search" aria-label=labels::STOCK_SEARCH_TITLE>
            <SearchForm stock_code=stock_search.stock_code loading=loading.into() on_submit=on_submit />
        </div>
        <Show when=move || has_invalid>
            <Alert variant=AlertVariant::Warning>{labels::STOCK_SEARCH_INVALID}</Alert>
        </Show>
        {move || {
            refresh_error
                .get()
                .map(|message| {
                    view! {
                        <div class="no-print mb-3">
                            <Alert variant=AlertVariant::Danger>
                                <strong>{labels::ERROR_PREFIX}</strong>
                                " "
                                {message}
                            </Alert>
                        </div>
                    }
                })
        }}
        {move || {
            let data = stock_search.stock_data();
            let error = stock_search.error_message();
            let not_found = stock_search.is_not_found();
            let is_loading = loading.get();
            if let Some(message) = error {
                view! {
                    <ListLoadError message=message on_retry=move || {
                        let code = stock_search.stock_code.get_untracked();
                        if !code.is_empty() && !stock_search.search.pending().get_untracked() {
                            stock_search.search.dispatch(code);
                        }
                    } />
                }
                    .into_any()
            } else if let Some(balance_message) = balance_error.get() {
                view! {
                    <div class="mb-3">
                        <ListLoadError message=balance_message on_retry=on_retry_balances />
                    </div>
                    {data
                        .map(|stock| {
                            view! { <StockInfo stock=stock /> }.into_any()
                        })}
                }
                    .into_any()
            } else if let Some(stock) = data {
                let group = holding_group.get();
                view! {
                    {group
                        .map(|group| {
                            view! { <HoldingSection group=group dividends=dividends /> }
                        })}
                    <StockInfo stock=stock />
                }
                    .into_any()
            } else if not_found {
                let group = holding_group.get();
                view! {
                    {group
                        .map(|group| {
                            view! { <HoldingSection group=group dividends=dividends /> }
                        })}
                    <EmptySearch title=labels::STOCK_SEARCH_NOT_FOUND_TITLE />
                }
                    .into_any()
            } else if !is_loading {
                view! {
                    <EmptySearch title=labels::STOCK_SEARCH_INITIAL_TITLE />
                }
                    .into_any()
            } else {
                view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }
                    .into_any()
            }
        }}
    }
}

#[component]
fn SearchForm(
    stock_code: RwSignal<String>,
    loading: Signal<bool>,
    /// 虫眼鏡アイコンのサイズ。PC は h-4、スマホは h-5
    #[prop(optional, default = "h-4 w-4")]
    icon_size: &'static str,
    on_submit: impl Fn(web_sys::SubmitEvent) + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <form on:submit=on_submit class="w-full">
            <SearchBox icon_size=icon_size>
                <SearchTextInput
                    placeholder=labels::search_placeholder(labels::STOCK_SEARCH_TARGET)
                    disabled=Signal::derive(move || loading.get())
                    value=move || stock_code.get()
                    on_input=move |value| stock_code.set(value)
                />
                <SearchSubmitButton
                    disabled=move || loading.get() || stock_code.get().is_empty()
                    loading=move || loading.get()
                    label=labels::STOCK_SEARCH_ACTION
                    loading_label=labels::SEARCH_LOADING
                />
            </SearchBox>
        </form>
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
        <EmptyState
            class="py-10"
            title=title
            description=description
            icon=EmptyStateIcon::Search
        >
            <ul class="mt-4 space-y-1 text-sm font-medium text-text-muted">
                <li>{labels::STOCK_SEARCH_TIP_CODE}</li>
                <li>{labels::STOCK_SEARCH_TIP_NAME}</li>
                <li>{labels::STOCK_SEARCH_TIP_LINKS}</li>
            </ul>
        </EmptyState>
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
    let code = stock.code.clone();
    let code_badge = code.clone();
    let name = normalize_display_name(&stock.name);
    view! {
        <div>
            <Card variant=CardVariant::Panel class="mb-6 overflow-hidden">
                <div class="border-b border-ink/10 bg-ink px-4 py-4 text-page sm:px-5">
                    <div class="flex flex-wrap items-center justify-between gap-3">
                        <h2 class="min-w-0 flex-1 break-words text-xl font-black leading-tight">{name}</h2>
                        <CodeBadge>{code_badge}</CodeBadge>
                    </div>
                </div>
                <div>
                    <dl class="grid grid-cols-2 gap-px bg-fill text-sm lg:grid-cols-4">
                        {meta
                            .into_iter()
                            .map(|(label, value)| {
                                let display = if value.is_empty() {
                                    "—".to_string()
                                } else {
                                    value
                                };
                                view! {
                                    <div class="flex min-h-16 flex-col justify-center bg-surface px-4 py-3">
                                        <dt class="text-xs font-medium text-text-muted">
                                            {label}
                                        </dt>
                                        <dd class="mt-1 break-words font-bold tabular-nums text-text-deep">{display}</dd>
                                    </div>
                                }
                            })
                            .collect_view()}
                    </dl>
                    <div class="px-4 py-4">
                        <StockInfoLinks code=code />
                    </div>
                </div>
            </Card>
        </div>
    }
}

#[component]
fn StockInfoLinks(code: String) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 gap-2 lg:grid-cols-4">
            {STOCK_LINKS
                .iter()
                .map(|(name, template)| {
                    let href = template.replace("{code}", &code);
                    view! {
                        <a
                            class="stock-link-button"
                            href={href}
                            target="_blank"
                            rel="noopener noreferrer"
                            aria-label=labels::stock_search_new_tab(name)
                        >
                            {*name}
                            <svg
                                class="h-3.5 w-3.5 shrink-0 text-text-faint"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke="currentColor"
                                aria-hidden="true"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    stroke-width="2"
                                    d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                                />
                            </svg>
                        </a>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// 結果カードの上に出す保有情報。口座が複数ある銘柄は合計の下に口座ごとを並べる。
/// 取得単価は口座ごとにだけ出す
#[component]
fn HoldingSection(group: HoldingGroup, dividends: RwSignal<DividendMaps>) -> impl IntoView {
    let multi = group.rows.len() > 1;
    let badges: Vec<Option<String>> = group.rows.iter().map(|row| row.account.clone()).collect();
    view! {
        <section aria-label=labels::STOCK_HOLDING_SECTION class="mb-6">
            <h2 class="mb-3 text-base font-bold text-text-deep">
                {labels::STOCK_HOLDING_SECTION}
            </h2>
            {move || {
                // 配当は保有一覧より遅れて届くため、ここで読み直してタイルを作り直す
                let maps = dividends.get();
                let total = HoldingTilesData::total(&group, &maps);
                let accounts: Vec<HoldingTilesData> = group
                    .rows
                    .iter()
                    .map(|row| HoldingTilesData::account(row, &group, &maps))
                    .collect();
                if multi {
                    view! {
                        <h3 class="mb-2 text-sm font-bold text-text-muted">
                            {labels::STOCK_HOLDING_TOTAL}
                        </h3>
                        <HoldingTilesView data=total />
                        <div class="mt-4 space-y-4">
                            {accounts
                                .into_iter()
                                .zip(badges.clone())
                                .map(|(data, badge)| {
                                    view! {
                                        <div>
                                            <p class="mb-2">
                                                {badge
                                                    .map(|label| {
                                                        view! { <AccountBadge label=label /> }.into_any()
                                                    })
                                                    .unwrap_or_else(|| labels::MISSING.into_any())}
                                            </p>
                                            <HoldingTilesView data=data />
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                        .into_any()
                } else {
                    view! { <HoldingTilesView data=total /> }.into_any()
                }
            }}
        </section>
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
    current_price: String,
    price_as_of: String,
    quantity: String,
    average_price: Option<String>,
    purchase: String,
    per_share: String,
    annual: String,
    yield_text: String,
    account: Option<String>,
    /// 銘柄共通の項目(現在値・価格基準日・1株配当)を出すか。合計だけに出し、口座ごとには出さない
    show_common: bool,
}

impl HoldingTilesData {
    fn of(input: TileInput<'_>) -> Self {
        let valuation = holding_valuation(&ValuationInput {
            current_price: input.current_price,
            shares: input.shares,
            total_purchase_amount: input.purchase,
        });
        let dividend = holding_dividend(input.code, input.shares, input.average_basis, input.maps);
        Self {
            market: format_row_valuation_amount(valuation.market_value),
            pnl: format_signed_row_valuation_amount(valuation.profit_loss.amount),
            pnl_rate: format_signed_valuation_rate(valuation.profit_loss.rate, 1),
            pnl_negative: is_negative_valuation(valuation.profit_loss.amount),
            pnl_positive: is_positive_valuation(valuation.profit_loss.amount),
            current_price: format_currency(input.current_price),
            price_as_of: input.price_as_of.map_or_else(
                || labels::MISSING.to_string(),
                |date| format_iso_date(&date),
            ),
            quantity: labels::stock_shares_label(&format_number_value(input.shares)),
            average_price: input.average_tile.map(format_currency),
            purchase: format_currency(input.purchase),
            per_share: format_dividend_per_share(
                &dividend,
                unit_price_decimals(dividend.per_share.into_iter()),
            ),
            annual: format_dividend_annual(&dividend),
            yield_text: format_dividend_yield(&dividend),
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

#[component]
fn HoldingTilesView(data: HoldingTilesData) -> impl IntoView {
    let show_common = data.show_common;
    let current_price = data.current_price.clone();
    let price_as_of = data.price_as_of.clone();
    let per_share = data.per_share.clone();
    view! {
        <HoldingSummary>
            <HoldingSummaryItem label=labels::MARKET_VALUE.to_string()>
                <span class="tabular-nums">{data.market}</span>
            </HoldingSummaryItem>
            <HoldingSummaryItem label=labels::VALUATION_PROFIT_LOSS.to_string()>
                <Amount
                    kind=AmountKind::Delta
                    class="tabular-nums"
                    text=data.pnl
                    negative=data.pnl_negative
                    positive=data.pnl_positive
                />
                <span class="ml-1 text-xs text-text-muted tabular-nums">{data.pnl_rate}</span>
            </HoldingSummaryItem>
        </HoldingSummary>
        <HoldingDetails>
            {show_common
                .then(|| {
                    view! {
                        <HoldingDetailItem label=labels::CURRENT_PRICE.to_string()>
                            <span class="tabular-nums">{current_price}</span>
                        </HoldingDetailItem>
                    }
                })}
            {show_common
                .then(|| {
                    view! {
                        <HoldingDetailItem label=labels::PRICE_AS_OF.to_string()>
                            {price_as_of}
                        </HoldingDetailItem>
                    }
                })}
            <HoldingDetailItem label=labels::QUANTITY.to_string()>
                <span class="tabular-nums">{data.quantity}</span>
            </HoldingDetailItem>
            {data
                .average_price
                .map(|average| {
                    view! {
                        <HoldingDetailItem label=labels::AVERAGE_PRICE.to_string()>
                            <span class="tabular-nums">{average}</span>
                        </HoldingDetailItem>
                    }
                })}
            <HoldingDetailItem label=labels::TOTAL_ACQUISITION.to_string()>
                <span class="tabular-nums">{data.purchase}</span>
            </HoldingDetailItem>
            {(show_common && data.account.is_some())
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
            {show_common
                .then(|| {
                    view! {
                        <HoldingDetailItem label=labels::PER_SHARE_DIVIDEND.to_string()>
                            <span class="tabular-nums">{per_share}</span>
                        </HoldingDetailItem>
                    }
                })}
            <HoldingDetailItem label=labels::ANNUAL_DIVIDEND_REFERENCE.to_string()>
                <span class="tabular-nums">{data.annual}</span>
            </HoldingDetailItem>
            <HoldingDetailItem label=labels::ACQUISITION_BASIS_YIELD.to_string()>
                <span class="tabular-nums">{data.yield_text}</span>
            </HoldingDetailItem>
        </HoldingDetails>
    }
}

#[cfg(test)]
mod tests;
