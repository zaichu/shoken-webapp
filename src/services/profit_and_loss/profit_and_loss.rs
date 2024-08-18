use crate::{services::common, setting::TAX_RATE};
use chrono::NaiveDate;
use csv::StringRecord;

#[derive(Debug, Clone)]
pub struct ProfitAndLoss {
    pub trade_date: Option<NaiveDate>,               // 約定日
    pub settlement_date: Option<NaiveDate>,          // 受渡日
    pub security_code: Option<String>,               // 銘柄コード
    pub security_name: Option<String>,               // 銘柄名
    pub account: Option<String>,                     // 口座
    pub shares: Option<i32>,                         // 数量[株]
    pub asked_price: Option<f64>,                    // 売却/決済単価[円]
    pub proceeds: Option<i32>,                       // 売却/決済額[円]
    pub purchase_price: Option<f64>,                 // 平均取得価額[円]
    pub realized_profit_and_loss: Option<i32>,       // 実現損益[円]
    pub total_realized_profit_and_loss: Option<i32>, // 合計実現損益[円]
    pub withholding_tax: Option<u32>,                // 源泉徴収税額
    pub profit_and_loss: Option<i32>,                // 損益
}

impl ProfitAndLoss {
    pub fn new() -> Self {
        ProfitAndLoss {
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
            withholding_tax: None,
            profit_and_loss: None,
        }
    }

    pub fn from_record(record: StringRecord) -> Self {
        ProfitAndLoss {
            trade_date: common::parse_date(record.get(0)),
            settlement_date: common::parse_date(record.get(1)),
            security_code: common::parse_string(record.get(2)),
            security_name: common::parse_string(record.get(3)),
            account: common::parse_string(record.get(4)),
            shares: common::parse_int(record.get(7)),
            asked_price: common::parse_float(record.get(8)),
            proceeds: common::parse_int(record.get(9)),
            purchase_price: common::parse_float(record.get(10)),
            realized_profit_and_loss: common::parse_int(record.get(11)),
            total_realized_profit_and_loss: None,
            withholding_tax: None,
            profit_and_loss: None,
        }
    }

    pub fn get_all_fields(&self) -> Vec<(String, Option<String>)> {
        vec![
            (
                "trade_date".to_string(),
                self.trade_date.map(|d| d.to_string()),
            ),
            (
                "settlement_date".to_string(),
                self.settlement_date.map(|d| d.to_string()),
            ),
            ("security_code".to_string(), self.security_code.clone()),
            ("security_name".to_string(), self.security_name.clone()),
            ("account".to_string(), self.account.clone()),
            ("shares".to_string(), self.shares.map(|s| s.to_string())),
            (
                "asked_price".to_string(),
                self.asked_price.map(|p| p.to_string()),
            ),
            ("proceeds".to_string(), self.proceeds.map(|p| p.to_string())),
            (
                "purchase_price".to_string(),
                self.purchase_price.map(|p| p.to_string()),
            ),
            (
                "realized_profit_and_loss".to_string(),
                self.realized_profit_and_loss.map(|p| p.to_string()),
            ),
            (
                "total_realized_profit_and_loss".to_string(),
                self.total_realized_profit_and_loss.map(|p| p.to_string()),
            ),
            (
                "withholding_tax".to_string(),
                self.withholding_tax.map(|p| p.to_string()),
            ),
            (
                "profit_and_loss".to_string(),
                self.profit_and_loss.map(|p| p.to_string()),
            ),
        ]
    }

    pub fn new_total_realized_profit_and_loss(
        (specific_account_total, nisa_account_total): (i32, i32),
    ) -> Self {
        let withholding_tax = if specific_account_total < 0 {
            0
        } else {
            (specific_account_total as f64 * TAX_RATE) as u32
        };
        let total = specific_account_total + nisa_account_total;

        ProfitAndLoss {
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
            withholding_tax: Some(withholding_tax),
            profit_and_loss: Some(total - withholding_tax as i32),
        }
    }
}
