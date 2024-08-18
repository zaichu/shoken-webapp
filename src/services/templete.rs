use crate::setting::HEADERS;

use super::csv::lib::CSVAccessor;
use anyhow::Result;
use async_trait::async_trait;
use csv::StringRecord;

pub struct TemplateStruct;

impl TemplateStruct {
    pub fn new() -> TemplateStruct {
        TemplateStruct {}
    }
}

#[async_trait]
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
            let header = HEADERS.get(&header).unwrap();
            table.push_str(&format!("<th>{header}</th>"));
        }
        table.push_str("</tr></thead>");
        table
    }
}
