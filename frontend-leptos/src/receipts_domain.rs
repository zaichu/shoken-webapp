use crate::dto::{Dividend, DomesticStock, DomesticStockSummary, Mutualfund};
use rust_decimal::{Decimal, RoundingStrategy};
use std::collections::BTreeMap;

const TAX_RATE: Decimal = Decimal::from_parts(20315, 0, 0, false, 5);

#[derive(Clone, Debug, PartialEq)]
pub struct DomesticDailySummary {
    pub filter: String,
    pub total_realized_profit_and_loss: Decimal,
    pub total_taxes: Decimal,
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq)]
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

pub fn format_date(value: &str) -> String {
    let bytes = value.as_bytes();
    if bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        format!("{}/{}/{}", &value[..4], &value[5..7], &value[8..])
    } else {
        "-".to_string()
    }
}

pub fn format_number(value: Decimal, maximum_fraction_digits: u32) -> String {
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
    let grouped_reversed: String = integer
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
    let grouped: String = grouped_reversed.chars().rev().collect();
    match fraction {
        Some(fraction) => format!("{sign}{grouped}.{fraction}"),
        None => format!("{sign}{grouped}"),
    }
}

pub fn format_currency(value: Decimal) -> String {
    let formatted = format_number(value.abs(), 15);
    if value.is_sign_negative() {
        format!("¥ -{formatted}")
    } else {
        format!("¥ {formatted}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{Dividend, DomesticStock, Mutualfund};
    use rust_decimal_macros::dec;

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
}
