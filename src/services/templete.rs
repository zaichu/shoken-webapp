use super::csv::lib::CSVAccessor;
use crate::setting::HEADERS;
use anyhow::Result;
use csv::StringRecord;

pub struct TemplateStruct;

impl TemplateStruct {
    pub fn new() -> TemplateStruct {
        TemplateStruct {}
    }
}

pub trait TemplateManager {
    fn execute(&self, bytes: Vec<u8>) -> Result<String> {
        let records = self.read_csv_data(bytes)?;
        self.process_records(records);
        self.generate_html_table()
    }

    fn read_csv_data(&self, bytes: Vec<u8>) -> Result<Vec<StringRecord>> {
        CSVAccessor::read(bytes)
    }

    fn process_records(&self, records: Vec<StringRecord>);
    fn generate_html_table(&self) -> Result<String>;

    fn generate_table_header(&self, headers: Vec<(String, Option<String>)>) -> String {
        let mut table = "<table><thead><tr>".to_string();
        for (header, _) in headers {
            let header_name = HEADERS.get(&header).unwrap();
            table.push_str(&format!("<th class=\"{header}\">{header_name}</th>"));
        }
        table.push_str("</tr></thead>");
        table
    }

    fn generate_table_row_with_class(
        &self,
        tr_class: &str,
        values: &[(String, Option<String>)],
    ) -> String {
        let format_class = |class: &str| {
            if class.is_empty() {
                String::new()
            } else {
                format!("class=\"{class}\"")
            }
        };

        let tr_class = format_class(tr_class);
        let row_content = values
            .iter()
            .map(|(key, value)| {
                let value = value.as_deref().unwrap_or("");
                let value = self.format_value(key, value);
                let key = if value.starts_with("-") {
                    format!("{key} negative")
                } else {
                    key.to_string()
                };
                format!("<td {}>{value}</td>", format_class(&key))
            })
            .collect::<String>();

        format!("<tr {tr_class}>{row_content}</tr>")
    }

    fn format_value(&self, key: &str, value: &str) -> String {
        match key {
            "settlement_date" | "trade_date" => self.format_date(value),
            "asked_price"
            | "dividends_before_tax"
            | "net_amount_received"
            | "proceeds"
            | "profit_and_loss"
            | "purchase_price"
            | "realized_profit_and_loss"
            | "shares"
            | "taxes"
            | "total_dividends_before_tax"
            | "total_net_amount_received"
            | "total_realized_profit_and_loss"
            | "total_taxes"
            | "withholding_tax" => self.format_number(value),
            _ => value.to_string(),
        }
    }

    fn generate_table_row(&self, values: &[(String, Option<String>)]) -> String {
        self.generate_table_row_with_class("", values)
    }

    fn format_date(&self, s: &str) -> String {
        s.replace("-", "/")
    }

    fn format_number(&self, s: &str) -> String {
        let is_negative = s.starts_with('-');
        let s = if is_negative { &s[1..] } else { s };
        let s = s.replace(",", ""); // カンマを除去
        let parts: Vec<&str> = s.split('.').collect();
        let mut result = String::new();
        for (i, c) in parts[0].chars().rev().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push(',');
            }
            result.push(c);
        }
        result = result.chars().rev().collect();
        if parts.len() > 1 {
            result.push('.');
            result.push_str(parts[1]);
        }
        if is_negative {
            format!("-{result}")
        } else {
            result
        }
    }
}
