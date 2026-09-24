use crate::dto::{Dividend, DomesticStock, DomesticStockSummary, Mutualfund};
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const TAX_RATE: Decimal = Decimal::from_parts(20315, 0, 0, false, 5);

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct DomesticDailySummary {
    pub filter: String,
    pub total_realized_profit_and_loss: Decimal,
    pub total_taxes: Decimal,
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct DividendTotals {
    pub total_dividends_before_tax: Decimal,
    pub total_taxes: Decimal,
    pub total_net_amount_received: Decimal,
}

pub fn calculate_domestic_daily(rows: &[DomesticStock]) -> Vec<DomesticDailySummary> {
    let mut groups: BTreeMap<&str, (Decimal, Decimal)> = BTreeMap::new();
    for row in rows {
        let totals = groups.entry(&row.trade_date).or_default();
        if row.account.contains("特定") {
            totals.0 += row.realized_profit_and_loss;
        } else {
            totals.1 += row.realized_profit_and_loss;
        }
    }
    groups
        .into_iter()
        .rev()
        .map(|(date, (specific, tax_exempt))| {
            let profit = specific + tax_exempt;
            let taxes = if specific.is_sign_positive() {
                (specific * TAX_RATE).floor()
            } else {
                Decimal::ZERO
            };
            DomesticDailySummary {
                filter: date.to_string(),
                total_realized_profit_and_loss: profit,
                total_taxes: taxes,
                total_realized_profit_and_loss_after_tax: profit - taxes,
            }
        })
        .collect()
}

pub fn calculate_domestic_total(rows: &[DomesticStock]) -> DomesticStockSummary {
    calculate_domestic_daily(rows).into_iter().fold(
        DomesticStockSummary {
            total_realized_profit_and_loss: Decimal::ZERO,
            total_taxes: Decimal::ZERO,
            total_realized_profit_and_loss_after_tax: Decimal::ZERO,
        },
        |mut total, day| {
            total.total_realized_profit_and_loss += day.total_realized_profit_and_loss;
            total.total_taxes += day.total_taxes;
            total.total_realized_profit_and_loss_after_tax +=
                day.total_realized_profit_and_loss_after_tax;
            total
        },
    )
}

pub fn calculate_dividends(rows: &[Dividend]) -> DividendTotals {
    rows.iter()
        .fold(DividendTotals::default(), |mut total, row| {
            total.total_dividends_before_tax += row.dividends_before_tax;
            total.total_taxes += row.taxes;
            total.total_net_amount_received += row.net_amount_received;
            total
        })
}

pub fn calculate_mutual_funds(rows: &[Mutualfund]) -> DomesticStockSummary {
    rows.iter().fold(
        DomesticStockSummary {
            total_realized_profit_and_loss: Decimal::ZERO,
            total_taxes: Decimal::ZERO,
            total_realized_profit_and_loss_after_tax: Decimal::ZERO,
        },
        |mut total, row| {
            total.total_realized_profit_and_loss += row.realized_profit_and_loss;
            total.total_taxes += row.taxes;
            total.total_realized_profit_and_loss_after_tax +=
                row.realized_profit_and_loss_after_tax;
            total
        },
    )
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DividendGroupSummary {
    pub filter: String,
    pub total_dividends_before_tax: Decimal,
    pub total_taxes: Decimal,
    pub total_net_amount_received: Decimal,
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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
        "-".to_string()
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

pub fn format_number(value: Decimal, maximum_fraction_digits: u32) -> String {
    format_number_with_options(value, 0, maximum_fraction_digits, true)
}

pub fn format_number_with_options(
    value: Decimal,
    minimum_fraction_digits: u32,
    maximum_fraction_digits: u32,
    use_grouping: bool,
) -> String {
    let rounded = value.round_dp_with_strategy(
        maximum_fraction_digits,
        RoundingStrategy::MidpointAwayFromZero,
    );
    let raw = rounded.normalize().to_string();
    let (sign, unsigned) = raw
        .strip_prefix('-')
        .map_or(("", raw.as_str()), |v| ("-", v));
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |(i, f)| (i, Some(f)));
    let grouped = if use_grouping {
        let reversed: String = integer
            .chars()
            .rev()
            .enumerate()
            .flat_map(|(index, character)| {
                if (index + 1) % 3 == 0 && index + 1 < integer.len() {
                    vec![character, ',']
                } else {
                    vec![character]
                }
            })
            .collect();
        reversed.chars().rev().collect()
    } else {
        integer.to_string()
    };
    let mut fraction = fraction.unwrap_or_default().to_string();
    while fraction.len() < minimum_fraction_digits as usize {
        fraction.push('0');
    }
    if fraction.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped}.{fraction}")
    }
}

pub fn format_currency(value: Decimal) -> String {
    format_currency_with_options(value, "¥", 0, 15)
}

pub fn format_currency_with_options(
    value: Decimal,
    currency: &str,
    minimum_fraction_digits: u32,
    maximum_fraction_digits: u32,
) -> String {
    let formatted = format_number_with_options(
        value.abs(),
        minimum_fraction_digits,
        maximum_fraction_digits,
        true,
    );
    if value.is_sign_negative() {
        format!("{currency} -{formatted}")
    } else {
        format!("{currency} {formatted}")
    }
}

#[cfg(test)]
pub fn parse_number(value: &str) -> Decimal {
    value.replace(',', "").parse().unwrap_or(Decimal::ZERO)
}

pub fn normalize_security_code(value: &str) -> String {
    value
        .split([':', '：'])
        .next()
        .unwrap_or_default()
        .split_whitespace()
        .collect::<String>()
        .to_uppercase()
}

#[cfg(test)]
pub fn normalize_security_name(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'Ａ'..='Ｚ' | 'ａ'..='ｚ' | '０'..='９' => {
                char::from_u32(character as u32 - 0xfee0).unwrap_or(character)
            }
            _ => character,
        })
        .collect()
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
pub fn format_percentage_value(value: Decimal, decimals: u32) -> String {
    format!(
        "{}%",
        format_number_with_options(value, decimals, decimals, false)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{Dividend, DomesticStock, Mutualfund};
    use rust_decimal_macros::dec;
    use serde::de::DeserializeOwned;

    #[derive(Deserialize)]
    struct FixtureDocument<T> {
        cases: Vec<T>,
    }

    #[derive(Deserialize)]
    struct DomesticInputCase {
        name: String,
        rows: Vec<DomesticFixtureRow>,
    }

    #[derive(Deserialize)]
    struct DomesticFixtureRow {
        trade_date: String,
        account: String,
        realized_profit_and_loss: Decimal,
        taxes: Decimal,
    }

    #[derive(Deserialize)]
    struct DomesticExpectedCase {
        name: String,
        daily: Vec<DomesticDailySummary>,
        total: DomesticStockSummary,
        row_taxes_total: Decimal,
    }

    #[derive(Deserialize)]
    struct DividendInputCase {
        name: String,
        rows: Vec<DividendFixtureRow>,
    }

    #[derive(Deserialize)]
    struct DividendFixtureRow {
        dividends_before_tax: Decimal,
        taxes: Decimal,
        net_amount_received: Decimal,
    }

    #[derive(Deserialize)]
    struct DividendExpectedCase {
        name: String,
        total: DividendTotals,
    }

    #[derive(Deserialize)]
    struct MutualFundInputCase {
        name: String,
        rows: Vec<MutualFundFixtureRow>,
    }

    #[derive(Deserialize)]
    struct MutualFundFixtureRow {
        realized_profit_and_loss: Decimal,
        taxes: Decimal,
        realized_profit_and_loss_after_tax: Decimal,
    }

    #[derive(Deserialize)]
    struct MutualFundExpectedCase {
        name: String,
        total: DomesticStockSummary,
    }

    fn fixture<T: DeserializeOwned>(json: &str) -> FixtureDocument<T> {
        serde_json::from_str(json).expect("shared receipt fixture parses")
    }

    #[test]
    fn shared_domestic_fixtures_match() {
        let input: FixtureDocument<DomesticInputCase> = fixture(include_str!(
            "../tests/fixtures/receipts/domestic-input.json"
        ));
        let expected: FixtureDocument<DomesticExpectedCase> = fixture(include_str!(
            "../tests/fixtures/receipts/domestic-expected.json"
        ));

        for (input, expected) in input.cases.into_iter().zip(expected.cases) {
            assert_eq!(input.name, expected.name);
            let rows: Vec<_> = input
                .rows
                .into_iter()
                .map(|row| {
                    domestic(
                        &row.trade_date,
                        &row.account,
                        row.realized_profit_and_loss,
                        row.taxes,
                    )
                })
                .collect();
            let daily = calculate_domestic_daily(&rows);
            assert_eq!(daily, expected.daily, "{} daily", input.name);
            assert_eq!(
                calculate_domestic_total(&rows),
                expected.total,
                "{} total",
                input.name
            );
            assert_eq!(
                rows.iter().map(|row| row.taxes).sum::<Decimal>(),
                expected.row_taxes_total,
                "{} row taxes",
                input.name
            );
        }
    }

    #[test]
    fn shared_dividend_fixtures_match() {
        let input: FixtureDocument<DividendInputCase> = fixture(include_str!(
            "../tests/fixtures/receipts/dividend-input.json"
        ));
        let expected: FixtureDocument<DividendExpectedCase> = fixture(include_str!(
            "../tests/fixtures/receipts/dividend-expected.json"
        ));

        for (input, expected) in input.cases.into_iter().zip(expected.cases) {
            assert_eq!(input.name, expected.name);
            let rows: Vec<_> = input
                .rows
                .into_iter()
                .map(|row| dividend(row.dividends_before_tax, row.taxes, row.net_amount_received))
                .collect();
            assert_eq!(calculate_dividends(&rows), expected.total, "{}", input.name);
        }
    }

    #[test]
    fn shared_mutual_fund_fixtures_match() {
        let input: FixtureDocument<MutualFundInputCase> = fixture(include_str!(
            "../tests/fixtures/receipts/mutualfund-input.json"
        ));
        let expected: FixtureDocument<MutualFundExpectedCase> = fixture(include_str!(
            "../tests/fixtures/receipts/mutualfund-expected.json"
        ));

        for (input, expected) in input.cases.into_iter().zip(expected.cases) {
            assert_eq!(input.name, expected.name);
            let rows: Vec<_> = input
                .rows
                .into_iter()
                .map(|row| {
                    mutual_fund(
                        row.realized_profit_and_loss,
                        row.taxes,
                        row.realized_profit_and_loss_after_tax,
                    )
                })
                .collect();
            assert_eq!(
                calculate_mutual_funds(&rows),
                expected.total,
                "{}",
                input.name
            );
        }
    }

    #[test]
    fn domestic_daily_groups_rows_with_the_same_date() {
        let rows = vec![
            domestic("2024-03-01", "特定口座", dec!(1000), dec!(0)),
            domestic("2024-03-01", "特定口座", dec!(2000), dec!(0)),
        ];

        assert_eq!(
            calculate_domestic_daily(&rows),
            vec![daily("2024-03-01", dec!(3000), dec!(609), dec!(2391))]
        );
    }

    #[test]
    fn domestic_daily_sorts_unsorted_dates_descending() {
        let rows = vec![
            domestic("2023-12-31", "特定口座", dec!(1), dec!(0)),
            domestic("2024-02-01", "特定口座", dec!(1), dec!(0)),
            domestic("2024-01-02", "特定口座", dec!(1), dec!(0)),
            domestic("2024-01-01", "特定口座", dec!(1), dec!(0)),
        ];

        let dates: Vec<_> = calculate_domestic_daily(&rows)
            .into_iter()
            .map(|summary| summary.filter)
            .collect();
        assert_eq!(
            dates,
            ["2024-02-01", "2024-01-02", "2024-01-01", "2023-12-31"]
        );
    }

    #[test]
    fn domestic_daily_taxes_specific_account_profit() {
        let result =
            calculate_domestic_daily(&[domestic("2024-03-01", "特定口座", dec!(10000), dec!(0))]);

        assert_eq!(
            result[0],
            daily("2024-03-01", dec!(10000), dec!(2031), dec!(7969))
        );
    }

    #[test]
    fn domestic_daily_does_not_tax_specific_account_loss() {
        let result =
            calculate_domestic_daily(&[domestic("2024-03-01", "特定口座", dec!(-10000), dec!(0))]);

        assert_eq!(
            result[0],
            daily("2024-03-01", dec!(-10000), dec!(0), dec!(-10000))
        );
    }

    #[test]
    fn domestic_daily_does_not_tax_nisa_profit() {
        let result =
            calculate_domestic_daily(&[domestic("2024-03-01", "NISA口座", dec!(10000), dec!(0))]);

        assert_eq!(
            result[0],
            daily("2024-03-01", dec!(10000), dec!(0), dec!(10000))
        );
    }

    #[test]
    fn domestic_daily_only_taxes_specific_account_in_mixed_accounts() {
        let rows = vec![
            domestic("2024-03-01", "特定口座", dec!(10000), dec!(0)),
            domestic("2024-03-01", "NISA口座", dec!(5000), dec!(0)),
        ];

        assert_eq!(
            calculate_domestic_daily(&rows)[0],
            daily("2024-03-01", dec!(15000), dec!(2031), dec!(12969))
        );
    }

    #[test]
    fn domestic_daily_returns_separate_dates_descending() {
        let rows = vec![
            domestic("2024-03-02", "特定口座", dec!(2000), dec!(0)),
            domestic("2024-03-01", "特定口座", dec!(1000), dec!(0)),
        ];

        assert_eq!(
            calculate_domestic_daily(&rows),
            vec![
                daily("2024-03-02", dec!(2000), dec!(406), dec!(1594)),
                daily("2024-03-01", dec!(1000), dec!(203), dec!(797)),
            ]
        );
    }

    #[test]
    fn domestic_total_returns_zero_for_empty_input() {
        assert_eq!(
            calculate_domestic_total(&[]),
            stock_totals(dec!(0), dec!(0), dec!(0))
        );
    }

    #[test]
    fn domestic_total_sums_all_daily_fields() {
        let rows = vec![
            domestic("2024-03-01", "特定口座", dec!(10000), dec!(0)),
            domestic("2024-03-02", "特定口座", dec!(-3000), dec!(0)),
            domestic("2024-03-03", "特定口座", dec!(5000), dec!(0)),
        ];

        assert_eq!(
            calculate_domestic_total(&rows),
            stock_totals(dec!(12000), dec!(3046), dec!(8954))
        );
    }

    #[test]
    fn dividends_return_zero_for_empty_input() {
        assert_eq!(calculate_dividends(&[]), DividendTotals::default());
    }

    #[test]
    fn dividends_return_the_single_row_values() {
        assert_eq!(
            calculate_dividends(&[dividend(dec!(5000), dec!(1015), dec!(3985))]),
            dividend_totals(dec!(5000), dec!(1015), dec!(3985))
        );
    }

    #[test]
    fn dividends_sum_multiple_rows() {
        let rows = vec![
            dividend(dec!(1000), dec!(203), dec!(797)),
            dividend(dec!(2500), dec!(507), dec!(1993)),
            dividend(dec!(1800), dec!(365), dec!(1435)),
        ];

        assert_eq!(
            calculate_dividends(&rows),
            dividend_totals(dec!(5300), dec!(1075), dec!(4225))
        );
    }

    #[test]
    fn dividends_sum_negative_values() {
        let rows = vec![
            dividend(dec!(1000), dec!(200), dec!(800)),
            dividend(dec!(-150), dec!(-30), dec!(-120)),
        ];

        assert_eq!(
            calculate_dividends(&rows),
            dividend_totals(dec!(850), dec!(170), dec!(680))
        );
    }

    #[test]
    fn mutual_funds_return_zero_for_empty_input() {
        assert_eq!(
            calculate_mutual_funds(&[]),
            stock_totals(dec!(0), dec!(0), dec!(0))
        );
    }

    #[test]
    fn mutual_funds_return_the_single_row_values() {
        assert_eq!(
            calculate_mutual_funds(&[mutual_fund(dec!(50000), dec!(10157), dec!(39843))]),
            stock_totals(dec!(50000), dec!(10157), dec!(39843))
        );
    }

    #[test]
    fn mutual_funds_sum_multiple_rows() {
        let rows = vec![
            mutual_fund(dec!(50000), dec!(10157), dec!(39843)),
            mutual_fund(dec!(-10000), dec!(0), dec!(-10000)),
            mutual_fund(dec!(25000), dec!(5078), dec!(19922)),
        ];

        assert_eq!(
            calculate_mutual_funds(&rows),
            stock_totals(dec!(65000), dec!(15235), dec!(49765))
        );
    }

    #[test]
    fn receipt_rows_are_sorted_by_their_react_date_keys() {
        let dividends = vec![
            dividend_on("2024-03-31"),
            dividend_on("2024-01-31"),
            dividend_on("2024-02-29"),
        ];
        let domestic = vec![
            domestic("2024-03-15", "", dec!(0), dec!(0)),
            domestic("2024-01-10", "", dec!(0), dec!(0)),
            domestic("2024-02-20", "", dec!(0), dec!(0)),
        ];
        let funds = vec![
            mutual_fund_on("2024-03-12"),
            mutual_fund_on("2024-01-05"),
            mutual_fund_on("2024-02-18"),
        ];

        assert_eq!(
            sort_dividends(&dividends)
                .iter()
                .map(|row| row.settlement_date.as_str())
                .collect::<Vec<_>>(),
            ["2024-03-31", "2024-02-29", "2024-01-31"]
        );
        assert_eq!(
            sort_domestic_stocks(&domestic)
                .iter()
                .map(|row| row.trade_date.as_str())
                .collect::<Vec<_>>(),
            ["2024-03-15", "2024-02-20", "2024-01-10"]
        );
        assert_eq!(
            sort_mutual_funds(&funds)
                .iter()
                .map(|row| row.trade_date.as_str())
                .collect::<Vec<_>>(),
            ["2024-03-12", "2024-02-18", "2024-01-05"]
        );
    }

    #[test]
    fn monthly_dividend_groups_sum_and_sort_descending() {
        let rows = vec![
            dividend_full("2023-01-01", dec!(100), dec!(10), dec!(90)),
            dividend_full("2023-01-02", dec!(200), dec!(20), dec!(180)),
            dividend_full("2023-02-01", dec!(300), dec!(30), dec!(270)),
            dividend_full("2023-02-15", dec!(400), dec!(40), dec!(360)),
        ];

        let groups = group_dividends_by_month(&rows);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].filter, "2023-02");
        assert_eq!(groups[0].total_dividends_before_tax, dec!(700));
        assert_eq!(groups[0].total_taxes, dec!(70));
        assert_eq!(groups[0].total_net_amount_received, dec!(630));
        assert_eq!(groups[1].filter, "2023-01");
    }

    #[test]
    fn monthly_mutual_fund_groups_sum_and_sort_descending() {
        let mut january = mutual_fund_on("2023-01-01");
        january.cancellation_amount_yen = dec!(100);
        january.realized_profit_and_loss = dec!(20);
        january.taxes = dec!(4);
        january.realized_profit_and_loss_after_tax = dec!(16);
        let mut february = mutual_fund_on("2023-02-01");
        february.cancellation_amount_yen = dec!(300);
        february.realized_profit_and_loss = dec!(50);
        february.taxes = dec!(10);
        february.realized_profit_and_loss_after_tax = dec!(40);

        let groups = group_mutual_funds_by_month(&[january, february]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].filter, "2023-02");
        assert_eq!(groups[0].cancellation_amount_yen, dec!(300));
        assert_eq!(groups[0].realized_profit_and_loss, dec!(50));
        assert_eq!(groups[0].taxes, dec!(10));
        assert_eq!(groups[0].realized_profit_and_loss_after_tax, dec!(40));
    }

    #[test]
    fn date_helpers_match_react_output_and_reject_invalid_values() {
        assert_eq!(format_date("2023-12-25"), "2023/12/25");
        assert_eq!(format_date("2023-02-29"), "-");
        assert_eq!(format_date("string"), "-");
        assert_eq!(create_year_month_key("2023-12-25"), "2023-12");
        assert_eq!(create_year_month_key("invalid"), "");
        assert_eq!(create_iso_date_key("2023-01-05"), "2023-01-05");
        assert_eq!(create_iso_date_key("invalid"), "");
    }

    #[test]
    fn number_formatter_supports_fraction_and_grouping_options() {
        assert_eq!(format_number(dec!(12345), 2), "12,345");
        assert_eq!(format_number(dec!(-12345), 2), "-12,345");
        assert_eq!(format_number(dec!(123.456), 2), "123.46");
        assert_eq!(format_number_with_options(dec!(123), 2, 2, true), "123.00");
        assert_eq!(
            format_number_with_options(dec!(12345), 0, 2, false),
            "12345"
        );
    }

    #[test]
    fn currency_formatter_supports_sign_symbol_and_fraction_options() {
        assert_eq!(format_currency(dec!(12345)), "¥ 12,345");
        assert_eq!(format_currency(dec!(-12345)), "¥ -12,345");
        assert_eq!(
            format_currency_with_options(dec!(12345), "$", 0, 15),
            "$ 12,345"
        );
        assert_eq!(
            format_currency_with_options(dec!(123.456), "¥", 0, 2),
            "¥ 123.46"
        );
    }

    #[test]
    fn parse_normalize_and_decimal_helpers_match_react_cases() {
        assert_eq!(parse_number("1,234"), dec!(1234));
        assert_eq!(parse_number("invalid"), dec!(0));
        assert_eq!(normalize_security_code(" 7974: 任天堂 "), "7974");
        assert_eq!(normalize_security_code("brk.b"), "BRK.B");
        assert_eq!(normalize_security_name("ＫＤＤＩ１２３"), "KDDI123");
        assert_eq!(normalize_security_name("日本株ABC123"), "日本株ABC123");
        assert_eq!(safe_add(dec!(0.1), dec!(0.2)), dec!(0.3));
        assert_eq!(safe_subtract(dec!(0.3), dec!(0.1)), dec!(0.2));
        assert_eq!(safe_multiply(dec!(0.1), dec!(3)), dec!(0.3));
        assert_eq!(safe_divide(dec!(0.3), dec!(3)), dec!(0.1));
        assert_eq!(safe_divide(dec!(10), dec!(0)), dec!(0));
    }

    #[test]
    fn percentage_helpers_match_react_cases() {
        assert_eq!(format_percentage(dec!(25), dec!(100), 2), "25.00%");
        assert_eq!(format_percentage(dec!(1), dec!(3), 1), "33.3%");
        assert_eq!(format_percentage(dec!(10), dec!(0), 2), "0%");
        assert_eq!(format_percentage_value(dec!(25.5), 2), "25.50%");
        assert_eq!(calculate_percentage(dec!(25), dec!(100), 2), dec!(25));
        assert_eq!(calculate_percentage(dec!(1), dec!(3), 1), dec!(33.3));
        assert_eq!(calculate_percentage(dec!(10), dec!(0), 2), dec!(0));
        assert_eq!(TAX_RATE, dec!(0.20315));
    }

    fn daily(
        date: &str,
        profit: Decimal,
        taxes: Decimal,
        after_tax: Decimal,
    ) -> DomesticDailySummary {
        DomesticDailySummary {
            filter: date.into(),
            total_realized_profit_and_loss: profit,
            total_taxes: taxes,
            total_realized_profit_and_loss_after_tax: after_tax,
        }
    }

    fn stock_totals(profit: Decimal, taxes: Decimal, after_tax: Decimal) -> DomesticStockSummary {
        DomesticStockSummary {
            total_realized_profit_and_loss: profit,
            total_taxes: taxes,
            total_realized_profit_and_loss_after_tax: after_tax,
        }
    }

    fn dividend_totals(before_tax: Decimal, taxes: Decimal, net: Decimal) -> DividendTotals {
        DividendTotals {
            total_dividends_before_tax: before_tax,
            total_taxes: taxes,
            total_net_amount_received: net,
        }
    }

    fn domestic(
        date: &str,
        account: &str,
        profit: rust_decimal::Decimal,
        taxes: rust_decimal::Decimal,
    ) -> DomesticStock {
        DomesticStock {
            id: String::new(),
            trade_date: date.into(),
            settlement_date: date.into(),
            security_code: String::new(),
            security_name: String::new(),
            account: account.into(),
            shares: dec!(0),
            asked_price: dec!(0),
            proceeds: dec!(0),
            purchase_price: dec!(0),
            realized_profit_and_loss: profit,
            taxes,
            realized_profit_and_loss_after_tax: profit - taxes,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn dividend(
        before_tax: rust_decimal::Decimal,
        taxes: rust_decimal::Decimal,
        net: rust_decimal::Decimal,
    ) -> Dividend {
        Dividend {
            id: String::new(),
            settlement_date: "2024-03-01".into(),
            product: String::new(),
            account: String::new(),
            security_code: String::new(),
            security_name: String::new(),
            unit_price: dec!(0),
            shares: dec!(0),
            dividends_before_tax: before_tax,
            taxes,
            net_amount_received: net,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn dividend_on(date: &str) -> Dividend {
        dividend_full(date, dec!(0), dec!(0), dec!(0))
    }

    fn dividend_full(date: &str, before_tax: Decimal, taxes: Decimal, net: Decimal) -> Dividend {
        let mut row = dividend(before_tax, taxes, net);
        row.settlement_date = date.into();
        row
    }

    fn mutual_fund(
        profit: rust_decimal::Decimal,
        taxes: rust_decimal::Decimal,
        after_tax: rust_decimal::Decimal,
    ) -> Mutualfund {
        Mutualfund {
            id: String::new(),
            trade_date: "2024-03-01".into(),
            settlement_date: "2024-03-05".into(),
            fund_name: String::new(),
            account: String::new(),
            shares: dec!(0),
            exchange_rate: dec!(0),
            cancellation_unit_price_yen: dec!(0),
            cancellation_amount_yen: dec!(0),
            average_acquisition_price_yen: dec!(0),
            realized_profit_and_loss: profit,
            taxes,
            realized_profit_and_loss_after_tax: after_tax,
            dividends: None,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn mutual_fund_on(date: &str) -> Mutualfund {
        let mut row = mutual_fund(dec!(0), dec!(0), dec!(0));
        row.trade_date = date.into();
        row
    }
}
