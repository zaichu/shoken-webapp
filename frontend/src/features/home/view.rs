//! ホームダッシュボードの表示。集計は model、取得は store に寄せ、
//! ここではスロットの値を表示用の部品に展開する。

mod composition_card;
mod dividend_card;
mod pnl_ranking;
mod recent_dividends;
mod summary_band;

use self::composition_card::CompositionCard;
use self::dividend_card::DividendCard;
use self::pnl_ranking::PnlRankingCard;
use self::recent_dividends::RecentDividendsCard;
use self::summary_band::SummaryBand;
use super::model;
use super::store::{self, HomeOverviewData};
use crate::api::dto::{AssetBalance, FundBalance};
use crate::features::asset_balance;
use crate::features::asset_balance::{LoadedAssetBalances, LoadedFundBalances};
use crate::session;
use crate::ui::format;
use crate::ui::home::{
    HomeImportLabel, HomeLogin, HomeMain, HomePartialError, HomeStatusNote, HomeSummarySection,
};
use crate::ui::labels;
use crate::ui::more_link::MoreLink;
use crate::ui::page_frame::PageFrame;
use crate::ui::page_skeleton::SkeletonSummary;
use leptos::prelude::*;

pub(crate) fn stocks_of(assets: &Option<Option<LoadedAssetBalances>>) -> Option<&[AssetBalance]> {
    assets
        .as_ref()
        .and_then(|slot| slot.as_ref())
        .map(|loaded| loaded.rows.as_slice())
}

pub(crate) fn stock_market(assets: &Option<Option<LoadedAssetBalances>>) -> f64 {
    match assets {
        Some(Some(loaded)) => loaded.summary.as_ref().map_or_else(
            || {
                asset_balance::dec_to_f64(
                    &model::stock_summary_from_rows(&loaded.rows).total_market_value,
                )
            },
            |summary| asset_balance::dec_to_f64(&summary.total_market_value),
        ),
        _ => 0.0,
    }
}

pub(crate) fn funds_of(funds: &Option<Option<LoadedFundBalances>>) -> Option<&[FundBalance]> {
    funds
        .as_ref()
        .and_then(|slot| slot.as_ref())
        .map(|loaded| loaded.rows.as_slice())
}

pub(crate) fn fund_market(funds: &Option<Option<LoadedFundBalances>>) -> f64 {
    match funds {
        Some(Some(loaded)) => loaded.summary.as_ref().map_or_else(
            || {
                asset_balance::dec_to_f64(
                    &model::fund_summary_from_rows(&loaded.rows).total_market_value,
                )
            },
            |summary| asset_balance::dec_to_f64(&summary.total_market_value),
        ),
        _ => 0.0,
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    let session = session::use_session();
    let data = store::use_home_overview(session);
    // 再訪では表示済みの値を消さず、取得済みの slot だけ裏で取り直す(未取得は Effect が担う)
    if session.user.get_untracked().is_some() {
        store::refresh_loaded(session, session.generation.get_untracked(), data);
    }
    let import_label = Signal::derive(move || {
        let generation = session.generation.get();
        let mut stamps: Vec<String> = Vec::new();
        if let Some(loaded) = store::current_summary(data.assets, generation).flatten() {
            stamps.extend(loaded.rows.iter().map(|row| row.updated_at.clone()));
        }
        if let Some(loaded) = store::current_summary(data.funds, generation).flatten() {
            stamps.extend(loaded.rows.iter().map(|row| row.updated_at.clone()));
        }
        let label = format::format_latest_import_date(stamps.iter().map(String::as_str));
        let import_iso = model::latest_import_iso(&stamps);
        let price_iso = {
            let assets = store::current_summary(data.assets, generation).flatten();
            let funds = store::current_summary(data.funds, generation).flatten();
            let stocks = assets.map_or_default(|loaded| loaded.rows.clone());
            let funds = funds.map_or_default(|loaded| loaded.rows.clone());
            model::oldest_home_price_iso(&stocks, &funds)
        };
        let stale = match (price_iso, import_iso) {
            (Some(price), Some(import)) => labels::stale_sides(&price, &import).1,
            _ => false,
        };
        label.map(|label| (label, stale))
    });
    let links: Children = Box::new(move || {
        view! {
            {move || {
                import_label.get().map(|(label, stale)| {
                    let text = if stale {
                        format!(
                            "{}{}",
                            labels::home_import_with_role(&label),
                            labels::IMPORT_STALE_MARK
                        )
                    } else {
                        labels::home_import_with_role(&label)
                    };
                    view! {
                        <HomeImportLabel stale=stale>{text}</HomeImportLabel>
                    }
                })
            }}
        }
        .into_any()
    });
    view! {
        <PageFrame title=labels::HOME_TITLE full_bleed=true links=links>
            <HomeDashboard data=data />
        </PageFrame>
    }
}

#[component]
fn HomeDashboard(data: HomeOverviewData) -> impl IntoView {
    let session = session::use_session();
    let snapshot = move || {
        let generation = session.generation.get();
        (
            store::current_summary(data.assets, generation),
            store::current_summary(data.funds, generation),
            store::current_summary(data.dividends_current, generation),
            store::current_summary(data.dividends_previous, generation),
            store::current_summary(data.recent, generation),
            store::current_summary(data.estimates, generation),
            generation,
        )
    };
    view! {
        <Show
            when=move || session.user.get().is_some()
            fallback=move || {
                session.connection_error.get().map_or_else(
                    || view! { <HomeLogin /> }.into_any(),
                    |error| {
                        view! {
                            <crate::ui::state::ListLoadError
                                message=error.user_message()
                                on_retry=move || {
                                    let session = session;
                                    leptos::task::spawn_local(async move { session.check().await; });
                                }
                            />
                        }
                        .into_any()
                    },
                )
            }
        >
            <div class="page-gutter">
                <HomeStatusNote>
                    {move || {
                        let (_, _, _, _, _, _, generation) = snapshot();
                        let any_failed = data.fetch_state.with(|map| {
                            store::refresh_failed(map, generation)
                        });
                        any_failed.then(|| {
                            view! { <HomePartialError /> }
                        })
                    }}
                </HomeStatusNote>
            </div>
            <HomeSummarySection>
                <SkeletonSummary>
                    <SummaryBand data=data />
                </SkeletonSummary>
            </HomeSummarySection>
            <HomeMain>
                <div class="home-card-grid">
                    <div class="home-card-col">
                        <DividendCard data=data />
                        <CompositionCard data=data />
                    </div>
                    <div class="home-card-col">
                        <RecentDividendsCard data=data />
                        <PnlRankingCard data=data />
                    </div>
                </div>
                <div class="home-footlink">
                    <MoreLink
                        href="/csv".to_string()
                        label=labels::HOME_CSV_IMPORT_LINK.to_string()
                        extra_class="more-link-centered"
                    />
                </div>
            </HomeMain>
        </Show>
    }
}

#[cfg(test)]
mod tests;
