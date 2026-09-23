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
    rows.iter().fold(DividendTotals::default(), |mut total, row| {
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
    let (sign, unsigned) = raw.strip_prefix('-').map_or(("", raw.as_str()), |v| ("-", v));
    let (integer, fraction) = unsigned.split_once('.').map_or((unsigned, None), |(i, f)| (i, Some(f)));
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
    fn domestic_stock_daily_tax_uses_net_specific_account_profit() {
        let rows = vec![
            domestic("2024-03-01", "特定口座", dec!(10000), dec!(2031)),
            domestic("2024-03-01", "特定口座", dec!(-10000), dec!(0)),
        ];

        assert_eq!(calculate_domestic_daily(&rows)[0].total_taxes, dec!(0));
        assert_eq!(calculate_domestic_total(&rows).total_taxes, dec!(0));
    }

    #[test]
    fn aggregate_dividends_and_mutual_funds_preserves_decimals() {
        let dividends = vec![dividend(dec!(10.25), dec!(2), dec!(8.25))];
        let funds = vec![mutual_fund(dec!(10.25), dec!(2), dec!(8.25))];

        assert_eq!(calculate_dividends(&dividends).total_net_amount_received, dec!(8.25));
        assert_eq!(calculate_mutual_funds(&funds).total_realized_profit_and_loss, dec!(10.25));
    }

    #[test]
    fn formats_receipt_values_like_react() {
        assert_eq!(format_date("2024-03-05"), "2024/03/05");
        assert_eq!(format_number(dec!(12345.678), 2), "12,345.68");
        assert_eq!(format_currency(dec!(-12345.5)), "¥ -12,345.5");
    }

    fn domestic(date: &str, account: &str, profit: rust_decimal::Decimal, taxes: rust_decimal::Decimal) -> DomesticStock {
        DomesticStock {
            id: String::new(), trade_date: date.into(), settlement_date: date.into(),
            security_code: String::new(), security_name: String::new(), account: account.into(),
            shares: dec!(0), asked_price: dec!(0), proceeds: dec!(0), purchase_price: dec!(0),
            realized_profit_and_loss: profit, taxes,
            realized_profit_and_loss_after_tax: profit - taxes,
            created_at: String::new(), updated_at: String::new(),
        }
    }

    fn dividend(before_tax: rust_decimal::Decimal, taxes: rust_decimal::Decimal, net: rust_decimal::Decimal) -> Dividend {
        Dividend {
            id: String::new(), settlement_date: "2024-03-01".into(), product: String::new(),
            account: String::new(), security_code: String::new(), security_name: String::new(),
            unit_price: dec!(0), shares: dec!(0), dividends_before_tax: before_tax, taxes,
            net_amount_received: net, created_at: String::new(), updated_at: String::new(),
        }
    }

    fn mutual_fund(profit: rust_decimal::Decimal, taxes: rust_decimal::Decimal, after_tax: rust_decimal::Decimal) -> Mutualfund {
        Mutualfund {
            id: String::new(), trade_date: "2024-03-01".into(), settlement_date: "2024-03-05".into(),
            fund_name: String::new(), account: String::new(), shares: dec!(0), exchange_rate: dec!(0),
            cancellation_unit_price_yen: dec!(0), cancellation_amount_yen: dec!(0),
            average_acquisition_price_yen: dec!(0), realized_profit_and_loss: profit, taxes,
            realized_profit_and_loss_after_tax: after_tax, dividends: None,
            created_at: String::new(), updated_at: String::new(),
        }
    }
}
