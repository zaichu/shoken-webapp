use super::super::model;
use super::super::store::{self, HomeOverviewData};
use super::stocks_of;
use crate::features::asset_balance;
use crate::session;
use crate::ui::cumulative_chart::CumulativeChart;
use crate::ui::home::{
    HomeCardError, HomeCardHead, HomeCardSkeleton, HomeChartSection, HomeDetailCard,
    HomeDividendHead,
};
use crate::ui::labels;
use leptos::prelude::*;

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
                let previous_rows = previous.flatten();
                let has_previous = previous_rows.is_some();
                let maps = estimates.as_ref().and_then(|slot| slot.as_ref());
                let stocks = stocks_of(&assets).unwrap_or(&[]);
                let forecast = maps.and_then(|maps| {
                    model::annual_dividend_forecast(stocks, maps).total_annual_dividends
                });
                let forecast_pending = maps.is_some_and(model::has_pending_estimate);
                let now = js_sys::Date::new_0();
                let year = now.get_full_year();
                let model = model::build_dividend_card(
                    &current_rows,
                    previous_rows.as_deref(),
                    forecast,
                );
                view! {
                    <DividendCardBody
                        model=model
                        year=year
                        previous_year=year - 1
                        pending=forecast_pending
                        has_previous=has_previous
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
    has_previous: bool,
) -> impl IntoView {
    let received_value = Some(asset_balance::dec_to_f64(&model.received));
    let received = asset_balance::format_signed_valuation_amount(received_value);
    let forecast = model.forecast_net.map_or_else(
        || labels::MISSING.to_string(),
        |v| asset_balance::format_valuation_amount(Some(v)),
    );
    let progress = model::progress_display(model.progress);
    let current_month = js_sys::Date::new_0().get_month() + 1;
    view! {
        <HomeDividendHead
            received_text=received
            received_negative=asset_balance::is_negative_valuation(received_value)
            received_positive=asset_balance::is_positive_valuation(received_value)
            forecast_text=(!pending).then(|| forecast.clone())
            progress_text=progress.text
        />
        <div class="home-progress">
            <div
                class="home-progress-fill"
                style:width=format!("{}%", progress.bar_pct)
            />
        </div>
        <DividendMonthlyChart
            monthly=model.monthly
            year=year
            previous_year=previous_year
            current_month=current_month
            has_previous=has_previous
        />
    }
}

#[component]
fn DividendMonthlyChart(
    monthly: Vec<model::MonthlyDividends>,
    year: u32,
    previous_year: u32,
    current_month: u32,
    has_previous: bool,
) -> impl IntoView {
    // 前年の取得に失敗したときは前年の線と差を出さない(0 を実績として見せない)
    let mut current = vec![0.0; 12];
    let mut previous_totals = vec![0.0; 12];
    for pair in &monthly {
        if (1..=12).contains(&pair.month) {
            current[(pair.month - 1) as usize] = asset_balance::dec_to_f64(&pair.current);
            previous_totals[(pair.month - 1) as usize] = asset_balance::dec_to_f64(&pair.previous);
        }
    }
    let previous: Option<Vec<f64>> = has_previous.then_some(previous_totals);
    view! {
        <HomeChartSection label=labels::home_cumulative_aria(year, previous_year)>
            <CumulativeChart
                current=current
                previous=previous
                current_year=year
                previous_year=previous_year
                aria_label=labels::home_cumulative_aria(year, previous_year)
                current_month=current_month
            />
        </HomeChartSection>
    }
}
