mod all_list;
mod asset_table;
mod composition_page;
mod day_change;
mod fund_detail_page;
mod fund_table;
mod holding_detail_page;
mod holdings_list;
mod kind_tabs;
mod main_content;
mod search_bar;
mod sort_header;
mod summary;

#[cfg(test)]
#[path = "view/tests/suite.rs"]
mod tests;

pub use composition_page::AssetBalanceCompositionPage;
pub use fund_detail_page::AssetFundHoldingPage;
pub use holding_detail_page::AssetBalanceHoldingPage;

use crate::features::asset_balance::fund::{AssetKind, AssetSearch, FundSortKey, text_only_search};
use crate::features::asset_balance::lookup::AssetBalanceLookupStore;
use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::features::asset_balance::store::csv::{
    AssetBalanceCsvStore, AssetCsvSlot, asset_balance_counts, fund_balance_counts,
    resolve_asset_balance, resolve_fund_balance,
};
use crate::features::asset_balance::store::{
    BalanceSlot, DataOps, FundSlot, has_current_balances, load_asset_balances, load_fund_balances,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::{SessionStore, use_session};
use crate::ui::labels;
use crate::ui::page_frame::PageFrame;
use crate::ui::page_skeleton::SkeletonTabs;
use crate::ui::state::{Alert, AlertVariant, ListLoadError, ListSkeleton, ListSkeletonVariant};
use crate::ui::underline_tabs::{SwipeArea, SwipeMotion, SwipeSlide};
use kind_tabs::AssetKindTabs;
use leptos::prelude::*;
use main_content::AssetBalanceMainContent;

/// タブごとの表示に要るもの。指で引いた先の面も同じ値で描く
#[derive(Clone, Copy)]
struct KindContent {
    session: SessionStore,
    balances: RwSignal<BalanceSlot>,
    funds: RwSignal<FundSlot>,
    search: RwSignal<AssetSearch>,
    counts: [RwSignal<(usize, usize)>; 3],
    kinds: [RwSignal<AssetKind>; 3],
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
    fund_sort_key: RwSignal<FundSortKey>,
    fund_sort_order: RwSignal<SortOrder>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    csv: RwSignal<AssetCsvSlot>,
    data_ops: RwSignal<DataOps>,
}

impl KindContent {
    fn reload(self) {
        if self.session.user.get_untracked().is_none() {
            return;
        }
        self.balances.set(None);
        load_asset_balances(
            self.session,
            self.session.generation.get_untracked(),
            self.balances,
            self.dividends,
            self.lookup,
            self.csv,
            self.data_ops,
        );
        self.funds.set(None);
        load_fund_balances(
            self.session,
            self.session.generation.get_untracked(),
            self.funds,
            self.data_ops,
        );
    }

    fn render(self, kind: AssetKind) -> AnyView {
        let generation = self.session.generation.get();
        let list_error = self.balances.with(|slot| match slot {
            Some((cached, Err(message))) if *cached == generation => Some(message.clone()),
            _ => None,
        });
        let resolved = self
            .balances
            .with(|slot| resolve_asset_balance(generation, slot));
        let resolved_fund = self
            .funds
            .with(|slot| resolve_fund_balance(generation, slot));
        let index = AssetKind::ALL
            .iter()
            .position(|item| *item == kind)
            .unwrap_or(0);
        match (resolved, resolved_fund, list_error) {
            (None, _, _) | (_, None, _) => {
                view! { <ListSkeleton variant=ListSkeletonVariant::Cards /> }.into_any()
            }
            (Some(_), _, Some(message)) => {
                let this = self;
                view! { <ListLoadError message=message on_retry=move || this.reload() /> }
                    .into_any()
            }
            (Some(resolved), Some(resolved_fund), None) => view! {
                <AssetBalanceMainContent
                    stock_rows=resolved.rows
                    fallback_totals=resolved.fallback_totals
                    fund_resolved=resolved_fund
                    search=self.search
                    search_count=self.counts[index]
                    sort_key=self.sort_key
                    sort_order=self.sort_order
                    fund_sort_key=self.fund_sort_key
                    fund_sort_order=self.fund_sort_order
                    kind=self.kinds[index]
                    dividends=self.dividends
                    lookup=self.lookup
                    generation=generation
                />
            }
            .into_any(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct AssetBalanceState {
    balances: RwSignal<BalanceSlot>,
    funds: RwSignal<FundSlot>,
    dividends: RwSignal<DividendMaps>,
    lookup: RwSignal<AssetBalanceLookupStore>,
    search: RwSignal<AssetSearch>,
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
    fund_sort_key: RwSignal<FundSortKey>,
    fund_sort_order: RwSignal<SortOrder>,
    pub(crate) kind: RwSignal<AssetKind>,
    data_ops: RwSignal<DataOps>,
    pub(crate) csv: AssetBalanceCsvStore,
}

impl AssetBalanceState {
    /// CSV 最新取得など画面の保存操作を通らない取り込みのあと、保存済み一覧を取り直す
    pub(crate) fn reload_lists(&self, session: SessionStore, stock: bool, fund: bool) {
        if session.user.get_untracked().is_none() {
            return;
        }
        let generation = session.generation.get_untracked();
        if stock {
            load_asset_balances(
                session,
                generation,
                self.balances,
                self.dividends,
                self.lookup,
                self.csv.csv,
                self.data_ops,
            );
        }
        if fund {
            load_fund_balances(session, generation, self.funds, self.data_ops);
        }
    }
}

pub(crate) fn use_asset_balance_state(session: SessionStore) -> AssetBalanceState {
    #[cfg(test)]
    {
        // テストでは呼び出しごとに新しい状態を返して独立性を保つ
        build_asset_balance_state(session)
    }
    #[cfg(not(test))]
    {
        // ページ遷移でビューを作り直しても取得済みデータを失わないよう、App コンテキストのスロットに保持する
        let states = use_context::<crate::app::PageStates>().unwrap_or_default();
        crate::app::page_state(states.asset_balance, || build_asset_balance_state(session))
    }
}

fn build_asset_balance_state(session: SessionStore) -> AssetBalanceState {
    let balances: RwSignal<BalanceSlot> = RwSignal::new(None);
    let funds: RwSignal<FundSlot> = RwSignal::new(None);
    let dividends: RwSignal<DividendMaps> = RwSignal::new(DividendMaps::default());
    let lookup = RwSignal::new(AssetBalanceLookupStore::new());
    let search = RwSignal::new(AssetSearch::default());
    let reset_search = search;
    let sort_key = RwSignal::new(SortKey::MarketValue);
    let sort_order = RwSignal::new(SortOrder::Desc);
    // 初期の並び順は設定から。設定が変わったら並び順に反映する(ユーザーの手動選択は設定変更まで保持)
    let configured_sort = crate::settings::try_use_settings().asset_sort;
    Effect::new(move |_| {
        let sort = configured_sort.get();
        sort_key.set(sort.key);
        sort_order.set(sort.order);
    });
    let fund_sort_key = RwSignal::new(FundSortKey::MarketValue);
    let fund_sort_order = RwSignal::new(SortOrder::Desc);
    let kind = RwSignal::new(AssetKind::Stock);
    let data_ops: RwSignal<DataOps> = RwSignal::new(DataOps::default());
    let csv_store =
        AssetBalanceCsvStore::new(session, balances, funds, dividends, lookup, data_ops);
    let csv_slot = csv_store.csv;
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            lookup.update(|store| store.clear());
            balances.set(None);
            funds.set(None);
            dividends.set(DividendMaps::default());
            data_ops.update(DataOps::reset);
            reset_search.set(AssetSearch::default());
            return;
        }
        lookup.update(|store| store.clear_if_stale(generation));
        if !has_current_balances(&balances.get_untracked(), generation) {
            load_asset_balances(
                session, generation, balances, dividends, lookup, csv_slot, data_ops,
            );
        }
        if !matches!(funds.get_untracked(), Some((cached, _)) if cached == generation) {
            load_fund_balances(session, generation, funds, data_ops);
        }
    });
    AssetBalanceState {
        balances,
        funds,
        dividends,
        lookup,
        search,
        sort_key,
        sort_order,
        fund_sort_key,
        fund_sort_order,
        kind,
        data_ops,
        csv: csv_store,
    }
}

#[component]
pub fn AssetBalancePage() -> impl IntoView {
    let session = use_session();
    let busy_session = session;
    let rail_session = session;
    let state = use_asset_balance_state(session);
    let balances = state.balances;
    let funds = state.funds;
    let dividends = state.dividends;
    let lookup = state.lookup;
    let search = state.search;
    let data_ops = state.data_ops;
    let busy_ops = data_ops;
    let csv_slot = state.csv.csv;
    let motion = SwipeMotion::new();
    let content = KindContent {
        session,
        balances,
        funds,
        search,
        counts: std::array::from_fn(|_| RwSignal::new((0usize, 0usize))),
        kinds: std::array::from_fn(|index| RwSignal::new(AssetKind::ALL[index])),
        sort_key: state.sort_key,
        sort_order: state.sort_order,
        fund_sort_key: state.fund_sort_key,
        fund_sort_order: state.fund_sort_order,
        dividends,
        lookup,
        csv: csv_slot,
        data_ops,
    };
    // 口座・コースの選択肢はタブごとに語彙が違うため、切替時は選択条件だけ消す
    // (テキストは全タブ共通で引き継ぐ)
    let prev_kind = RwSignal::new(state.kind.get_untracked());
    Effect::new(move |_| {
        let next = state.kind.get();
        if prev_kind.get_untracked() == next {
            return;
        }
        prev_kind.set(next);
        search.update(|search| *search = text_only_search(search));
    });
    // タブの件数表示用。タブ行をページレベルで描くためここで算出
    let stock_count = Signal::derive(move || {
        let generation = rail_session.generation.get();
        balances
            .with(|slot| asset_balance_counts(generation, slot))
            .map(|(rows_len, _)| rows_len)
            .unwrap_or(0)
    });
    let fund_count = Signal::derive(move || {
        let generation = rail_session.generation.get();
        funds
            .with(|slot| fund_balance_counts(generation, slot))
            .map(|(rows_len, total)| summary::fund_tab_count(rows_len, total))
            .unwrap_or(0)
    });
    if session.user.get_untracked().is_some()
        && has_current_balances(
            &balances.get_untracked(),
            session.generation.get_untracked(),
        )
        && !data_ops.with_untracked(|ops| ops.list_fetch_in_flight())
    {
        load_asset_balances(
            session,
            session.generation.get_untracked(),
            balances,
            dividends,
            lookup,
            csv_slot,
            data_ops,
        );
        load_fund_balances(session, session.generation.get_untracked(), funds, data_ops);
    }
    let current_tab = Signal::derive(move || {
        AssetKind::ALL
            .iter()
            .position(|kind| *kind == state.kind.get())
            .unwrap_or(0)
    });
    let csv_link: Children = Box::new(move || {
        view! {
            <span class="flex items-center gap-2">
                <a class="page-secondary-link-sm max-lg:hidden" href="/assetbalance/composition">{labels::COMPOSITION_VIEW_LINK}</a>
                <a class="page-secondary-link-sm" href="/csv">{labels::CSV_MANAGE_TITLE}</a>
            </span>
        }
        .into_any()
    });
    view! {
        <PageFrame title=labels::ASSET_BALANCE_TITLE full_bleed=true links=csv_link>
        <div
            class="flex flex-1 flex-col min-h-0 lg:flex-none"
            data-testid="assetbalance-workspace"
            aria-busy=move || {
                if !busy_session.loaded.get()
                    || busy_ops.with(|ops| !ops.inflight.is_empty())
                    || (busy_session.user.get().is_some()
                        && balances
                            .get()
                            .filter(|(cached, _)| {
                                *cached == busy_session.generation.get()
                            })
                            .is_none()) {
                    "true"
                } else {
                    "false"
                }
            }
        >
            {/* タブ行: 固定上部 */}
            <div class="content-gutter no-print">
                <SkeletonTabs>
                    <AssetKindTabs
                        kind=state.kind
                        all_count=Signal::derive(move || stock_count.get() + fund_count.get())
                        stock_count=stock_count
                        fund_count=fund_count
                        motion=motion
                    />
                </SkeletonTabs>
            </div>
            {/* 本文: スクロール可能。スマホ幅では横スワイプでタブを切り替える */}
            <SwipeArea
                testid="assetbalance-main-stage"
                count=Signal::derive(move || AssetKind::ALL.len())
                current=current_tab
                on_select=Callback::new(move |next: usize| state.kind.set(AssetKind::ALL[next]))
                motion=motion
            >
                {move || {
                    data_ops
                        .with(|ops| ops.refresh_error.clone())
                        .map(|message| {
                            view! {
                                <div class="no-print mb-3 px-4 lg:px-0">
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
                    let generation = rail_session.generation.get();
                    balances
                        .with(|slot| resolve_asset_balance(generation, slot))
                        .and_then(|view| view.warning)
                        .map(|message| {
                        view! {
                            <div class="no-print mb-3 px-4 lg:px-0">
                                <Alert variant=AlertVariant::Warning>
                                    <strong>{labels::WARNING_PREFIX}</strong>
                                    " "
                                    <span data-testid="truncated-list-warning">{message}</span>
                                </Alert>
                            </div>
                        }
                    })
                }}
                {move || {
                    let generation = rail_session.generation.get();
                    funds
                        .with(|slot| resolve_fund_balance(generation, slot))
                        .and_then(|view| view.warning)
                        .map(|message| {
                        view! {
                            <div class="no-print mb-3 px-4 lg:px-0">
                                <Alert variant=AlertVariant::Warning>
                                    <strong>{labels::WARNING_PREFIX}</strong>
                                    " "
                                    <span data-testid="fund-truncated-list-warning">{message}</span>
                                </Alert>
                            </div>
                        }
                    })
                }}
                {AssetKind::ALL
                    .iter()
                    .enumerate()
                    .map(|(index, kind)| {
                        let kind = *kind;
                        view! {
                            <SwipeSlide index=index current=current_tab motion=motion>
                                {move || content.render(kind)}
                            </SwipeSlide>
                        }
                    })
                    .collect_view()}
            </SwipeArea>
        </div>
        </PageFrame>
    }
}
