use super::{
    super::templete::{TemplateManager, TemplateStruct},
    profit_and_loss::ProfitAndLoss,
};
use anyhow::Result;
use chrono::NaiveDate;
use csv::StringRecord;
use std::{cell::RefCell, collections::BTreeMap};

pub struct ProfitAndLossManager {
    _template_struct: TemplateStruct,
    profit_and_loss_map: RefCell<BTreeMap<NaiveDate, Vec<ProfitAndLoss>>>,
}

impl ProfitAndLossManager {
    pub fn new() -> Self {
        ProfitAndLossManager {
            _template_struct: TemplateStruct::new(),
            profit_and_loss_map: RefCell::new(BTreeMap::new()),
        }
    }

    fn generate_table_tbody(&self, table: &mut String, profit_and_loss_list: &[ProfitAndLoss]) {
        let mut specific_account_total = 0;
        let mut nisa_account_total = 0;

        for profit_and_loss in profit_and_loss_list {
            table.push_str(&self.generate_table_row(&profit_and_loss.get_all_fields()));

            if let (Some(account), Some(realized_profit_and_loss)) = (
                profit_and_loss.account.as_deref(),
                profit_and_loss.realized_profit_and_loss,
            ) {
                if account.contains("特定") {
                    specific_account_total += realized_profit_and_loss;
                } else {
                    nisa_account_total += realized_profit_and_loss;
                }
            }
        }

        let total = (specific_account_total, nisa_account_total);
        let profit_and_loss = ProfitAndLoss::new_total_realized_profit_and_loss(total);

        table.push_str(
            &self.generate_table_row_with_class("group-total", &profit_and_loss.get_all_fields()),
        );

        table.push_str("</tr>");
    }
}

impl TemplateManager for ProfitAndLossManager {
    fn process_records(&self, records: Vec<StringRecord>) {
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
        let headers = ProfitAndLoss::new().get_all_fields();
        let mut table = self.generate_table_header(headers);

        table.push_str("<tbody>");
        for (_, profit_and_loss_list) in self.profit_and_loss_map.borrow().iter() {
            let _total = self.generate_table_tbody(&mut table, profit_and_loss_list);
        }
        table.push_str("</tbody></table>");

        Ok(table)
    }
}
