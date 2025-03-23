use chrono::NaiveDate;
use csv::StringRecord;
use itertools::Itertools;
use yew::prelude::*;

use super::receipt_template::ReceiptProps;
use crate::{services::parser::*, setting::*};

#[derive(PartialEq, Properties, Debug, Clone, Default)]
pub struct DomesticStock {
    pub trade_date: Option<NaiveDate>,                         // 約定日
    pub settlement_date: Option<NaiveDate>,                    // 受渡日
    pub security_code: Option<String>,                         // 銘柄コード
    pub security_name: Option<String>,                         // 銘柄名
    pub account: Option<String>,                               // 口座
    pub shares: Option<i32>,                                   // 数量[株]
    pub asked_price: Option<f64>,                              // 売却/決済単価[円]
    pub proceeds: Option<i32>,                                 // 売却/決済額[円]
    pub purchase_price: Option<f64>,                           // 平均取得価額[円]
    pub realized_profit_and_loss: Option<i32>,                 // 実現損益[円]
    pub total_realized_profit_and_loss: Option<i32>,           // 合計実現損益[円]
    pub total_taxes: Option<u32>,                              // 源泉徴収税額
    pub total_realized_profit_and_loss_after_tax: Option<i32>, // 損益
}

impl ReceiptProps for DomesticStock {
    fn new() -> Self {
        Self {
            trade_date: None,
            settlement_date: None,
            security_code: None,
            security_name: None,
            account: None,
            shares: None,
            asked_price: None,
            proceeds: None,
            purchase_price: None,
            realized_profit_and_loss: None,
            total_realized_profit_and_loss: None,
            total_taxes: None,
            total_realized_profit_and_loss_after_tax: None,
        }
    }

    fn new_summary(receipts: &[&Self]) -> Option<Self> {
        let (specific_account_total, nisa_account_total) = receipts
            .iter()
            .filter_map(|domestic_stock| {
                Some((
                    domestic_stock.account.as_deref()?,
                    domestic_stock.realized_profit_and_loss?,
                ))
            })
            .fold(
                (0, 0),
                |(specific, nisa), (account, realized_profit_and_loss)| {
                    if account.contains("特定") {
                        (specific + realized_profit_and_loss, nisa)
                    } else {
                        (specific, nisa + realized_profit_and_loss)
                    }
                },
            );

        let total_taxes = ((specific_account_total.max(0) as f64) * TAX_RATE) as u32;
        let total = specific_account_total + nisa_account_total;

        Some(Self {
            trade_date: None,
            settlement_date: None,
            security_code: None,
            security_name: None,
            account: None,
            shares: None,
            asked_price: None,
            proceeds: None,
            purchase_price: None,
            realized_profit_and_loss: None,
            total_realized_profit_and_loss: Some(total),
            total_taxes: Some(total_taxes),
            total_realized_profit_and_loss_after_tax: Some(total - total_taxes as i32),
        })
    }

    fn new_from_string_record(record: StringRecord) -> Self {
        Self {
            trade_date: record.get(0).try_parse_date(),
            settlement_date: record.get(1).try_parse_date(),
            security_code: record.get(2).try_parse_string(),
            security_name: record.get(3).try_parse_string(),
            account: record.get(4).try_parse_string(),
            shares: record.get(7).try_parse_num(),
            asked_price: record.get(8).try_parse_num(),
            proceeds: record.get(9).try_parse_num(),
            purchase_price: record.get(10).try_parse_num(),
            realized_profit_and_loss: record.get(11).try_parse_num(),
            total_realized_profit_and_loss: None,
            total_taxes: None,
            total_realized_profit_and_loss_after_tax: None,
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
            ("security_code", self.security_code.clone()),
            ("security_name", self.security_name.clone()),
            ("account", self.account.clone()),
            ("shares", self.shares.map(|s| s.to_string())),
            ("asked_price", self.asked_price.map(|p| p.to_string())),
            ("proceeds", self.proceeds.map(|p| p.to_string())),
            ("purchase_price", self.purchase_price.map(|p| p.to_string())),
            (
                "realized_profit_and_loss",
                self.realized_profit_and_loss.map(|p| p.to_string()),
            ),
            (
                "total_realized_profit_and_loss",
                self.total_realized_profit_and_loss.map(|p| p.to_string()),
            ),
            ("total_taxes", self.total_taxes.map(|p| p.to_string())),
            (
                "total_realized_profit_and_loss_after_tax",
                self.total_realized_profit_and_loss_after_tax
                    .map(|p| p.to_string()),
            ),
        ]
    }

    fn view_summary(receipts: &[Self]) -> Html {
        let (total_realized_profit_and_loss, total_taxes, total_realized_profit_and_loss_after_tax) =
            receipts
                .into_iter()
                .chunk_by(|receipt| receipt.get_date().expect("get_date error"))
                .into_iter()
                .filter_map(|(_, group)| Self::new_summary(&group.collect::<Vec<_>>()))
                .fold(
                    (0, 0, 0),
                    |(total_realized_profit_and_loss, withholding_tax, profit_and_loss), p| {
                        (
                            total_realized_profit_and_loss
                                + p.total_realized_profit_and_loss.unwrap_or(0),
                            withholding_tax + p.total_taxes.unwrap_or(0),
                            profit_and_loss
                                + p.total_realized_profit_and_loss_after_tax.unwrap_or(0),
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

    fn search(&self, query: &str) -> bool {
        self.security_code.as_deref().unwrap_or_default() == query
    }

    fn get_security_code(&self) -> &str {
        self.security_code.as_deref().unwrap_or_default()
    }

    fn get_security_name(&self) -> &str {
        self.security_name.as_deref().unwrap_or_default()
    }
}
