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
        let records = self.get(bytes)?;
        self.set(records);
        self.generate_html_table()
    }

    fn get(&self, bytes: Vec<u8>) -> Result<Vec<StringRecord>> {
        CSVAccessor::read(bytes)
    }

    fn set(&self, records: Vec<StringRecord>);
    fn generate_html_table(&self) -> Result<String>;
}
