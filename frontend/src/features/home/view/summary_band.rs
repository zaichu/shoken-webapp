use super::super::model;
use super::super::store::{self, HomeOverviewData};
use super::{funds_of, stocks_of};
use crate::api::dto::AssetBalanceSummary;
use crate::features::asset_balance::{
    dec_to_f64, format_signed_valuation_amount, format_signed_valuation_rate,
    format_valuation_amount, is_negative_valuation, is_positive_valuation, summary_valuation,
};
use crate::features::dividend_per_share::DividendMaps;
use crate::session::use_session;
use crate::ui::labels;
use crate::ui::summary::{
    SummaryBand as UiSummaryBand, SummaryStat, SummaryTone, with_pending_note,
};
use leptos::prelude::*;

#[component]
pub(super) fn SummaryBand(
    data: HomeOverviewData,
    import_label: Signal<Option<String>>,
) -> impl IntoView {
    let session = use_session();
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
                let (merged, as_of, pending) = if loading {
                    (None, String::new(), false)
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
                    let merged = model::merged_asset_summary(
                        stock_summary.as_ref(),
                        fund_summary.as_ref(),
                    );
                    let stock_rows = stocks_of(&assets).unwrap_or(&[]);
                    let fund_rows = funds_of(&funds).unwrap_or(&[]);
                    let as_of = model::oldest_home_as_of(stock_rows, fund_rows)
                        .unwrap_or_default();
                    let pending = model::home_has_pending(
                        stock_rows,
                        fund_rows,
                        stock_summary.as_ref(),
                    );
                    (merged, as_of, pending)
                };
                view! {
                    <BandTiles
                        merged=merged
                        estimates=estimates
                        stocks=stocks_of(&assets).unwrap_or(&[]).to_vec()
                        loading=loading
                        as_of=as_of
                        show_pending=pending
                        import_label=import_label
                    />
                }
            }}
        </section>
    }
}

#[component]
fn BandTiles(
    merged: Option<AssetBalanceSummary>,
    estimates: Option<Option<DividendMaps>>,
    stocks: Vec<crate::api::dto::AssetBalance>,
    loading: bool,
    #[prop(optional, into)] as_of: String,
    #[prop(optional)] show_pending: bool,
    import_label: Signal<Option<String>>,
) -> impl IntoView {
    let market_value = merged
        .as_ref()
        .map(|summary| dec_to_f64(&summary.total_market_value));
    let market_text = if loading {
        labels::MISSING.to_string()
    } else {
        format_valuation_amount(market_value)
    };
    let valuation = merged.as_ref().map(summary_valuation);
    let pnl_amount = valuation
        .as_ref()
        .and_then(|valuation| valuation.profit_loss.amount);
    let pnl_negative = is_negative_valuation(pnl_amount);
    let pnl_positive = is_positive_valuation(pnl_amount);
    let profit_text = if loading {
        labels::MISSING.to_string()
    } else {
        valuation.as_ref().map_or_else(
            || labels::MISSING.to_string(),
            |valuation| format_signed_valuation_amount(valuation.profit_loss.amount),
        )
    };
    let profit_rate = valuation
        .as_ref()
        .and_then(|valuation| valuation.profit_loss.rate)
        .map_or_default(|rate| format_signed_valuation_rate(Some(rate), 1));
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
                |amount| format_valuation_amount(Some(amount)),
            )
    };
    let dates_note = with_pending_note(&as_of, show_pending);
    let import_text = import_label
        .get()
        .map_or_else(String::new, |label| labels::home_import_label(&label));
    let mut pc_dates = vec![dates_note.clone()];
    if !import_text.is_empty() {
        pc_dates.push(import_text);
    }
    let pc_dates: Vec<String> = pc_dates
        .into_iter()
        .filter(|line| !line.is_empty())
        .collect();
    let pnl_tone = if pnl_negative {
        SummaryTone::Loss
    } else if pnl_positive {
        SummaryTone::Gain
    } else {
        SummaryTone::Neutral
    };
    view! {
        <UiSummaryBand
            primary=SummaryStat {
                label: labels::MARKET_VALUE.to_string(),
                value: market_text,
                note: String::new(),
                tone: SummaryTone::Neutral,
            }
            secondary=SummaryStat {
                label: labels::VALUATION_PROFIT_LOSS.to_string(),
                value: profit_text,
                note: profit_rate,
                tone: pnl_tone,
            }
            pc_items=vec![
                SummaryStat {
                    label: labels::ANNUAL_EXPECTED_DIVIDEND.to_string(),
                    value: dividend_text,
                    note: String::new(),
                    tone: SummaryTone::Neutral,
                },
            ]
            mobile_dates=dates_note
            pc_dates=pc_dates
        />
    }
}
