use crate::api::dto::{
    AssetBalance, AssetBalanceListResponse, AssetBalanceSummary, Dividend, DividendListResponse,
    DividendSummary,
};
use crate::api::{ApiClient, ApiError};
use crate::features::asset_balance::{
    calculate_valuation_from_decimal, format_valuation_amount, format_valuation_rate,
    valuation_tone,
};
use crate::session::{Generation, SessionStore, use_session};
use crate::support::pagination::ListEndpoint;
use crate::ui::page_frame::PageFrame;
use crate::ui::summary_strip::{SummaryItem, SummaryStrip};
use leptos::prelude::*;
use shared::format::format_currency as format_currency_decimal;
use std::collections::HashMap;
use std::future::Future;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <PageFrame title="ホーム">
            <HomeOverview />
        </PageFrame>
    }
}

// 外側の None は取得中、内側の None は取得失敗
type SummarySlot<T> = RwSignal<Option<(Generation, Option<T>)>>;

async fn fetch_asset_summary() -> Result<Option<AssetBalanceSummary>, ApiError> {
    ApiClient::read_client()
        .get_json::<AssetBalanceListResponse>(
            <AssetBalance as ListEndpoint>::PATH,
            &[("per_page", "1"), ("include_summary", "true")],
        )
        .await
        .map(|response| response.summary)
}

async fn fetch_dividend_summary(year: u32) -> Result<Option<DividendSummary>, ApiError> {
    let year = year.to_string();
    ApiClient::read_client()
        .get_json::<DividendListResponse>(
            <Dividend as ListEndpoint>::PATH,
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
pub(crate) struct HomeOverviewData {
    asset: SummarySlot<AssetBalanceSummary>,
    dividend: SummarySlot<DividendSummary>,
    fetch_state: RwSignal<HashMap<(Generation, SummaryKind), SummaryFetch>>,
}

// ページ遷移でビューを作り直しても取得済みの集計を失わないよう、App コンテキストのスロットに保持する
fn use_home_overview(session: SessionStore) -> HomeOverviewData {
    let states = use_context::<crate::app::PageStates>().unwrap_or_default();
    crate::app::page_state(states.home_overview, || build_home_overview(session))
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
    let (_profit_class, profit_negative) =
        valuation_tone(valuation.as_ref().and_then(|valuation| valuation.amount));
    let profit_rate = valuation.as_ref().map(|valuation| match valuation.rate {
        Some(rate) => format_valuation_rate(Some(rate), 1),
        None => "算出不可".to_string(),
    });
    let profit = valuation.map(|valuation| format_valuation_amount(valuation.amount));
    let dividend = dividend
        .flatten()
        .map(|summary| format_currency_decimal(summary.total_net_amount_received));
    let items = vec![
        SummaryItem::new(
            "評価額",
            (!asset_busy).then(|| market.unwrap_or_else(|| "—".to_string())),
        ),
        SummaryItem {
            label: "評価損益",
            value: (!asset_busy).then(|| profit.unwrap_or_else(|| "—".to_string())),
            note: (!asset_busy).then(|| profit_rate.unwrap_or_else(|| "—".to_string())),
            negative: profit_negative.is_some(),
        },
        SummaryItem::new(
            "今年の税引後配当金",
            (!dividend_busy).then(|| dividend.unwrap_or_else(|| "—".to_string())),
        ),
    ];
    view! { <SummaryStrip items=items /> }
}

#[cfg(test)]
mod tests;
