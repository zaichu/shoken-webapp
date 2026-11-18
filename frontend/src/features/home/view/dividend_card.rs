use super::super::model;
use super::super::store::{self, HomeOverviewData};
use super::stocks_of;
use crate::features::asset_balance;
use crate::session;
use crate::ui::amount::Amount;
use crate::ui::format;
use crate::ui::home::{
    HomeCardError, HomeCardHead, HomeCardSkeleton, HomeChartLegend, HomeChartSection,
    HomeDetailCard, HomeDividendHead, HomeDividendNote, HomeDividendTooltip,
};
use crate::ui::labels;
use leptos::prelude::*;
use rust_decimal::Decimal;

#[component]
pub(super) fn DividendCard(data: HomeOverviewData) -> impl IntoView {
    let session = session::use_session();
    let card = move || {
        let generation = session.generation.get();
        let current = store::current_summary(data.dividends_current, generation);
        let previous = store::current_summary(data.dividends_previous, generation);
        let assets = store::current_summary(data.assets, generation);
        let estimates = store::current_summary(data.estimates, generation);
        (current, previous, assets, estimates)
    };
    view! {
        <HomeDetailCard testid="home-dividend-card">
            <HomeCardHead
                title=labels::HOME_THIS_YEAR_DIVIDEND
                href="/receipts"
                link_label=labels::HOME_RECEIPTS_VIEW_LINK
            />
            {move || {
                let (current, previous, assets, estimates) = card();
                if current.is_none() || previous.is_none() {
                    return view! { <HomeCardSkeleton /> }.into_any();
                }
                if matches!(current, Some(None)) {
                    return view! { <HomeCardError message=labels::HOME_DIVIDEND_FETCH_FAILED.to_string() /> }
                        .into_any();
                }
                let current_rows = current.flatten().unwrap_or_default();
                // 前年側の失敗はグラフの前年系列を 0 に倒して今年分は見せる。
                // 前年比較の数値は model 側で None になり「—」表示になる
                let previous_rows = previous.flatten();
                let maps = estimates.as_ref().and_then(|slot| slot.as_ref());
                let stocks = stocks_of(&assets).unwrap_or(&[]);
                let forecast = maps.and_then(|maps| {
                    model::annual_dividend_forecast(stocks, maps).total_annual_dividends
                });
                let forecast_pending = maps.is_some_and(model::has_pending_estimate);
                let now = js_sys::Date::new_0();
                let current_month = now.get_month() + 1;
                let year = now.get_full_year();
                let model = model::build_dividend_card(
                    &current_rows,
                    previous_rows.as_deref(),
                    forecast,
                    current_month,
                );
                view! {
                    <DividendCardBody
                        model=model
                        year=year
                        previous_year=year - 1
                        pending=forecast_pending
                    />
                }
                    .into_any()
            }}
        </HomeDetailCard>
    }
}

#[component]
fn DividendCardBody(
    model: model::DividendCardModel,
    year: u32,
    previous_year: u32,
    pending: bool,
) -> impl IntoView {
    let received_value = Some(asset_balance::dec_to_f64(&model.received));
    let received = asset_balance::format_signed_valuation_amount(received_value);
    let forecast = model.forecast_net.map_or_else(
        || labels::MISSING.to_string(),
        |v| asset_balance::format_valuation_amount(Some(v)),
    );
    let progress_pct = model.progress.map(|p| p * 100.0);
    let progress_text = progress_pct.map_or_else(
        || labels::MISSING.to_string(),
        format::format_integer_percent,
    );
    let (remaining_prefix, remaining_amount, remaining_suffix) =
        model::remaining_label(model.remaining);
    let current_month = js_sys::Date::new_0().get_month() + 1;
    let previous_span = labels::previous_dividend_span(current_month);
    let previous_remaining = model
        .previous_remaining
        .filter(|_| !previous_span.is_empty())
        .map(|remaining| {
            asset_balance::format_valuation_amount(Some(asset_balance::dec_to_f64(&remaining)))
        });
    let yoy_value = model.yoy_diff.map(|diff| asset_balance::dec_to_f64(&diff));
    let yoy_text = yoy_value.map_or_else(
        || labels::MISSING.to_string(),
        |value| asset_balance::format_signed_valuation_amount(Some(value)),
    );
    view! {
        <HomeDividendHead
            received_text=received
            received_negative=asset_balance::is_negative_valuation(received_value)
            received_positive=asset_balance::is_positive_valuation(received_value)
            forecast_text=(!pending).then(|| forecast.clone())
            progress_text=progress_text
        />
        <div class="home-progress">
            {progress_pct.map(|pct| {
                view! {
                    <div
                        class="home-progress-fill"
                        style:width=format!("{}%", pct.clamp(0.0, 100.0))
                    />
                }
            })}
        </div>
        <HomeDividendNote
            prefix=remaining_prefix
            remaining=remaining_amount
            suffix=remaining_suffix
            previous=previous_remaining.map(|amount| (previous_span.clone(), amount))
            yoy_text=yoy_text
            yoy_negative=asset_balance::is_negative_valuation(yoy_value)
            yoy_positive=asset_balance::is_positive_valuation(yoy_value)
        />
        <DividendMonthlyChart
            monthly=model.monthly
            year=year
            previous_year=previous_year
            current_month=current_month
        />
    }
}

#[component]
fn DividendMonthlyChart(
    monthly: Vec<model::MonthlyDividends>,
    year: u32,
    previous_year: u32,
    current_month: u32,
) -> impl IntoView {
    let max = monthly
        .iter()
        .flat_map(|pair| [pair.current, pair.previous])
        .max()
        .unwrap_or_default();
    let months: Vec<u32> = monthly.iter().map(|pair| pair.month).collect();
    let cells = monthly
        .into_iter()
        .map(|pair| {
            view! {
                <DividendMonthCell
                    pair=pair
                    max=max
                    year=year
                    previous_year=previous_year
                    current_month=current_month
                />
            }
        })
        .collect_view();
    view! {
        <HomeChartSection label=labels::home_monthly_aria(year, previous_year)>
            <p class="home-chart-max"><Amount text=asset_balance::format_valuation_amount(Some(asset_balance::dec_to_f64(&max))) /></p>
            <div class="home-chart-plot">
                <div class="home-chart-grid" aria-hidden="true">
                    <span class="home-chart-grid-line" data-level="top"></span>
                    <span class="home-chart-grid-line" data-level="mid"></span>
                    <span class="home-chart-grid-line" data-level="base"></span>
                </div>
                <div class="home-chart-months">{cells}</div>
            </div>
            <div class="home-chart-month-labels" aria-hidden="true">
                {months
                    .iter()
                    .map(|month| {
                        let current = *month == current_month;
                        view! {
                            <span
                                class="home-chart-month-label"
                                data-current=current.to_string()
                            >
                                {labels::receipt_month_label(*month)}
                            </span>
                        }
                    })
                    .collect_view()}
            </div>
        </HomeChartSection>
        <HomeChartLegend year=year previous_year=previous_year />
    }
}

#[component]
fn DividendMonthCell(
    pair: model::MonthlyDividends,
    max: Decimal,
    year: u32,
    previous_year: u32,
    current_month: u32,
) -> impl IntoView {
    let cur = model::bar_height(pair.current, max);
    let prev = model::bar_height(pair.previous, max);
    let cur_text = format::format_currency(asset_balance::dec_to_f64(&pair.current));
    let prev_text = format::format_currency(asset_balance::dec_to_f64(&pair.previous));
    let diff_value = asset_balance::dec_to_f64(&(pair.current - pair.previous));
    let diff_text = asset_balance::format_signed_valuation_amount(Some(diff_value));
    let current_note = if pair.month == current_month {
        labels::HOME_CURRENT_MONTH
    } else {
        ""
    };
    let month_aria = labels::home_month_aria(
        pair.month,
        current_note,
        year,
        &cur_text,
        previous_year,
        &prev_text,
        &diff_text,
    );
    view! {
        <div
            class="home-chart-month"
            tabindex="0"
            role="group"
            aria-label=month_aria
        >
            <div class="home-chart-bars">
                <span
                    class="home-month-bar home-chart-previous"
                    style:height=format!("{prev}%")
                />
                <span
                    class="home-month-bar bg-chart-1"
                    style:height=format!("{cur}%")
                />
            </div>
            <div class="home-chart-tip" aria-hidden="true">
                <p class="font-bold text-text">
                    {labels::receipt_month_label(pair.month)}
                </p>
                <HomeDividendTooltip
                    year=year
                    previous_year=previous_year
                    cur_text=cur_text
                    prev_text=prev_text
                    diff_text=diff_text
                    diff_negative=asset_balance::is_negative_valuation(Some(diff_value))
                    diff_positive=asset_balance::is_positive_valuation(Some(diff_value))
                />
            </div>
        </div>
    }
}
