//! ホームダッシュボードのデータ取得。世代つきスロットに保持し、
//! ページ遷移でビューを作り直しても取得済みの値を失わない。

use crate::api::dto::{AssetBalance, Dividend, DividendListResponse, FundBalance};
use crate::api::{ApiClient, ApiError};
use crate::features::asset_balance::{LoadedAssetBalances, LoadedFundBalances};
use crate::features::dividend_per_share::{
    DIVIDEND_NETWORK_MAX_RETRIES, DIVIDEND_RETRY_DELAY_MS, DividendMaps, dividend_maps_from_batch,
    dividend_pending_max_retries, fetch_dividend_batch, post_dividend_batch, unique_sorted_codes,
};
use crate::session::{Generation, SessionStore};
use crate::support::pagination::{
    LIST_MAX_PAGES, LIST_PER_PAGE, ListEndpoint, collect_list_pages, fetch_all_pages,
};
use leptos::prelude::*;
use std::collections::HashMap;
use std::future::Future;

// 外側の None は取得中、内側の None は取得失敗
pub(crate) type SummarySlot<T> = RwSignal<Option<(Generation, Option<T>)>>;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SummaryKind {
    Assets,
    Funds,
    DividendsCurrent,
    DividendsPrevious,
    Recent,
    Estimates,
}

impl SummaryKind {
    /// 部分失敗の案内に数える取得
    pub(crate) const ALL: &'static [SummaryKind] = &[
        SummaryKind::Assets,
        SummaryKind::Funds,
        SummaryKind::DividendsCurrent,
        SummaryKind::DividendsPrevious,
        SummaryKind::Recent,
        SummaryKind::Estimates,
    ];
}

#[derive(Default)]
pub(crate) struct SummaryFetch {
    rev: u64,
    inflight: bool,
    // 裏再取得が失敗したときだけ立て、表示済みの集計は残したまま失敗を伝える
    failed: bool,
}

// 往復で要求を重ねないよう取得中は None を返す。返した rev で応答の新しさを判定する
pub(crate) fn begin_summary_fetch(
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
pub(crate) fn finish_summary_fetch(
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
pub(crate) fn settle_summary<T>(
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

pub(crate) fn current_summary<T: Clone + Send + Sync + 'static>(
    slot: SummarySlot<T>,
    generation: Generation,
) -> Option<Option<T>> {
    slot.get()
        .filter(|(cached, _)| *cached == generation)
        .map(|(_, summary)| summary)
}

pub(crate) fn has_current_summary<T>(
    slot: &Option<(Generation, Option<T>)>,
    generation: Generation,
) -> bool {
    matches!(slot, Some((cached, _)) if *cached == generation)
}

pub(crate) fn refresh_failed(
    map: &HashMap<(Generation, SummaryKind), SummaryFetch>,
    generation: Generation,
) -> bool {
    SummaryKind::ALL.iter().any(|kind| {
        map.get(&(generation, *kind))
            .is_some_and(|entry| entry.failed)
    })
}

async fn fetch_stock_rows() -> Result<Option<LoadedAssetBalances>, ApiError> {
    let page = fetch_all_pages::<AssetBalance>().await?;
    Ok(Some(LoadedAssetBalances {
        total: page.total.unwrap_or(page.rows.len()),
        rows: page.rows,
        summary: page.summary,
        truncated: page.truncated,
    }))
}

async fn fetch_fund_rows() -> Result<Option<LoadedFundBalances>, ApiError> {
    let page = fetch_all_pages::<FundBalance>().await?;
    Ok(Some(LoadedFundBalances {
        total: page.total.unwrap_or(page.rows.len()),
        rows: page.rows,
        summary: page.summary,
        truncated: page.truncated,
    }))
}

/// 決済日が `year` の配当を全ページ取る。月別・前年同期・実効税率の材料にする
async fn fetch_dividend_year(year: u32) -> Result<Option<Vec<Dividend>>, ApiError> {
    let client = ApiClient::read_client();
    let year = year.to_string();
    let page = collect_list_pages(LIST_PER_PAGE, LIST_MAX_PAGES, |page_no| {
        let client = client.clone();
        let year = year.clone();
        async move {
            let page = page_no.to_string();
            let per_page = LIST_PER_PAGE.to_string();
            let include_summary = if page_no == 1 { "true" } else { "false" };
            let query = vec![
                ("page", page.as_str()),
                ("per_page", per_page.as_str()),
                ("include_summary", include_summary),
                ("year", year.as_str()),
            ];
            client
                .get_json::<DividendListResponse>(<Dividend as ListEndpoint>::PATH, &query)
                .await
        }
    })
    .await?;
    Ok(Some(page.rows))
}

/// 直近の配当入金。API は決済日降順で返すため先頭 `count` 件が新しい
async fn fetch_recent_dividends(count: usize) -> Result<Option<Vec<Dividend>>, ApiError> {
    let count = count.to_string();
    ApiClient::read_client()
        .get_json::<DividendListResponse>(
            <Dividend as ListEndpoint>::PATH,
            &[("per_page", count.as_str())],
        )
        .await
        .map(|response| Some(response.data))
}

#[derive(Clone, Copy)]
pub(crate) struct HomeOverviewData {
    pub(crate) assets: SummarySlot<LoadedAssetBalances>,
    pub(crate) funds: SummarySlot<LoadedFundBalances>,
    pub(crate) dividends_current: SummarySlot<Vec<Dividend>>,
    pub(crate) dividends_previous: SummarySlot<Vec<Dividend>>,
    pub(crate) recent: SummarySlot<Vec<Dividend>>,
    pub(crate) estimates: SummarySlot<DividendMaps>,
    pub(crate) fetch_state: RwSignal<HashMap<(Generation, SummaryKind), SummaryFetch>>,
}

impl HomeOverviewData {
    fn clear(&self) {
        self.assets.set(None);
        self.funds.set(None);
        self.dividends_current.set(None);
        self.dividends_previous.set(None);
        self.recent.set(None);
        self.estimates.set(None);
    }
}

// ページ遷移でビューを作り直しても取得済みの集計を失わないよう、App コンテキストのスロットに保持する
pub(crate) fn use_home_overview(session: SessionStore) -> HomeOverviewData {
    let states = use_context::<crate::app::PageStates>().unwrap_or_default();
    crate::app::page_state(states.home_overview, || build_home_overview(session))
}

fn build_home_overview(session: SessionStore) -> HomeOverviewData {
    let data = HomeOverviewData {
        assets: RwSignal::new(None),
        funds: RwSignal::new(None),
        dividends_current: RwSignal::new(None),
        dividends_previous: RwSignal::new(None),
        recent: RwSignal::new(None),
        estimates: RwSignal::new(None),
        fetch_state: RwSignal::new(HashMap::new()),
    };
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            data.clear();
            // 前ユーザーの取得中エントリが残ると次ユーザーの同種取得が欠かされる
            data.fetch_state
                .update(|map| map.retain(|(cached, _), _| *cached == generation));
            return;
        }
        load_home_data(session, generation, data);
    });
    // 見積もりは保有株式の銘柄コードが揃ってから取るため、資産行の着地を別 Effect で追う
    Effect::new(move |_| {
        let generation = session.generation.get();
        if session.user.get().is_none() {
            return;
        }
        let settled = data.assets.with(|slot| match slot {
            Some((cached, loaded)) if *cached == generation => Some(loaded.is_some()),
            _ => None,
        });
        match settled {
            // 資産行の着地待ち
            None => return,
            // 資産の取得失敗。見積もりは取れないので失敗側で確定させる
            Some(false) => {
                data.estimates.set(Some((generation, None)));
                return;
            }
            Some(true) => {}
        }
        let codes = data.assets.with(|slot| match slot {
            Some((cached, Some(loaded))) if *cached == generation => unique_sorted_codes(
                &loaded
                    .rows
                    .iter()
                    .map(|row| row.security_code.to_string())
                    .collect::<Vec<_>>(),
            ),
            _ => Vec::new(),
        });
        if codes.is_empty() {
            // 株式を持たない(または0件着地)ユーザーは見積もりを取らない。
            // 取得中のままだと帯がずっと busy になるため「空の見積もり」で確定させる
            data.estimates
                .set(Some((generation, Some(DividendMaps::default()))));
            return;
        }
        load_estimates(session, generation, codes, data);
    });
    data
}

fn load_home_data(session: SessionStore, generation: Generation, data: HomeOverviewData) {
    let year = js_sys::Date::new_0().get_full_year();
    load_summary(
        session,
        generation,
        SummaryKind::Assets,
        data.assets,
        data.fetch_state,
        fetch_stock_rows(),
    );
    load_summary(
        session,
        generation,
        SummaryKind::Funds,
        data.funds,
        data.fetch_state,
        fetch_fund_rows(),
    );
    load_summary(
        session,
        generation,
        SummaryKind::DividendsCurrent,
        data.dividends_current,
        data.fetch_state,
        fetch_dividend_year(year),
    );
    load_summary(
        session,
        generation,
        SummaryKind::DividendsPrevious,
        data.dividends_previous,
        data.fetch_state,
        fetch_dividend_year(year - 1),
    );
    load_summary(
        session,
        generation,
        SummaryKind::Recent,
        data.recent,
        data.fetch_state,
        fetch_recent_dividends(5),
    );
}

// 見積もりの pending はバックエンドの集計完了を待つため、応答を書きながら間隔を置いて取り直す
fn load_estimates(
    session: SessionStore,
    generation: Generation,
    codes: Vec<String>,
    data: HomeOverviewData,
) {
    let Some(rev) = data
        .fetch_state
        .try_update(|map| begin_summary_fetch(map, generation, SummaryKind::Estimates))
        .flatten()
    else {
        return;
    };
    let estimates = data.estimates;
    let fetch_state = data.fetch_state;
    leptos::task::spawn_local(async move {
        let max_pending = dividend_pending_max_retries(codes.len());
        let mut pending_used = 0u32;
        let mut network_used = 0u32;
        let mut received_any = false;
        loop {
            match fetch_dividend_batch(&codes, post_dividend_batch).await {
                Ok(batch) => {
                    if !session.is_current(generation) {
                        return;
                    }
                    let (maps, has_pending) = dividend_maps_from_batch(&batch);
                    received_any = true;
                    estimates.set(Some((generation, Some(maps))));
                    if !has_pending || pending_used >= max_pending {
                        break;
                    }
                    pending_used += 1;
                }
                Err(error) => {
                    if error.is_unauthorized() {
                        session.mark_unauthenticated();
                        return;
                    }
                    if network_used >= DIVIDEND_NETWORK_MAX_RETRIES {
                        break;
                    }
                    network_used += 1;
                }
            }
            gloo_timers::future::TimeoutFuture::new(DIVIDEND_RETRY_DELAY_MS).await;
        }
        let is_current = fetch_state
            .try_update(|map| finish_summary_fetch(map, generation, SummaryKind::Estimates, rev))
            .unwrap_or_default();
        if !session.is_current(generation) || !is_current {
            return;
        }
        if !received_any {
            estimates.set(Some((generation, None)));
            fetch_state.update(|map| {
                if let Some(entry) = map.get_mut(&(generation, SummaryKind::Estimates)) {
                    entry.failed = true;
                }
            });
        }
    });
}

/// 再訪時に表示済みのスロットだけ裏で取り直す(未取得は Effect が担う)
pub(crate) fn refresh_loaded(
    session: SessionStore,
    generation: Generation,
    data: HomeOverviewData,
) {
    let year = js_sys::Date::new_0().get_full_year();
    if has_current_summary(&data.assets.get_untracked(), generation) {
        load_summary(
            session,
            generation,
            SummaryKind::Assets,
            data.assets,
            data.fetch_state,
            fetch_stock_rows(),
        );
    }
    if has_current_summary(&data.funds.get_untracked(), generation) {
        load_summary(
            session,
            generation,
            SummaryKind::Funds,
            data.funds,
            data.fetch_state,
            fetch_fund_rows(),
        );
    }
    if has_current_summary(&data.dividends_current.get_untracked(), generation) {
        load_summary(
            session,
            generation,
            SummaryKind::DividendsCurrent,
            data.dividends_current,
            data.fetch_state,
            fetch_dividend_year(year),
        );
    }
    if has_current_summary(&data.dividends_previous.get_untracked(), generation) {
        load_summary(
            session,
            generation,
            SummaryKind::DividendsPrevious,
            data.dividends_previous,
            data.fetch_state,
            fetch_dividend_year(year - 1),
        );
    }
    if has_current_summary(&data.recent.get_untracked(), generation) {
        load_summary(
            session,
            generation,
            SummaryKind::Recent,
            data.recent,
            data.fetch_state,
            fetch_recent_dividends(5),
        );
    }
    // 見積もりの再取得は資産行の更新に連動する Effect に任せる
}

#[cfg(test)]
mod tests;
