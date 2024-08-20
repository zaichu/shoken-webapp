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
        let row_content: String = values
            .iter()
            .map(|(key, value)| {
                format!(
                    "<td {}>{}</td>",
                    format_class(key),
                    value.as_deref().unwrap_or("")
                )
            })
            .collect();

        format!("<tr {tr_class}>{row_content}</tr>")
    }

    fn generate_table_row(&self, values: &[(String, Option<String>)]) -> String {
        self.generate_table_row_with_class("", values)
    }
}
