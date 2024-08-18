use crate::setting::HEADERS;

use super::{
    super::templete::{TemplateManager, TemplateStruct},
    profit_and_loss::ProfitAndLoss,
};
use anyhow::Result;
use chrono::NaiveDate;
use csv::StringRecord;
use std::{cell::RefCell, collections::BTreeMap};

pub struct ProfitAndLossManager {
    template_struct: TemplateStruct,
    profit_and_loss_map: RefCell<BTreeMap<NaiveDate, Vec<ProfitAndLoss>>>,
}

impl ProfitAndLossManager {
    pub fn new() -> Self {
        ProfitAndLossManager {
            template_struct: TemplateStruct::new(),
            profit_and_loss_map: RefCell::new(BTreeMap::new()),
        }
    }
}

impl TemplateManager for ProfitAndLossManager {
    fn set(&self, records: Vec<StringRecord>) {
        for record in records {
            let profit_and_loss = ProfitAndLoss::from_record(record);
            if let Some(trade_date) = profit_and_loss.trade_date {
                self.profit_and_loss_map
                    .borrow_mut()
                    .entry(trade_date)
                    .or_insert_with(Vec::new)
                    .push(profit_and_loss);
            }
        }
    }

    fn generate_html_table(&self) -> Result<String> {
        let mut table = "<table><thead><tr>".to_string();
        let headers = ProfitAndLoss::new().get_all_fields();

        for (header, _) in headers {
            let header = HEADERS.get(&header).unwrap();
            table.push_str(&format!("<th>{}</th>", header));
        }
        table.push_str("</tr></thead><tbody>");

        Ok(table)
    }
}
