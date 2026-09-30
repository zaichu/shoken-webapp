use crate::api::dto::{
    AssetBalanceListResponse, AssetBalanceSummary, DividendListResponse, DividendSummary,
};
use crate::api::{ApiClient, ApiError};
use crate::features::asset_balance::{
    calculate_valuation_from_decimal, format_valuation_amount, format_valuation_rate,
    valuation_tone,
};
use crate::session::{use_session, Generation, SessionStore};
use crate::ui::amount::Amount;
use crate::ui::card::{Card, CardVariant, SectionHeader, SectionHeaderVariant};
use crate::ui::elements::Skeleton;
use leptos::prelude::*;
use shared::format::format_currency as format_currency_decimal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;

const STATUS_ITEMS: &[(&str, &str, Option<&str>, &str, &str)] = &[
    (
        "銘柄検索",
        "検索",
        Some("/search"),
        "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z",
        "h-5 w-5 text-ink",
    ),
    (
        "資産管理",
        "一覧確認",
        Some("/assetbalance"),
        "M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z",
        "h-5 w-5 text-ink",
    ),
    (
        "取引明細",
        "明細確認",
        Some("/receipts"),
        "M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
        "h-5 w-5 text-ink",
    ),
    (
        "CSV取込",
        "CSV反映",
        None,
        "M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12",
        "h-5 w-5 text-text-subtle",
    ),
];

const FLOW_STEPS: &[(&str, &str)] = &[
    ("01", "CSV取得"),
    ("02", "各ページで取込"),
    ("03", "資産と明細を確認"),
];

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="space-y-7">
            <div>
                <h1 class="text-3xl font-black leading-tight tracking-normal text-ink sm:text-4xl">
                    "証券Web"
                </h1>
                <p class="mt-2 max-w-2xl text-sm font-medium text-text-muted">
                    "資産、配当、取引明細をひとつの作業面で確認します。"
                </p>
            </div>

            <HomeOverview />

            <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
                {STATUS_ITEMS
                    .iter()
                    .map(|(label, sub, to, icon_d, icon_class)| {
                        let inner = view! {
                            <div class="flex items-start justify-between gap-3">
                                <span class="inline-flex h-10 w-10 items-center justify-center rounded-md border border-ink/10 bg-surface shadow-sm">
                                    <svg
                                        class={*icon_class}
                                        fill="none"
                                        viewBox="0 0 24 24"
                                        stroke="currentColor"
                                        aria-hidden="true"
                                        focusable="false"
                                    >
                                        <path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="1.8"
                                            d={*icon_d}
                                        />
                                    </svg>
                                </span>
                                <span class="text-xs font-bold text-text-muted">
                                    {*sub}
                                </span>
                            </div>
                            <p class="mt-4 text-base font-black text-ink">{*label}</p>
                        };
                        match to {
                            Some(to) => {
                                view! {
                                    <Card variant=CardVariant::Feature class="group" href=(*to).to_string()>
                                        {inner}
                                    </Card>
                                }
                                    .into_any()
                            }
                            None => {
                                view! {
                                    <Card variant=CardVariant::Dashed>
                                        {inner}
                                        <p class="mt-2 text-xs font-medium text-text-muted">"各ページから取込可能"</p>
                                    </Card>
                                }
                                    .into_any()
                            }
                        }
                    })
                    .collect_view()}
            </div>

            <section class="border-t border-ink/10 pt-5">
                <SectionHeader variant=SectionHeaderVariant::Divider>
                    "データ確認フロー"
                </SectionHeader>
                <div class="mt-3 grid gap-2 sm:grid-cols-3">
                    {FLOW_STEPS
                        .iter()
                        .map(|(step, text)| {
                            view! {
                                <Card variant=CardVariant::Step>
                                    <span class="text-xs font-black tabular-nums text-accent-deep">
                                        {*step}
                                    </span>
                                    <p class="mt-1 text-sm font-bold text-text">{*text}</p>
                                </Card>
                            }
                        })
                        .collect_view()}
                </div>
            </section>
        </div>
    }
}

// 外側の None は取得中、内側の None は取得失敗
type SummarySlot<T> = RwSignal<Option<(Generation, Option<T>)>>;

async fn fetch_asset_summary() -> Result<Option<AssetBalanceSummary>, ApiError> {
    ApiClient::read_client()
        .get_json::<AssetBalanceListResponse>(
            "/api/v1/asset-balances",
            &[("per_page", "1"), ("include_summary", "true")],
        )
        .await
        .map(|response| response.summary)
}

async fn fetch_dividend_summary(year: u32) -> Result<Option<DividendSummary>, ApiError> {
    let year = year.to_string();
    ApiClient::read_client()
        .get_json::<DividendListResponse>(
            "/api/v1/dividends",
            &[
                ("per_page", "1"),
                ("year", year.as_str()),
                ("include_summary", "true"),
            ],
        )
        .await
        .map(|response| response.summary)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SummaryKind {
    Asset,
    Dividend,
}

#[derive(Default)]
struct SummaryFetch {
    rev: u64,
    inflight: bool,
    // 裏再取得が失敗したときだけ立て、表示済みの集計は残したまま失敗を伝える
    failed: bool,
}

// 往復で要求を重ねないよう取得中は None を返す。返した rev で応答の新しさを判定する
fn begin_summary_fetch(
    map: &mut HashMap<(Generation, SummaryKind), SummaryFetch>,
    generation: Generation,
    kind: SummaryKind,
) -> Option<u64> {
    let entry = map.entry((generation, kind)).or_default();
    if entry.inflight {
        return None;
    }
    entry.inflight = true;
    entry.rev += 1;
    Some(entry.rev)
}

// 応答が最新の取得のものなら true。古い取得なら inflight を下ろさず応答は捨てる
fn finish_summary_fetch(
    map: &mut HashMap<(Generation, SummaryKind), SummaryFetch>,
    generation: Generation,
    kind: SummaryKind,
    rev: u64,
) -> bool {
    let Some(entry) = map.get_mut(&(generation, kind)) else {
        return false;
    };
    if entry.rev != rev {
        return false;
    }
    entry.inflight = false;
    true
}

// 裏再取得の失敗で表示済みの集計を消さないため None=既存を維持。初回失敗は従来どおり失敗として書く
fn settle_summary<T>(
    current: &Option<(Generation, Option<T>)>,
    generation: Generation,
    result: Result<Option<T>, ApiError>,
) -> Option<(Generation, Option<T>)> {
    match result {
        Ok(summary) => Some((generation, summary)),
        Err(_) => match current {
            Some((cached, Some(_))) if *cached == generation => None,
            _ => Some((generation, None)),
        },
    }
}

fn load_summary<T: Clone + Send + Sync + 'static>(
    session: SessionStore,
    generation: Generation,
    kind: SummaryKind,
    slot: SummarySlot<T>,
    fetch_state: RwSignal<HashMap<(Generation, SummaryKind), SummaryFetch>>,
    fetch: impl Future<Output = Result<Option<T>, ApiError>> + 'static,
) {
    let Some(rev) = fetch_state
        .try_update(|map| begin_summary_fetch(map, generation, kind))
        .flatten()
    else {
        return;
    };
    leptos::task::spawn_local(async move {
        let result = fetch.await;
        let is_current = fetch_state
            .try_update(|map| finish_summary_fetch(map, generation, kind, rev))
            .unwrap_or_default();
        if !session.is_current(generation) || !is_current {
            return;
        }
        if result.as_ref().is_err_and(ApiError::is_unauthorized) {
            session.mark_unauthenticated();
            return;
        }
        let failed = result.is_err();
        fetch_state.update(|map| {
            if let Some(entry) = map.get_mut(&(generation, kind)) {
                entry.failed = failed;
            }
        });
        if let Some(next) = settle_summary(&slot.get_untracked(), generation, result) {
            slot.set(Some(next));
        }
    });
}

fn current_summary<T: Clone + Send + Sync + 'static>(
    slot: SummarySlot<T>,
    generation: Generation,
) -> Option<Option<T>> {
    slot.get()
        .filter(|(cached, _)| *cached == generation)
        .map(|(_, summary)| summary)
}

fn has_current_summary<T>(slot: &Option<(Generation, Option<T>)>, generation: Generation) -> bool {
    matches!(slot, Some((cached, _)) if *cached == generation)
}

#[derive(Clone, Copy)]
struct HomeOverviewData {
    asset: SummarySlot<AssetBalanceSummary>,
    dividend: SummarySlot<DividendSummary>,
    fetch_state: RwSignal<HashMap<(Generation, SummaryKind), SummaryFetch>>,
}

thread_local! {
    // ページ遷移でビューを作り直しても取得済みの集計を失わないよう、アプリ寿命のオーナーに作る。
    // Owner::new() は現オーナーの子として登録されページと一緒に破棄されるため、AppOwner の子を使う
    static OVERVIEW: RefCell<Option<(Owner, HomeOverviewData)>> = const { RefCell::new(None) };
}

fn use_home_overview(session: SessionStore) -> HomeOverviewData {
    let app_owner = use_context::<crate::app::AppOwner>()
        .map(|app| app.0)
        .unwrap_or_default();
    OVERVIEW.with(|cell| {
        if let Some((_, data)) = cell.borrow().as_ref() {
            return *data;
        }
        let owner = app_owner.child();
        let data = owner.with(|| build_home_overview(session));
        *cell.borrow_mut() = Some((owner, data));
        data
    })
}

fn build_home_overview(session: SessionStore) -> HomeOverviewData {
    let data = HomeOverviewData {
        asset: RwSignal::new(None),
        dividend: RwSignal::new(None),
        fetch_state: RwSignal::new(HashMap::new()),
    };
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            data.asset.set(None);
            data.dividend.set(None);
            // 前ユーザーの取得中エントリが残ると次ユーザーの同種取得が欠かされる
            data.fetch_state
                .update(|map| map.retain(|(cached, _), _| *cached == generation));
            return;
        }
        load_home_summaries(session, generation, data);
    });
    data
}

fn load_home_summaries(session: SessionStore, generation: Generation, data: HomeOverviewData) {
    load_summary(
        session,
        generation,
        SummaryKind::Asset,
        data.asset,
        data.fetch_state,
        fetch_asset_summary(),
    );
    load_summary(
        session,
        generation,
        SummaryKind::Dividend,
        data.dividend,
        data.fetch_state,
        fetch_dividend_summary(js_sys::Date::new_0().get_full_year()),
    );
}

#[component]
fn HomeOverview() -> impl IntoView {
    let session = use_session();
    let data = use_home_overview(session);
    let asset = data.asset;
    let dividend = data.dividend;
    // 再訪では表示済みの集計を消さず、取得済みの slot だけ裏で取り直す(未取得は Effect が担う)
    if session.user.get_untracked().is_some() {
        let generation = session.generation.get_untracked();
        if has_current_summary(&asset.get_untracked(), generation) {
            load_summary(
                session,
                generation,
                SummaryKind::Asset,
                asset,
                data.fetch_state,
                fetch_asset_summary(),
            );
        }
        if has_current_summary(&dividend.get_untracked(), generation) {
            load_summary(
                session,
                generation,
                SummaryKind::Dividend,
                dividend,
                data.fetch_state,
                fetch_dividend_summary(js_sys::Date::new_0().get_full_year()),
            );
        }
    }
    let snapshot = move || {
        let generation = session.generation.get();
        (
            current_summary(asset, generation),
            current_summary(dividend, generation),
        )
    };
    view! {
        <Show when=move || session.user.get().is_some()>
            <section
                aria-label="資産の概要"
                aria-busy=move || {
                    let (asset, dividend) = snapshot();
                    if asset.is_none() || dividend.is_none() { "true" } else { "false" }
                }
                data-testid="home-overview"
            >
                {move || {
                    let (asset, dividend) = snapshot();
                    view! { <OverviewTiles asset=asset dividend=dividend /> }
                }}
                <p class="text-xs font-medium text-text-muted" role="status" aria-live="polite">
                    {move || {
                        let (asset, dividend) = snapshot();
                        let refresh_failed = data.fetch_state.with(|map| {
                            let generation = session.generation.get();
                            [SummaryKind::Asset, SummaryKind::Dividend].iter().any(|kind| {
                                map.get(&(generation, *kind))
                                    .is_some_and(|entry| entry.failed)
                            })
                        });
                        (matches!((asset, dividend), (Some(None), _) | (_, Some(None)))
                            || refresh_failed)
                            .then(|| {
                                view! {
                                    <span class="mt-2 block">"一部の集計を取得できませんでした。"</span>
                                }
                            })
                    }}
                </p>
            </section>
        </Show>
    }
}

#[component]
fn OverviewTiles(
    asset: Option<Option<AssetBalanceSummary>>,
    dividend: Option<Option<DividendSummary>>,
) -> impl IntoView {
    let asset_busy = asset.is_none();
    let dividend_busy = dividend.is_none();
    let asset = asset.flatten();
    let market = asset
        .as_ref()
        .map(|summary| format_currency_decimal(summary.total_market_value));
    let valuation = asset
        .as_ref()
        .filter(|summary| {
            !(summary.total_market_value.is_zero() && summary.total_purchase_amount.is_zero())
        })
        .map(|summary| {
            calculate_valuation_from_decimal(
                summary.total_market_value,
                summary.total_purchase_amount,
            )
        });
    let (profit_class, profit_negative) =
        valuation_tone(valuation.as_ref().and_then(|valuation| valuation.amount));
    let profit_rate = valuation.as_ref().map(|valuation| match valuation.rate {
        Some(rate) => format_valuation_rate(Some(rate), 1),
        None => "算出不可".to_string(),
    });
    let profit = valuation.map(|valuation| format_valuation_amount(valuation.amount));
    let dividend = dividend
        .flatten()
        .map(|summary| format_currency_decimal(summary.total_net_amount_received));
    view! {
        <div class="grid grid-cols-1 gap-3 lg:grid-cols-3">
            <OverviewTile label="評価額" value=market busy=asset_busy />
            <OverviewTile
                label="評価損益"
                value=profit
                busy=asset_busy
                value_class=profit_class
                negative=profit_negative
                note=profit_rate
            />
            <OverviewTile label="今年の税引後配当金" value=dividend busy=dividend_busy />
        </div>
    }
}

#[component]
fn OverviewTile(
    label: &'static str,
    value: Option<String>,
    busy: bool,
    #[prop(default = "text-ink")] value_class: &'static str,
    #[prop(default = None)] negative: Option<&'static str>,
    #[prop(default = None)] note: Option<String>,
) -> impl IntoView {
    view! {
        <Card variant=CardVariant::Tile>
            <p class="text-sm font-medium text-text-muted">{label}</p>
            {if busy {
                view! { <Skeleton class="mt-2 h-7 w-32" /> }.into_any()
            } else {
                view! {
                    <p
                        class=format!(
                            "mt-1 flex flex-wrap items-baseline gap-x-2 text-2xl font-black tabular-nums {value_class}",
                        )
                        data-negative=negative
                    >
                        <Amount
                            class="break-all"
                            text=value.unwrap_or_else(|| "—".to_string())
                            negative=negative.is_some()
                        />
                        {note
                            .map(|note| {
                                view! {
                                    <span class="whitespace-nowrap text-sm font-bold">{format!("（{note}）")}</span>
                                }
                            })}
                    </p>
                }
                    .into_any()
            }}
        </Card>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_current_summary_only_matches_same_generation() {
        let generation = Generation::new(1);
        let other = Generation::new(2);
        let slot: Option<(Generation, Option<u32>)> = Some((generation, Some(7)));
        assert!(has_current_summary(&slot, generation));
        assert!(!has_current_summary(&slot, other));
        assert!(!has_current_summary::<u32>(
            &Some((generation, None)),
            other
        ));
        assert!(!has_current_summary::<u32>(&None, generation));
    }

    #[test]
    fn summary_fetch_blocks_second_request_while_inflight() {
        let generation = Generation::new(1);
        let mut map = HashMap::new();
        assert_eq!(
            begin_summary_fetch(&mut map, generation, SummaryKind::Asset),
            Some(1)
        );
        assert_eq!(
            begin_summary_fetch(&mut map, generation, SummaryKind::Asset),
            None,
            "取得中の同じ集計を重ねて要求しない"
        );
        assert_eq!(
            begin_summary_fetch(&mut map, generation, SummaryKind::Dividend),
            Some(1),
            "別の集計は取得中でも取れる"
        );
        assert!(finish_summary_fetch(
            &mut map,
            generation,
            SummaryKind::Asset,
            1
        ));
        assert_eq!(
            begin_summary_fetch(&mut map, generation, SummaryKind::Asset),
            Some(2),
            "応答後は再取得できる"
        );
    }

    #[test]
    fn summary_fetch_drops_stale_response() {
        let generation = Generation::new(1);
        let mut map = HashMap::new();
        let rev = begin_summary_fetch(&mut map, generation, SummaryKind::Asset).unwrap();
        assert!(finish_summary_fetch(
            &mut map,
            generation,
            SummaryKind::Asset,
            rev
        ));
        let next = begin_summary_fetch(&mut map, generation, SummaryKind::Asset).unwrap();
        assert!(
            !finish_summary_fetch(&mut map, generation, SummaryKind::Asset, rev),
            "古い応答は inflight を下ろさない"
        );
        assert!(
            map.get(&(generation, SummaryKind::Asset))
                .is_some_and(|entry| entry.inflight),
            "新しい取得はまだ取得中のまま"
        );
        assert!(finish_summary_fetch(
            &mut map,
            generation,
            SummaryKind::Asset,
            next
        ));
        // 別世代のキーは同じ rev でも混ざらない
        let other = Generation::new(2);
        assert_eq!(
            begin_summary_fetch(&mut map, other, SummaryKind::Asset),
            Some(1)
        );
    }

    #[test]
    fn settle_summary_keeps_displayed_value_on_refresh_error() {
        let generation = Generation::new(1);
        let error = || -> Result<Option<u32>, ApiError> { Err(ApiError::http(500)) };
        // 裏再取得の失敗では表示済みの値を消さない(書き戻さない)
        let current = Some((generation, Some(260_000u32)));
        assert_eq!(settle_summary(&current, generation, error()), None);
        // 初回の失敗は従来どおり失敗として書く(「一部の集計を取得できませんでした」が出る)
        assert_eq!(
            settle_summary::<u32>(&None, generation, error()),
            Some((generation, None))
        );
        assert_eq!(
            settle_summary(&Some((generation, None::<u32>)), generation, error()),
            Some((generation, None))
        );
        // 別世代の成功値は今の世代の失敗を隠さない
        assert_eq!(
            settle_summary(&Some((Generation::new(0), Some(1u32))), generation, error()),
            Some((generation, None))
        );
        // 成功は常に書く(Ok(None) は「集計なし」の確定)
        assert_eq!(
            settle_summary(&current, generation, Ok(Some(1))),
            Some((generation, Some(1)))
        );
        assert_eq!(
            settle_summary(&current, generation, Ok(None)),
            Some((generation, None))
        );
    }
}
