use chrono::NaiveDate;
use csv::StringRecord;
use yew::prelude::*;

use super::receipt_template::ReceiptProps;
use crate::{services::parser::*, setting::*};

#[derive(PartialEq, Properties, Debug, Clone, Default)]
pub struct MutualFund {
    pub trade_date: Option<NaiveDate>,                   // 約定日
    pub settlement_date: Option<NaiveDate>,              // 受渡日
    pub fund_name: Option<String>,                       // ファンド名
    pub dividends: Option<String>,                       // 分配金
    pub account: Option<String>,                         // 口座
    pub shares: Option<u32>,                             // 数量[株]
    pub exchange_rate: Option<u32>,                      // 為替レート［円］
    pub cancellation_unit_price_yen: Option<u32>,        // 解約単価［円］
    pub cancellation_amount_yen: Option<u32>,            // 解約額［円］
    pub average_acquisition_price_yen: Option<f64>,      // 平均取得価額［円］
    pub realized_profit_and_loss: Option<i32>,           // 実現損益［円］
    pub taxes: Option<i32>,                              // 税額
    pub realized_profit_and_loss_after_tax: Option<i32>, // 実現損益(税引)
}

impl ReceiptProps for MutualFund {
    fn new() -> Self {
        MutualFund::default()
    }

    fn new_from_string_record(record: StringRecord) -> Self {
        let tmp_account = record.get(4).try_parse_string().unwrap();
        let tmp_realized_profit_and_loss = record.get(11).try_parse_num();
        let (taxes, tmp_realized_profit_and_loss_after_tax) =
            tmp_realized_profit_and_loss.map_or((None, None), |profit| {
                if profit > 0 {
                    if tmp_account.contains("特定") {
                        let tmp_taxes = (profit as f64 * TAX_RATE) as i32;
                        (Some(tmp_taxes), Some(profit - tmp_taxes))
                    } else {
                        (Some(0), Some(profit))
                    }
                } else {
                    (Some(0), Some(profit))
                }
            });

        Self {
            trade_date: record.get(0).try_parse_date(),
            settlement_date: record.get(1).try_parse_date(),
            fund_name: record.get(2).try_parse_string(),
            dividends: record.get(3).try_parse_string(),
            account: Some(tmp_account),
            shares: record.get(6).try_parse_num(),
            exchange_rate: record.get(7).try_parse_num(),
            cancellation_unit_price_yen: record.get(8).try_parse_num(),
            cancellation_amount_yen: record.get(9).try_parse_num(),
            average_acquisition_price_yen: record.get(10).try_parse_num(),
            realized_profit_and_loss: tmp_realized_profit_and_loss,
            taxes: taxes,
            realized_profit_and_loss_after_tax: tmp_realized_profit_and_loss_after_tax,
        }
    }

    fn get_date(&self) -> Option<NaiveDate> {
        self.trade_date
    }

    fn get_all_fields(&self) -> Vec<(&'static str, Option<String>)> {
        vec![
            ("trade_date", self.trade_date.map(|d| d.to_string())),
            (
                "settlement_date",
                self.settlement_date.map(|d| d.to_string()),
            ),
            ("fund_name", self.fund_name.clone()),
            ("dividends", self.dividends.clone()),
            ("account", self.account.clone()),
            ("shares", self.shares.map(|s| s.to_string())),
            ("exchange_rate", self.exchange_rate.map(|s| s.to_string())),
            (
                "cancellation_unit_price_yen",
                self.cancellation_unit_price_yen.map(|s| s.to_string()),
            ),
            (
                "cancellation_amount_yen",
                self.cancellation_amount_yen.map(|s| s.to_string()),
            ),
            (
                "average_acquisition_price_yen",
                self.average_acquisition_price_yen.map(|s| s.to_string()),
            ),
            (
                "realized_profit_and_loss",
                self.realized_profit_and_loss.map(|s| s.to_string()),
            ),
            ("taxes", self.taxes.map(|s| s.to_string())),
            (
                "realized_profit_and_loss_after_tax",
                self.realized_profit_and_loss_after_tax
                    .map(|s| s.to_string()),
            ),
        ]
    }

    fn view_summary(receipts: &[Self]) -> Html {
        let (total_realized_profit_and_loss, total_taxes, total_realized_profit_and_loss_after_tax) =
            receipts.iter().fold(
                (0, 0, 0),
                |(total_realized_profit_and_loss, withholding_tax, profit_and_loss), p| {
                    (
                        total_realized_profit_and_loss + p.realized_profit_and_loss.unwrap(),
                        withholding_tax + p.taxes.unwrap(),
                        profit_and_loss + p.realized_profit_and_loss_after_tax.unwrap(),
                    )
                },
            );

        html! {
            <tbody>
                <tr>
                    { Self::render_summary_th_td("total_realized_profit_and_loss", total_realized_profit_and_loss) }
                    { Self::render_summary_th_td("total_taxes", total_taxes as i32) }
                    { Self::render_summary_th_td("total_realized_profit_and_loss_after_tax", total_realized_profit_and_loss_after_tax) }
                </tr>
            </tbody>
        }
    }

    fn is_view_search() -> bool {
        false
    }
}
