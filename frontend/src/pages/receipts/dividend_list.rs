use chrono::{Datelike, NaiveDate};
use csv::StringRecord;
use yew::prelude::*;

use super::receipt_template::ReceiptProps;
use crate::services::parser::*;

#[derive(PartialEq, Properties, Debug, Clone, Default)]
pub struct DividendList {
    pub settlement_date: Option<NaiveDate>,      // 入金日(受渡日)
    pub product: Option<String>,                 // 商品
    pub account: Option<String>,                 // 口座
    pub security_code: Option<String>,           // 銘柄コード
    pub security_name: Option<String>,           // 銘柄
    pub currency: Option<String>,                // 受取通貨
    pub unit_price: Option<String>,              // 単価[円/現地通貨]
    pub shares: Option<i32>,                     // 数量[株/口]
    pub dividends_before_tax: Option<i32>,       // 配当・分配金（税引前）[円/現地通貨]
    pub taxes: Option<i32>,                      // 税額[円/現地通貨]
    pub net_amount_received: Option<i32>,        // 受取金額[円/現地通貨]
    pub total_dividends_before_tax: Option<i32>, // 配当・分配金合計（税引前）[円/現地通貨]
    pub total_taxes: Option<i32>,                // 税額合計[円/現地通貨]
    pub total_net_amount_received: Option<i32>,  // 受取金額合計[円/現地通貨]
}

impl ReceiptProps for DividendList {
    fn new() -> Self {
        Self {
            settlement_date: None,
            product: None,
            account: None,
            security_code: None,
            security_name: None,
            currency: None,
            unit_price: None,
            shares: None,
            dividends_before_tax: None,
            taxes: None,
            net_amount_received: None,
            total_dividends_before_tax: None,
            total_taxes: None,
            total_net_amount_received: None,
        }
    }

    fn new_summary(receipts: &[&Self]) -> Option<Self> {
        let (total_dividends_before_tax, total_taxes, total_net_amount_received) =
            receipts.iter().fold(
                (0, 0, 0),
                |(total_dividends_before_tax, total_taxes, total_net_amount_received), dividend| {
                    if let (Some(dividends_before_tax), Some(taxes), Some(net_amount_received)) = (
                        dividend.dividends_before_tax,
                        dividend.taxes,
                        dividend.net_amount_received,
                    ) {
                        (
                            total_dividends_before_tax + dividends_before_tax,
                            total_taxes + taxes,
                            total_net_amount_received + net_amount_received,
                        )
                    } else {
                        (
                            total_dividends_before_tax,
                            total_taxes,
                            total_net_amount_received,
                        )
                    }
                },
            );

        Some(Self {
            settlement_date: None,
            product: None,
            account: None,
            security_code: None,
            security_name: None,
            currency: None,
            unit_price: None,
            shares: None,
            dividends_before_tax: None,
            taxes: None,
            net_amount_received: None,
            total_dividends_before_tax: Some(total_dividends_before_tax),
            total_taxes: Some(total_taxes),
            total_net_amount_received: Some(total_net_amount_received),
        })
    }

    fn new_from_string_record(record: StringRecord) -> Self {
        DividendList {
            settlement_date: record.get(0).try_parse_date(),
            product: record.get(1).try_parse_string(),
            account: record.get(2).try_parse_string(),
            security_code: record.get(3).try_parse_string(),
            security_name: record.get(4).try_parse_string(),
            currency: record.get(5).try_parse_string(),
            unit_price: record.get(6).try_parse_string(),
            shares: record.get(7).try_parse_num(),
            dividends_before_tax: record.get(8).try_parse_num(),
            taxes: record.get(9).try_parse_num(),
            net_amount_received: record.get(10).try_parse_num(),
            total_dividends_before_tax: None,
            total_taxes: None,
            total_net_amount_received: None,
        }
    }

    fn get_all_fields(&self) -> Vec<(&'static str, Option<String>)> {
        vec![
            (
                "settlement_date",
                self.settlement_date.map(|d| d.to_string()),
            ),
            ("product", self.product.clone()),
            ("account", self.account.clone()),
            ("security_code", self.security_code.clone()),
            ("security_name", self.security_name.clone()),
            // ("currency", self.currency.clone()),
            ("unit_price", self.unit_price.clone()),
            ("shares", self.shares.map(|s| s.to_string())),
            (
                "dividends_before_tax",
                self.dividends_before_tax.map(|t| t.to_string()),
            ),
            ("taxes", self.taxes.map(|t| t.to_string())),
            (
                "net_amount_received",
                self.net_amount_received.map(|t| t.to_string()),
            ),
            (
                "total_dividends_before_tax",
                self.total_dividends_before_tax.map(|t| t.to_string()),
            ),
            ("total_taxes", self.total_taxes.map(|t| t.to_string())),
            (
                "total_net_amount_received",
                self.total_net_amount_received.map(|t| t.to_string()),
            ),
        ]
    }

    fn get_date(&self) -> Option<NaiveDate> {
        let date = self.settlement_date.unwrap();
        NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
    }

    fn view_summary(receipts: &[Self]) -> Html {
        let (total_dividends_before_tax, total_taxes, total_net_amount_received) =
            receipts.iter().fold(
                (0, 0, 0),
                |(total_dividends_before_tax, total_taxes, total_net_amount_received), dividend| {
                    (
                        total_dividends_before_tax + dividend.dividends_before_tax.unwrap_or(0),
                        total_taxes + dividend.taxes.unwrap_or(0),
                        total_net_amount_received + dividend.net_amount_received.unwrap_or(0),
                    )
                },
            );

        html! {
            <tbody>
                <tr>
                    { Self::render_summary_th_td("total_dividends_before_tax", total_dividends_before_tax) }
                    { Self::render_summary_th_td("total_taxes", total_taxes) }
                    { Self::render_summary_th_td("total_net_amount_received", total_net_amount_received) }
                </tr>
            </tbody>
        }
    }

    fn search(&self, query: &str) -> bool {
        self.security_code.as_deref().unwrap_or_default() == query
            || self.security_name.as_deref().unwrap_or_default() == query
    }

    fn get_security_code(&self) -> &str {
        self.security_code.as_deref().unwrap_or_default()
    }

    fn get_security_name(&self) -> &str {
        self.security_name.as_deref().unwrap_or_default()
    }
}
