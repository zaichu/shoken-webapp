use chrono::NaiveDate;
use csv::StringRecord;
use std::error::Error;

#[derive(Debug, Clone)]
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

impl DividendList {
    pub fn new() -> Self {
        DividendList {
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

    pub fn new_total_dividend_list(
        (total_dividends_before_tax, total_taxes, total_net_amount_received): (i32, i32, i32),
    ) -> Self {
        DividendList {
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
        }
    }

    pub fn from_record(record: StringRecord) -> Result<Self, Box<dyn Error>> {
        Ok(DividendList {
            settlement_date: Self::parse_date(record.get(0))?,
            product: Self::parse_string(record.get(1))?,
            account: Self::parse_string(record.get(2))?,
            security_code: Self::parse_string(record.get(3))?,
            security_name: Self::parse_string(record.get(4))?,
            currency: Self::parse_string(record.get(5))?,
            unit_price: Self::parse_string(record.get(6))?,
            shares: Self::parse_int(record.get(7))?,
            dividends_before_tax: Self::parse_int(record.get(8))?,
            taxes: Self::parse_int(record.get(9))?,
            net_amount_received: Self::parse_int(record.get(10))?,
            total_dividends_before_tax: None,
            total_taxes: None,
            total_net_amount_received: None,
        })
    }

    pub fn get_all_fields(&self) -> Vec<(String, Option<String>)> {
        vec![
            (
                "settlement_date".to_string(),
                self.settlement_date.map(|d| d.to_string()),
            ),
            ("product".to_string(), self.product.clone()),
            ("account".to_string(), self.account.clone()),
            ("security_code".to_string(), self.security_code.clone()),
            ("security_name".to_string(), self.security_name.clone()),
            ("currency".to_string(), self.currency.clone()),
            ("unit_price".to_string(), self.unit_price.clone()),
            ("shares".to_string(), self.shares.map(|s| s.to_string())),
            (
                "dividends_before_tax".to_string(),
                self.dividends_before_tax.map(|t| t.to_string()),
            ),
            ("taxes".to_string(), self.taxes.map(|t| t.to_string())),
            (
                "net_amount_received".to_string(),
                self.net_amount_received.map(|n| n.to_string()),
            ),
            (
                "total_dividends_before_tax".to_string(),
                self.total_dividends_before_tax.map(|t| t.to_string()),
            ),
            (
                "total_taxes".to_string(),
                self.total_taxes.map(|t| t.to_string()),
            ),
            (
                "total_net_amount_received".to_string(),
                self.total_net_amount_received.map(|n| n.to_string()),
            ),
        ]
    }

    fn parse_date(date_str: Option<&str>) -> Result<Option<NaiveDate>, Box<dyn Error>> {
        date_str.map_or(Ok(None), |s| {
            NaiveDate::parse_from_str(&s.replace("/", "-"), "%Y-%m-%d")
                .map(Some)
                .map_err(|e| format!("Failed to parse date '{s}': {e}").into())
        })
    }

    fn parse_int(num_str: Option<&str>) -> Result<Option<i32>, Box<dyn Error>> {
        num_str.map_or(Ok(None), |s| {
            s.replace(",", "")
                .parse::<i32>()
                .map(Some)
                .map_err(|e| format!("Failed to parse integer '{s}': {e}").into())
        })
    }

    fn parse_string(value: Option<&str>) -> Result<Option<String>, Box<dyn Error>> {
        Ok(value.map(|s| s.to_string()))
    }
}
