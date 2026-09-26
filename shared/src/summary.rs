use crate::domain::{
    Dividend, DividendSummary, DomesticStock, DomesticStockSummary, Mutualfund, MutualfundSummary,
    NaiveDate,
};
use crate::tax::{is_taxable_account, tax_amount};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 国内株式の日次集計（表示用グループ行）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct DomesticDailySummary {
    pub filter: String,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_taxes: Decimal,
    #[cfg_attr(feature = "utoipa", schema(value_type = f64))]
    pub total_realized_profit_and_loss_after_tax: Decimal,
}

/// 国内株式の日次集計（新しい日付順）。
///
/// trade_date ごとに特定口座（account に「特定」を含む）と NISA 等口座の実現損益を分離し、
/// 特定口座合計がプラスの日だけ `floor(合計 * 税率)` を日次税額とする。
/// backend の検索 summary SQL と同一の仕様。
pub fn domestic_daily(rows: &[DomesticStock]) -> Vec<DomesticDailySummary> {
    let mut groups: BTreeMap<NaiveDate, (Decimal, Decimal)> = BTreeMap::new();
    for row in rows {
        let totals = groups.entry(row.trade_date).or_default();
        if is_taxable_account(&row.account) {
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
            let taxes = tax_amount(specific);
            DomesticDailySummary {
                filter: date.to_string(),
                total_realized_profit_and_loss: profit,
                total_taxes: taxes,
                total_realized_profit_and_loss_after_tax: profit - taxes,
            }
        })
        .collect()
}

/// 国内株式の検索条件全体の合計（日次集計の合算）。
pub fn domestic_total(rows: &[DomesticStock]) -> DomesticStockSummary {
    domestic_daily(rows)
        .into_iter()
        .fold(DomesticStockSummary::default(), |mut total, day| {
            total.total_realized_profit_and_loss += day.total_realized_profit_and_loss;
            total.total_taxes += day.total_taxes;
            total.total_realized_profit_and_loss_after_tax +=
                day.total_realized_profit_and_loss_after_tax;
            total
        })
}

/// 配当金の検索条件全体の合計（行の単純合算。backend の summary SQL と同じ仕様）。
pub fn dividend_totals(rows: &[Dividend]) -> DividendSummary {
    rows.iter()
        .fold(DividendSummary::default(), |mut total, row| {
            total.total_dividends_before_tax += row.dividends_before_tax;
            total.total_taxes += row.taxes;
            total.total_net_amount_received += row.net_amount_received;
            total
        })
}

/// 投資信託の検索条件全体の合計（行の単純合算。backend の summary SQL と同じ仕様）。
pub fn mutualfund_totals(rows: &[Mutualfund]) -> MutualfundSummary {
    rows.iter()
        .fold(MutualfundSummary::default(), |mut total, row| {
            total.total_realized_profit_and_loss += row.realized_profit_and_loss;
            total.total_taxes += row.taxes;
            total.total_realized_profit_and_loss_after_tax +=
                row.realized_profit_and_loss_after_tax;
            total
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DateTime, Uuid};
    use rust_decimal_macros::dec;

    fn domestic(date: &str, account: &str, pnl: Decimal, taxes: Decimal) -> DomesticStock {
        DomesticStock {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            trade_date: NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("valid date"),
            settlement_date: NaiveDate::parse_from_str(date, "%Y-%m-%d").expect("valid date"),
            security_code: "1234".to_string(),
            security_name: "テスト".to_string(),
            account: account.to_string(),
            shares: dec!(0),
            asked_price: dec!(0),
            proceeds: dec!(0),
            purchase_price: dec!(0),
            realized_profit_and_loss: pnl,
            taxes,
            realized_profit_and_loss_after_tax: pnl - taxes,
            created_at: DateTime::default(),
            updated_at: DateTime::default(),
        }
    }

    #[test]
    fn domestic_daily_groups_by_date_and_taxes_specific_sum() {
        let rows = vec![
            domestic("2024-03-01", "特定口座", dec!(10000), dec!(0)),
            domestic("2024-03-01", "特定口座", dec!(5000), dec!(0)),
            domestic("2024-03-01", "NISA", dec!(2000), dec!(0)),
            domestic("2024-03-02", "特定", dec!(-3000), dec!(0)),
        ];
        let daily = domestic_daily(&rows);
        assert_eq!(daily.len(), 2);
        // day1: 特定 +15000 → floor(15000*0.20315)=3047、NISA +2000 は非課税
        assert_eq!(daily[0].filter, "2024-03-02");
        assert_eq!(daily[0].total_taxes, dec!(0));
        assert_eq!(daily[1].filter, "2024-03-01");
        assert_eq!(daily[1].total_realized_profit_and_loss, dec!(17000));
        assert_eq!(daily[1].total_taxes, dec!(3047));
        assert_eq!(
            daily[1].total_realized_profit_and_loss_after_tax,
            dec!(13953)
        );

        let total = domestic_total(&rows);
        assert_eq!(total.total_realized_profit_and_loss, dec!(14000));
        assert_eq!(total.total_taxes, dec!(3047));
        assert_eq!(total.total_realized_profit_and_loss_after_tax, dec!(10953));
    }
}
