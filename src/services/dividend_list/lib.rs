use crate::setting::HEADERS;

use super::{
    super::templete::{TemplateManager, TemplateStruct},
    dividend_list::DividendList,
};
use anyhow::Result;
use chrono::NaiveDate;
use csv::StringRecord;
use std::{cell::RefCell, collections::BTreeMap};

pub struct DividendListManager {
    template_struct: TemplateStruct,
    dividend_list_map: RefCell<BTreeMap<NaiveDate, Vec<DividendList>>>,
}

impl DividendListManager {
    pub fn new() -> Self {
        DividendListManager {
            template_struct: TemplateStruct::new(),
            dividend_list_map: RefCell::new(BTreeMap::new()),
        }
    }
}

impl TemplateManager for DividendListManager {
    fn set(&self, records: Vec<StringRecord>) {
        todo!()
    }

    fn generate_html_table(&self) -> Result<String> {
        let mut table = "<table><thead><tr>".to_string();
        let headers = DividendList::new().get_all_fields();

        for (header, _) in headers {
            let header = HEADERS.get(&header).unwrap();
            table.push_str(&format!("<th>{}</th>", header));
        }
        table.push_str("</tr></thead><tbody>");

        Ok(table)
    }
}
