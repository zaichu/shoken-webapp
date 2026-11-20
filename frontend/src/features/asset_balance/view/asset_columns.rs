use super::sort_header::{sort_header, sort_header_titled};
use crate::features::asset_balance::format as balance_format;
use crate::features::asset_balance::fund::{FundSortKey, FundView};
use crate::features::asset_balance::holdings;
use crate::features::asset_balance::holdings::{HoldingDividend, HoldingView};
use crate::features::asset_balance::model;
use crate::features::asset_balance::model::ValuationInput;
use crate::features::asset_balance::portfolio::{SortKey, SortOrder};
use crate::features::dividend_per_share::DividendMaps;
use crate::ui::amount::{Amount, AmountKind};
use crate::ui::asset_balance::DayChangeNote;
use crate::ui::detail_dialog::DetailCell;
use crate::ui::format;
use crate::ui::labels;
use crate::urls;
use leptos::prelude::*;

pub(crate) fn stock_table_decimals(
    views: &[HoldingView],
    dividends: &DividendMaps,
) -> (u32, u32, u32) {
    let average_price_decimals =
        format::unit_price_decimals(views.iter().map(|view| view.average_price));
    let current_price_decimals =
        format::unit_price_decimals(views.iter().map(|view| view.current_price));
    let dividend_decimals = format::unit_price_decimals(views.iter().filter_map(|view| {
        holdings::holding_dividend(&view.code, view.shares, view.average_price, dividends).per_share
    }));
    (
        average_price_decimals,
        current_price_decimals,
        dividend_decimals,
    )
}

pub(crate) fn stock_date_outliers(
    views: &[HoldingView],
    price_date_rep: Option<&str>,
) -> Vec<bool> {
    balance_format::price_date_outliers(
        &views
            .iter()
            .map(|view| view.price_as_of.clone())
            .collect::<Vec<_>>(),
        price_date_rep,
    )
}

pub(crate) fn mark_stock_date_outliers(rows: &mut [UnifiedRow], rep: Option<&str>) {
    for row in rows {
        row.date_outlier = balance_format::price_date_is_outlier(row.price_as_of.as_deref(), rep);
    }
}

pub(crate) fn fund_base_price_decimals(views: &[FundView]) -> u32 {
    format::unit_price_decimals(views.iter().map(|view| view.base_price))
}

pub(crate) fn merged_dividend_decimals(rows: &[UnifiedRow]) -> u32 {
    format::unit_price_decimals(
        rows.iter()
            .filter_map(|row| row.dividend.as_ref()?.per_share),
    )
}

pub(crate) fn stock_table_head(
    sort_key: RwSignal<SortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    view! {
        <tr>
            <th scope="col" class="col-text">{labels::CODE_COLUMN}</th>
            <th scope="col" class="col-text">{labels::NAME_COLUMN}</th>
            <th scope="col" class="col-text">{labels::ACCOUNT}</th>
            <th scope="col" class="col-num">{labels::QUANTITY}</th>
            <th scope="col" class="col-num hidden xl:table-cell print:table-cell">{labels::AVERAGE_PRICE}</th>
            <th scope="col" class="col-num">{labels::CURRENT_PRICE}</th>
            <th scope="col" class="col-num">
                {labels::DAY_CHANGE}
                <br />
                {labels::PER_SHARE_UNIT}
            </th>
            {sort_header(labels::MARKET_VALUE, SortKey::MarketValue, sort_key, sort_order)}
            {sort_header(
                labels::VALUATION_PROFIT_LOSS,
                SortKey::ProfitLoss,
                sort_key,
                sort_order,
            )}
            <th scope="col" class="col-num hidden xl:table-cell print:table-cell">{labels::PER_SHARE_DIVIDEND}</th>
            {sort_header_titled(
                labels::ANNUAL_DIVIDEND_HEAD,
                None,
                Some(labels::ANNUAL_DIVIDEND_NOTE),
                SortKey::Dividend,
                sort_key,
                sort_order,
            )}
            {sort_header_titled(
                labels::ACQUISITION_YIELD,
                None,
                Some(labels::ACQUISITION_YIELD_NOTE),
                SortKey::Yield,
                sort_key,
                sort_order,
            )}
        </tr>
    }
}

pub(crate) fn fund_table_head(
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    view! {
        <tr>
            <th scope="col" class="col-text">{labels::FUND}</th>
            <th scope="col" class="col-text">{labels::ACCOUNT}</th>
            <th scope="col" class="col-text">{labels::DISTRIBUTION_COURSE}</th>
            {sort_header(labels::FUND_SHARES, FundSortKey::Shares, sort_key, sort_order)}
            {sort_header(labels::BASE_PRICE, FundSortKey::BasePrice, sort_key, sort_order)}
            <th
                scope="col"
                class="col-num"
                title=labels::DAY_CHANGE_COLUMN_NOTE
                abbr=labels::DAY_CHANGE_COLUMN_NOTE
            >
                {labels::DAY_CHANGE}
                <br />
                {labels::PER_10K_UNIT}
            </th>
            {sort_header(labels::MARKET_VALUE, FundSortKey::MarketValue, sort_key, sort_order)}
            {sort_header(
                labels::VALUATION_PROFIT_LOSS,
                FundSortKey::ProfitLoss,
                sort_key,
                sort_order,
            )}
        </tr>
    }
}

pub(crate) fn merged_table_head(
    sort_key: RwSignal<FundSortKey>,
    sort_order: RwSignal<SortOrder>,
) -> impl IntoView {
    view! {
        <tr>
            <th scope="col" class="col-text">{labels::CODE_COLUMN}</th>
            <th scope="col" class="col-text">{labels::SECURITY_AND_FUND}</th>
            <th scope="col" class="col-text">{labels::KIND}</th>
            <th scope="col" class="col-text">{labels::ACCOUNT}</th>
            <th scope="col" class="col-text">{labels::DISTRIBUTION_COURSE}</th>
            <th scope="col" class="col-num">{labels::PER_SHARE_DIVIDEND}</th>
            <th scope="col" class="col-num">{labels::DAY_CHANGE_RATE}</th>
            {sort_header(labels::MARKET_VALUE, FundSortKey::MarketValue, sort_key, sort_order)}
            {sort_header(
                labels::VALUATION_PROFIT_LOSS,
                FundSortKey::ProfitLoss,
                sort_key,
                sort_order,
            )}
        </tr>
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UnifiedRow {
    pub name: String,
    pub kind: &'static str,
    pub sub: String,
    pub code: Option<String>,
    pub account: Option<String>,
    pub course: Option<String>,
    pub quantity: String,
    pub day_change: Option<f64>,
    pub day_change_total: Option<f64>,
    pub price_as_of: Option<String>,
    pub date_outlier: bool,
    pub nav_pending: bool,
    pub day_change_title: Option<String>,
    pub market: Option<f64>,
    pub profit: Option<f64>,
    pub rate: Option<f64>,
    pub href: Option<String>,
    pub dividend: Option<HoldingDividend>,
    pub current_price: Option<f64>,
    pub average_price: Option<f64>,
    pub purchase: Option<f64>,
}

pub(crate) fn unified_from_stock(view: &HoldingView, dividends: &DividendMaps) -> UnifiedRow {
    let valuation = model::holding_valuation(&ValuationInput::from(view));
    UnifiedRow {
        name: view.name.clone(),
        kind: labels::DOMESTIC_STOCK,
        sub: view.code.clone(),
        code: Some(view.code.clone()),
        account: view.account.clone(),
        course: None,
        quantity: labels::stock_shares_label(&format::format_number_value(view.shares)),
        day_change: view.day_change,
        day_change_total: view.day_change.map(|change| change * view.shares),
        price_as_of: view.price_as_of.clone(),
        date_outlier: false,
        nav_pending: false,
        day_change_title: None,
        market: valuation.market_value,
        profit: valuation.profit_loss.amount,
        rate: valuation.profit_loss.rate,
        href: Some(holdings::holding_stock_href(&view.code)),
        dividend: Some(holdings::holding_dividend(
            &view.code,
            view.shares,
            view.average_price,
            dividends,
        )),
        current_price: Some(view.current_price),
        average_price: Some(view.average_price),
        purchase: Some(view.purchase),
    }
}

pub(crate) fn unified_from_fund(view: &FundView) -> UnifiedRow {
    UnifiedRow {
        name: view.name.clone(),
        kind: labels::MUTUAL_FUND,
        sub: view.course.clone().unwrap_or_default(),
        code: None,
        account: Some(view.account.clone()),
        course: view.course.clone(),
        quantity: labels::fund_units_label(&format::format_number_value(view.shares)),
        day_change: view.view_day_change(),
        day_change_total: view
            .view_day_change()
            .map(|change| change * view.shares / 10_000.0),
        price_as_of: view.price_as_of.clone(),
        date_outlier: false,
        nav_pending: view.nav_pending(),
        day_change_title: view.day_change_title(),
        market: Some(view.market_value),
        profit: Some(view.profit_loss),
        rate: view.profit_loss_rate,
        href: view.id.as_ref().map(urls::fund_holding_href),
        dividend: None,
        current_price: Some(view.base_price),
        average_price: Some(view.average_purchase_price),
        purchase: Some(view.purchase),
    }
}

pub(crate) fn unified_day_change_rate(row: &UnifiedRow) -> Option<f64> {
    if row.nav_pending {
        return None;
    }
    let (Some(market), Some(daily)) = (row.market, row.day_change_total) else {
        return None;
    };
    if market - daily <= 0.0 {
        return None;
    }
    Some(daily / (market - daily) * 100.0)
}

pub(crate) fn day_change_rate_cell(rate: Option<f64>, date: Option<String>) -> impl IntoView {
    let negative = balance_format::is_negative_rate(rate);
    let positive = balance_format::is_positive_rate(rate);
    let text = balance_format::format_signed_valuation_rate(rate, 1);
    let date_text = date.map(|date| format::format_price_date(&date));
    view! {
        <Amount
            kind=AmountKind::Delta
            text=text
            negative=negative
            positive=positive
            class="font-bold"
        />
        {date_text.map(|date| {
            view! { <DayChangeNote text=format!("({date})") /> }
        })}
    }
}

pub(crate) fn sort_unified(rows: &[UnifiedRow], key: FundSortKey, order: SortOrder) -> Vec<usize> {
    let value = |row: &UnifiedRow| match key {
        FundSortKey::MarketValue => row.market,
        _ => row.profit,
    };
    let mut indexed: Vec<usize> = (0..rows.len()).collect();
    indexed.sort_by(|&a, &b| {
        let ordering = match (value(&rows[a]), value(&rows[b])) {
            (Some(a), Some(b)) => {
                let ordering = b.total_cmp(&a);
                match order {
                    SortOrder::Desc => ordering,
                    SortOrder::Asc => ordering.reverse(),
                }
            }
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (Some(_), None) => std::cmp::Ordering::Less,
        };
        if ordering == std::cmp::Ordering::Equal {
            rows[a].name.cmp(&rows[b].name)
        } else {
            ordering
        }
    });
    indexed
}

pub(crate) fn holding_detail_cells(
    view: &HoldingView,
    dividends: &DividendMaps,
) -> (Vec<DetailCell>, Vec<DetailCell>) {
    let valuation = model::holding_valuation(&ValuationInput::from(view));
    let market_text = balance_format::format_row_valuation_amount(valuation.market_value);
    let pnl_text = balance_format::format_signed_row_valuation_amount(valuation.profit_loss.amount);
    let pnl_rate = balance_format::format_signed_valuation_rate(valuation.profit_loss.rate, 1);
    let pnl_negative = balance_format::is_negative_valuation(valuation.profit_loss.amount);
    let pnl_positive = balance_format::is_positive_valuation(valuation.profit_loss.amount);
    let (day_text, day_negative, day_positive) = (
        balance_format::format_signed_row_valuation_amount(view.day_change),
        balance_format::is_negative_valuation(view.day_change),
        balance_format::is_positive_valuation(view.day_change),
    );
    let dividend =
        holdings::holding_dividend(&view.code, view.shares, view.average_price, dividends);
    let current_text = format::format_currency(view.current_price);
    let average_text = format::format_currency(view.average_price);
    let purchase_text = format::format_currency(view.purchase);
    let shares_text = labels::stock_shares_label(&format::format_number_value(view.shares));
    let per_share = holdings::format_dividend_per_share(&dividend, 0);
    let annual = holdings::format_dividend_annual(&dividend);
    let yield_text = holdings::format_dividend_yield(&dividend);
    let as_of = labels::stock_price_point_value(view.price_as_of.as_deref());
    let left = vec![
        DetailCell::with_sub(
            labels::CURRENT_PRICE.to_string(),
            current_text,
            day_text,
            day_negative,
            day_positive,
        ),
        DetailCell::plain(labels::AVERAGE_PRICE.to_string(), average_text),
        DetailCell {
            label: labels::ACQUISITION_YIELD.to_string(),
            value: yield_text,
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative: balance_format::is_negative_rate(dividend.yield_value),
            positive: balance_format::is_positive_rate(dividend.yield_value),
            bold: false,
            kind: AmountKind::Delta,
        },
        DetailCell {
            label: labels::PER_SHARE_DIVIDEND.to_string(),
            value: per_share,
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative: balance_format::is_negative_rate(dividend.per_share),
            positive: balance_format::is_positive_rate(dividend.per_share),
            bold: false,
            kind: AmountKind::Delta,
        },
    ];
    let mut market_cell = match valuation.profit_loss.amount {
        Some(_) => DetailCell::strong_sub(
            labels::MARKET_VALUE.to_string(),
            market_text,
            pnl_text,
            pnl_negative,
            pnl_positive,
        ),
        None => DetailCell::plain(labels::MARKET_VALUE.to_string(), market_text),
    };
    market_cell.sub_rate = valuation.profit_loss.rate.map(|_| pnl_rate);
    market_cell.sub_rate_negative = balance_format::is_negative_rate(valuation.profit_loss.rate);
    market_cell.sub_rate_positive = balance_format::is_positive_rate(valuation.profit_loss.rate);
    let right = vec![
        DetailCell::plain(labels::QUANTITY.to_string(), shares_text),
        DetailCell::plain(labels::TOTAL_ACQUISITION.to_string(), purchase_text),
        market_cell,
        DetailCell::plain(labels::ANNUAL_EXPECTED_DIVIDEND.to_string(), annual),
        DetailCell::plain(labels::STOCK_PRICE.to_string(), as_of),
    ];
    (left, right)
}

pub(crate) fn fund_detail_cells(view: &FundView) -> (Vec<DetailCell>, Vec<DetailCell>) {
    let base_text = format::format_unit_price(
        view.base_price,
        format::unit_price_decimals(std::iter::once(view.base_price)),
    );
    let day_text = if view.nav_pending() {
        labels::PENDING_UPDATE.to_string()
    } else {
        balance_format::format_signed_row_valuation_amount(view.view_day_change())
    };
    let day_negative = balance_format::is_negative_valuation(view.view_day_change());
    let day_positive = balance_format::is_positive_valuation(view.view_day_change());
    let month_text = if view.nav_pending() {
        labels::PENDING_UPDATE.to_string()
    } else {
        balance_format::format_signed_row_valuation_amount(view.base_price_month_change)
    };
    let month_negative = balance_format::is_negative_valuation(view.base_price_month_change);
    let month_positive = balance_format::is_positive_valuation(view.base_price_month_change);
    let average_text = format::format_unit_price(
        view.average_purchase_price,
        format::unit_price_decimals(std::iter::once(view.average_purchase_price)),
    );
    let course_text = view
        .course
        .clone()
        .unwrap_or_else(|| labels::FETCH_UNAVAILABLE.to_string());
    let shares_text = labels::fund_units_label(&format::format_number_value(view.shares));
    let purchase_text = format::format_currency(view.purchase);
    let market_text = balance_format::format_row_valuation_amount(Some(view.market_value));
    let pnl_text = balance_format::format_signed_row_valuation_amount(Some(view.profit_loss));
    let pnl_rate = balance_format::format_signed_valuation_rate(view.profit_loss_rate, 1);
    let pnl_rate_negative = balance_format::is_negative_rate(view.profit_loss_rate);
    let pnl_rate_positive = balance_format::is_positive_rate(view.profit_loss_rate);
    let pnl_negative = balance_format::is_negative_valuation(Some(view.profit_loss));
    let pnl_positive = balance_format::is_positive_valuation(Some(view.profit_loss));
    let as_of = view.price_as_of.clone().map_or_else(
        || labels::FETCH_UNAVAILABLE.to_string(),
        |date| format::format_iso_date(&date),
    );
    let left = vec![
        DetailCell::with_sub(
            labels::BASE_PRICE.to_string(),
            base_text,
            day_text,
            day_negative,
            day_positive,
        ),
        DetailCell {
            label: labels::BASE_PRICE_MONTH_CHANGE.to_string(),
            value: month_text,
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative: month_negative,
            positive: month_positive,
            bold: false,
            kind: AmountKind::Delta,
        },
        DetailCell::plain(labels::AVERAGE_PRICE_PER_10K.to_string(), average_text),
        DetailCell::plain(labels::DISTRIBUTION_COURSE.to_string(), course_text),
    ];
    let mut pnl_cell = DetailCell::strong_sub(
        labels::MARKET_VALUE.to_string(),
        market_text,
        pnl_text,
        pnl_negative,
        pnl_positive,
    );
    pnl_cell.sub_rate = Some(pnl_rate);
    pnl_cell.sub_rate_negative = pnl_rate_negative;
    pnl_cell.sub_rate_positive = pnl_rate_positive;
    let right = vec![
        DetailCell::plain(labels::FUND_SHARES.to_string(), shares_text),
        DetailCell::plain(labels::TOTAL_ACQUISITION.to_string(), purchase_text),
        pnl_cell,
        DetailCell::plain(labels::BASE_PRICE_AS_OF.to_string(), as_of),
    ];
    (left, right)
}

pub(crate) fn unified_detail_cells(row: &UnifiedRow) -> (Vec<DetailCell>, Vec<DetailCell>) {
    let is_fund = row.kind == labels::MUTUAL_FUND;
    let price_label = if is_fund {
        labels::BASE_PRICE
    } else {
        labels::CURRENT_PRICE
    };
    let price_text = row
        .current_price
        .map_or_else(|| "—".to_string(), format::format_currency);
    let (day_text, day_negative, day_positive) = if row.nav_pending {
        (labels::PENDING_UPDATE.to_string(), false, false)
    } else {
        let rate = unified_day_change_rate(row);
        (
            balance_format::format_signed_valuation_rate(rate, 1),
            balance_format::is_negative_rate(rate),
            balance_format::is_positive_rate(rate),
        )
    };
    let average_text = row
        .average_price
        .map_or_else(|| "—".to_string(), format::format_currency);
    let purchase_text = row
        .purchase
        .map_or_else(|| "—".to_string(), format::format_currency);
    let market_text = balance_format::format_row_valuation_amount(row.market);
    let pnl_text = balance_format::format_signed_row_valuation_amount(row.profit);
    let pnl_rate = balance_format::format_signed_valuation_rate(row.rate, 1);
    let pnl_negative = balance_format::is_negative_valuation(row.profit);
    let pnl_positive = balance_format::is_positive_valuation(row.profit);
    let as_of = if is_fund {
        row.price_as_of.clone().map_or_else(
            || labels::FETCH_UNAVAILABLE.to_string(),
            |date| format::format_iso_date(&date),
        )
    } else {
        labels::stock_price_point_value(row.price_as_of.as_deref())
    };
    let mut left = vec![
        DetailCell::with_sub(
            price_label.to_string(),
            price_text,
            day_text,
            day_negative,
            day_positive,
        ),
        DetailCell::plain(
            if is_fund {
                labels::AVERAGE_PRICE_PER_10K.to_string()
            } else {
                labels::AVERAGE_PRICE.to_string()
            },
            average_text,
        ),
    ];
    if is_fund {
        left.push(DetailCell::plain(
            labels::DISTRIBUTION_COURSE.to_string(),
            row.course
                .clone()
                .unwrap_or_else(|| labels::FETCH_UNAVAILABLE.to_string()),
        ));
    } else {
        let per_share_value = row
            .dividend
            .as_ref()
            .and_then(|dividend| dividend.per_share);
        let yield_value = row
            .dividend
            .as_ref()
            .and_then(|dividend| dividend.yield_value);
        let per_share = row.dividend.as_ref().map_or_else(
            || labels::FETCH_UNAVAILABLE.to_string(),
            |dividend| {
                crate::features::asset_balance::holdings::format_dividend_per_share(
                    dividend,
                    format::unit_price_decimals(dividend.per_share.into_iter()),
                )
            },
        );
        let yield_text = row.dividend.as_ref().map_or_else(
            || labels::FETCH_UNAVAILABLE.to_string(),
            crate::features::asset_balance::holdings::format_dividend_yield,
        );
        left.push(DetailCell {
            label: labels::ACQUISITION_YIELD.to_string(),
            value: yield_text,
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative: balance_format::is_negative_rate(yield_value),
            positive: balance_format::is_positive_rate(yield_value),
            bold: false,
            kind: AmountKind::Delta,
        });
        left.push(DetailCell {
            label: labels::PER_SHARE_DIVIDEND.to_string(),
            value: per_share,
            sub: None,
            sub_rate: None,
            sub_rate_negative: false,
            sub_rate_positive: false,
            negative: balance_format::is_negative_rate(per_share_value),
            positive: balance_format::is_positive_rate(per_share_value),
            bold: false,
            kind: AmountKind::Delta,
        });
    }
    let mut market_cell = match row.profit {
        Some(_) => DetailCell::strong_sub(
            labels::MARKET_VALUE.to_string(),
            market_text,
            pnl_text,
            pnl_negative,
            pnl_positive,
        ),
        None => DetailCell::plain(labels::MARKET_VALUE.to_string(), market_text),
    };
    market_cell.sub_rate = row.rate.map(|_| pnl_rate);
    market_cell.sub_rate_negative = balance_format::is_negative_rate(row.rate);
    market_cell.sub_rate_positive = balance_format::is_positive_rate(row.rate);
    let mut right = vec![
        DetailCell::plain(
            if is_fund {
                labels::FUND_SHARES.to_string()
            } else {
                labels::QUANTITY.to_string()
            },
            row.quantity.clone(),
        ),
        DetailCell::plain(labels::TOTAL_ACQUISITION.to_string(), purchase_text),
        market_cell,
    ];
    if !is_fund {
        let annual = row.dividend.as_ref().map_or_else(
            || labels::FETCH_UNAVAILABLE.to_string(),
            crate::features::asset_balance::holdings::format_dividend_annual,
        );
        right.push(DetailCell::plain(
            labels::ANNUAL_EXPECTED_DIVIDEND.to_string(),
            annual,
        ));
    }
    right.push(DetailCell::plain(
        if is_fund {
            labels::BASE_PRICE_AS_OF.to_string()
        } else {
            labels::STOCK_PRICE.to_string()
        },
        as_of,
    ));
    (left, right)
}
