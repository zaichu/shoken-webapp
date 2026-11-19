use super::super::model;
use super::super::store::{self, HomeOverviewData};
use super::{funds_of, stocks_of};
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance;
use crate::features::dividend_per_share::DividendMaps;
use crate::session;
use crate::ui::labels;
use crate::ui::summary;
use crate::ui::summary::{
    SummaryBand as UiSummaryBand, SummaryBreakdownEntry, SummarySubItem, SummaryTone,
};
use leptos::prelude::*;

#[component]
pub(super) fn SummaryBand(data: HomeOverviewData) -> impl IntoView {
    let session = session::use_session();
    let state = move || {
        let generation = session.generation.get();
        let assets = store::current_summary(data.assets, generation);
        let funds = store::current_summary(data.funds, generation);
        let estimates = store::current_summary(data.estimates, generation);
        (assets, funds, estimates)
    };
    view! {
        <section
            class="summary-band"
            role="region"
            aria-label=labels::HOME_OVERVIEW_LABEL
            aria-busy=move || {
                let (assets, funds, estimates) = state();
                if assets.is_none() || funds.is_none() || estimates.is_none() {
                    "true"
                } else {
                    "false"
                }
            }
            data-testid="home-overview"
        >
            {move || {
                let (assets, funds, estimates) = state();
                let loading = assets.is_none() || funds.is_none();
                let (merged, daily, as_of, pending, price_stale, import_note, import_stale) = if loading {
                    (
                        None,
                        None,
                        String::new(),
                        false,
                        false,
                        String::new(),
                        false,
                    )
                } else {
                    let stock_summary = assets.as_ref().and_then(|slot| {
                        slot.as_ref().map(|loaded| {
                            loaded.summary.clone().unwrap_or_else(|| {
                                model::stock_summary_from_rows(&loaded.rows)
                            })
                        })
                    });
                    let fund_summary = funds.as_ref().and_then(|slot| {
                        slot.as_ref().map(|loaded| {
                            loaded.summary.clone().unwrap_or_else(|| {
                                model::fund_summary_from_rows(&loaded.rows)
                            })
                        })
                    });
                    let daily = model::merged_daily_change(
                        stock_summary.as_ref(),
                        stocks_of(&assets).is_some_and(|rows| !rows.is_empty()),
                        fund_summary.as_ref(),
                    );
                    let merged = model::merged_asset_summary(
                        stock_summary.as_ref(),
                        fund_summary.as_ref(),
                    );
                    let stock_rows = stocks_of(&assets).unwrap_or(&[]);
                    let fund_rows = funds_of(&funds).unwrap_or(&[]);
                    let pending = model::home_has_pending(
                        stock_rows,
                        fund_rows,
                        stock_summary.as_ref(),
                    );
                    let mut stamps: Vec<String> = stock_rows
                        .iter()
                        .map(|row| row.updated_at.clone())
                        .collect();
                    stamps.extend(fund_rows.iter().map(|row| row.updated_at.clone()));
                    let price_iso = model::oldest_home_price_iso(stock_rows, fund_rows);
                    let import_iso = model::latest_import_iso(&stamps);
                    let (price_stale, import_stale) = match (&price_iso, &import_iso) {
                        (Some(price), Some(import)) => labels::stale_sides(price, import),
                        _ => (false, false),
                    };
                    let mut as_of = model::oldest_home_as_of(stock_rows, fund_rows)
                        .unwrap_or_default();
                    if price_stale && !as_of.is_empty() {
                        as_of.push_str(labels::PRICE_STALE_MARK);
                    }
                    let import_note = import_iso
                        .as_deref()
                        .map_or_default(labels::stale_import_label);
                    (
                        merged,
                        daily,
                        as_of,
                        pending,
                        price_stale,
                        import_note,
                        import_stale,
                    )
                };
                view! {
                    <BandTiles
                        merged=merged
                        daily=daily
                        estimates=estimates
                        stocks=stocks_of(&assets).unwrap_or(&[]).to_vec()
                        loading=loading
                        as_of=as_of
                        show_pending=pending
                        as_of_stale=price_stale
                        import_note=import_note
                        import_stale=import_stale
                    />
                }
            }}
        </section>
    }
}

#[component]
fn BandTiles(
    merged: Option<AssetBalanceSummary>,
    daily: Option<f64>,
    estimates: Option<Option<DividendMaps>>,
    stocks: Vec<crate::api::dto::AssetBalance>,
    loading: bool,
    #[prop(optional, into)] as_of: String,
    #[prop(optional)] show_pending: bool,
    #[prop(optional)] as_of_stale: bool,
    #[prop(optional, into)] import_note: String,
    #[prop(optional)] import_stale: bool,
) -> impl IntoView {
    let market_value = merged
        .as_ref()
        .map(|summary| asset_balance::dec_to_f64(&summary.total_market_value));
    let market_text = if loading {
        labels::MISSING.to_string()
    } else {
        asset_balance::format_valuation_amount(market_value)
    };
    let valuation = merged.as_ref().map(asset_balance::summary_valuation);
    let pnl_amount = valuation
        .as_ref()
        .and_then(|valuation| valuation.profit_loss.amount);
    let pnl_negative = asset_balance::is_negative_valuation(pnl_amount);
    let pnl_positive = asset_balance::is_positive_valuation(pnl_amount);
    let profit_text = if loading {
        labels::MISSING.to_string()
    } else {
        valuation.as_ref().map_or_else(
            || labels::MISSING.to_string(),
            |valuation| asset_balance::format_signed_valuation_amount(valuation.profit_loss.amount),
        )
    };
    let profit_rate_value = valuation
        .as_ref()
        .and_then(|valuation| valuation.profit_loss.rate);
    let profit_rate = profit_rate_value
        .map_or_default(|rate| asset_balance::format_signed_valuation_rate(Some(rate), 1));
    let daily_value = daily;
    let daily_text = if loading {
        labels::MISSING.to_string()
    } else {
        asset_balance::format_signed_row_valuation_amount(daily_value)
    };
    let daily_rate_value = summary::daily_rate(market_value, daily_value);
    let daily_rate_text = if loading {
        String::new()
    } else {
        daily_rate_value
            .map_or_default(|rate| asset_balance::format_signed_valuation_rate(Some(rate), 1))
    };
    let daily_negative = asset_balance::is_negative_valuation(daily_value);
    let daily_positive = asset_balance::is_positive_valuation(daily_value);
    let purchase_value = merged
        .as_ref()
        .map(|summary| asset_balance::dec_to_f64(&summary.total_purchase_amount));
    let purchase_text = if loading {
        labels::MISSING.to_string()
    } else {
        asset_balance::format_valuation_amount(purchase_value)
    };
    let estimates_pending = estimates.is_none();
    let maps = estimates.as_ref().and_then(|slot| slot.as_ref());
    let kpi = maps.map(|maps| model::annual_dividend_forecast(&stocks, maps));
    let dividend_text = if estimates_pending || maps.is_none() {
        labels::MISSING.to_string()
    } else {
        kpi.as_ref()
            .and_then(|kpi| kpi.total_annual_dividends)
            .map_or_else(
                || labels::MISSING.to_string(),
                |amount| asset_balance::format_valuation_amount(Some(amount)),
            )
    };
    let estimate_pending_text = match (maps, kpi.as_ref().and_then(|k| k.dividend_yield)) {
        (Some(maps), None) if model::has_pending_estimate(maps) => {
            labels::HOME_ESTIMATE_PENDING.to_string()
        }
        _ => String::new(),
    };
    view! {
        <UiSummaryBand
            primary_label=labels::MARKET_VALUE
            primary_value=market_text
            primary_as_of=as_of
            show_pending=show_pending
            as_of_stale=as_of_stale
            primary_note2=import_note
            primary_note2_stale=import_stale
            aux_label=labels::VALUATION_PROFIT_LOSS
            aux_value=profit_text
            aux_rate=profit_rate
            aux_rate_negative=profit_rate_value.is_some_and(|rate| asset_balance::is_negative_rate(Some(rate)))
            aux_rate_positive=profit_rate_value.is_some_and(|rate| asset_balance::is_positive_rate(Some(rate)))
            aux_negative=pnl_negative
            aux_positive=pnl_positive
            pc_items=vec![
                SummarySubItem {
                    label: labels::DAY_CHANGE.to_string(),
                    value: daily_text.clone(),
                    note: String::new(),
                    tone: if daily_negative {
                        SummaryTone::Loss
                    } else if daily_positive {
                        SummaryTone::Gain
                    } else {
                        SummaryTone::Neutral
                    },
                    note_negative: false,
                    note_positive: false,
                    extra_class: String::new(),
                },
                SummarySubItem {
                    label: labels::ANNUAL_EXPECTED_DIVIDEND.to_string(),
                    value: dividend_text.clone(),
                    note: String::new(),
                    tone: SummaryTone::Neutral,
                    note_negative: false,
                    note_positive: false,
                    extra_class: String::new(),
                },
                SummarySubItem {
                    label: labels::TOTAL_ACQUISITION.to_string(),
                    value: purchase_text.clone(),
                    note: String::new(),
                    tone: SummaryTone::Neutral,
                    note_negative: false,
                    note_positive: false,
                    extra_class: String::new(),
                },
            ]
            breakdown=vec![
                SummaryBreakdownEntry {
                    label: labels::DAY_CHANGE.to_string(),
                    value: daily_text,
                    negative: daily_negative,
                    positive: daily_positive,
                    rate: daily_rate_text,
                    rate_negative: daily_rate_value.is_some_and(|rate| asset_balance::is_negative_rate(Some(rate))),
                    rate_positive: daily_rate_value.is_some_and(|rate| asset_balance::is_positive_rate(Some(rate))),
                },
                SummaryBreakdownEntry {
                    label: labels::ANNUAL_EXPECTED_DIVIDEND.to_string(),
                    value: dividend_text,
                    negative: false,
                    positive: false,
                    rate: estimate_pending_text,
                    rate_negative: false,
                    rate_positive: false,
                },
                SummaryBreakdownEntry {
                    label: labels::TOTAL_ACQUISITION.to_string(),
                    value: purchase_text,
                    negative: false,
                    positive: false,
                    rate: String::new(),
                    rate_negative: false,
                    rate_positive: false,
                },
            ]
        />
    }
}
