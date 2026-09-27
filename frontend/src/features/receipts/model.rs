use crate::api::dto::{Dividend, DomesticStock, Mutualfund};
#[cfg(test)]
use rust_decimal::Decimal;
#[cfg(test)]
use std::collections::BTreeMap;

pub use shared::domain::DividendSummary as DividendTotals;
#[cfg(test)]
pub use shared::format::format_percentage_value;
pub use shared::format::{format_currency, format_number};
pub use shared::normalize::normalize_security_code;
#[cfg(test)]
pub use shared::summary::DomesticDailySummary;
pub use shared::summary::{
    dividend_totals as calculate_dividends, domestic_daily as calculate_domestic_daily,
    domestic_total as calculate_domestic_total, mutualfund_totals as calculate_mutual_funds,
};

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DividendGroupSummary {
    pub filter: String,
    pub total_dividends_before_tax: Decimal,
    pub total_taxes: Decimal,
    pub total_net_amount_received: Decimal,
}

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MutualFundGroupSummary {
    pub filter: String,
    pub cancellation_amount_yen: Decimal,
    pub realized_profit_and_loss: Decimal,
    pub taxes: Decimal,
    pub realized_profit_and_loss_after_tax: Decimal,
}

pub fn sort_dividends(rows: &[Dividend]) -> Vec<Dividend> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| b.settlement_date.cmp(&a.settlement_date));
    sorted
}

pub fn sort_domestic_stocks(rows: &[DomesticStock]) -> Vec<DomesticStock> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| b.trade_date.cmp(&a.trade_date));
    sorted
}

pub fn sort_mutual_funds(rows: &[Mutualfund]) -> Vec<Mutualfund> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| b.trade_date.cmp(&a.trade_date));
    sorted
}

#[cfg(test)]
pub fn group_dividends_by_month(rows: &[Dividend]) -> Vec<DividendGroupSummary> {
    let mut groups: BTreeMap<String, DividendGroupSummary> = BTreeMap::new();
    for row in rows {
        let key = create_year_month_key(&row.settlement_date);
        let group = groups
            .entry(key.clone())
            .or_insert_with(|| DividendGroupSummary {
                filter: key,
                ..DividendGroupSummary::default()
            });
        group.total_dividends_before_tax += row.dividends_before_tax;
        group.total_taxes += row.taxes;
        group.total_net_amount_received += row.net_amount_received;
    }
    groups.into_values().rev().collect()
}

#[cfg(test)]
pub fn group_mutual_funds_by_month(rows: &[Mutualfund]) -> Vec<MutualFundGroupSummary> {
    let mut groups: BTreeMap<String, MutualFundGroupSummary> = BTreeMap::new();
    for row in rows {
        let key = create_year_month_key(&row.trade_date);
        let group = groups
            .entry(key.clone())
            .or_insert_with(|| MutualFundGroupSummary {
                filter: key,
                ..MutualFundGroupSummary::default()
            });
        group.cancellation_amount_yen += row.cancellation_amount_yen;
        group.realized_profit_and_loss += row.realized_profit_and_loss;
        group.taxes += row.taxes;
        group.realized_profit_and_loss_after_tax += row.realized_profit_and_loss_after_tax;
    }
    groups.into_values().rev().collect()
}

fn valid_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return false;
    }
    let year = value[..4].parse::<u32>().unwrap_or(0);
    let month = value[5..7].parse::<u32>().unwrap_or(0);
    let day = value[8..].parse::<u32>().unwrap_or(0);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let maximum_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=maximum_day).contains(&day)
}

pub fn format_date(value: &str) -> String {
    if valid_iso_date(value) {
        format!("{}/{}/{}", &value[..4], &value[5..7], &value[8..])
    } else {
        "—".to_string()
    }
}

pub fn create_year_month_key(value: &str) -> String {
    if valid_iso_date(value) {
        value[..7].to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
pub fn create_iso_date_key(value: &str) -> String {
    if valid_iso_date(value) {
        value.to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
pub fn parse_number(value: &str) -> Decimal {
    value.replace(',', "").parse().unwrap_or(Decimal::ZERO)
}

#[cfg(test)]
pub fn safe_add(a: Decimal, b: Decimal) -> Decimal {
    a + b
}

#[cfg(test)]
pub fn safe_subtract(a: Decimal, b: Decimal) -> Decimal {
    a - b
}

#[cfg(test)]
pub fn safe_multiply(a: Decimal, b: Decimal) -> Decimal {
    a * b
}

#[cfg(test)]
pub fn safe_divide(a: Decimal, b: Decimal) -> Decimal {
    if b.is_zero() {
        Decimal::ZERO
    } else {
        (a / b).round_dp(10).normalize()
    }
}

#[cfg(test)]
pub fn calculate_percentage(value: Decimal, total: Decimal, decimals: u32) -> Decimal {
    if total.is_zero() {
        Decimal::ZERO
    } else {
        ((value / total) * Decimal::ONE_HUNDRED)
            .round_dp(decimals)
            .normalize()
    }
}

#[cfg(test)]
pub fn format_percentage(value: Decimal, total: Decimal, decimals: u32) -> String {
    if total.is_zero() {
        "0%".to_string()
    } else {
        format_percentage_value(calculate_percentage(value, total, decimals), decimals)
    }
}

#[cfg(test)]
mod tests;
