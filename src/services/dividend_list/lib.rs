use super::{
    super::templete::{TemplateManager, TemplateStruct},
    dividend_list::DividendList,
};
use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use csv::StringRecord;
use std::{cell::RefCell, collections::BTreeMap};

pub struct DividendListManager {
    _template_struct: TemplateStruct,
    dividend_list_map: RefCell<BTreeMap<NaiveDate, Vec<DividendList>>>,
}

impl DividendListManager {
    pub fn new() -> Self {
        DividendListManager {
            _template_struct: TemplateStruct::new(),
            dividend_list_map: RefCell::new(BTreeMap::new()),
        }
    }

    fn generate_table_tbody(&self, table: &mut String, dividend_list: &[DividendList]) {
        let mut total_dividends_before_tax = 0; // 配当・分配金合計（税引前）[円/現地通貨]
        let mut total_taxes = 0; // 税額合計[円/現地通貨]
        let mut total_net_amount_received = 0; // 受取金額[円/現地通貨]

        for dividend in dividend_list {
            table.push_str(&self.generate_table_row(&dividend.get_all_fields()));

            if let (Some(dividends_before_tax), Some(taxes), Some(net_amount_received)) = (
                dividend.dividends_before_tax,
                dividend.taxes,
                dividend.net_amount_received,
            ) {
                total_dividends_before_tax += dividends_before_tax;
                total_taxes += taxes;
                total_net_amount_received += net_amount_received;
            }
        }

        let total = (
            total_dividends_before_tax,
            total_taxes,
            total_net_amount_received,
        );
        let dividend_list = DividendList::new_total_dividend_list(total);
        table.push_str(
            &self.generate_table_row_with_class("group-total", &dividend_list.get_all_fields()),
        );
    }
}

impl TemplateManager for DividendListManager {
    fn process_records(&self, records: Vec<StringRecord>) {
        for record in records {
            let dividend = DividendList::from_record(record);
            if let Some(settlement_date) = dividend.settlement_date {
                let date =
                    NaiveDate::from_ymd_opt(settlement_date.year(), settlement_date.month(), 1)
                        .unwrap();
                self.dividend_list_map
                    .borrow_mut()
                    .entry(date)
                    .or_insert_with(Vec::new)
                    .push(dividend);
            }
        }
    }

    fn generate_html_table(&self) -> Result<String> {
        let headers = DividendList::new().get_all_fields();
        let mut table = self.generate_table_header(headers);

        table.push_str("<tbody>");
        for (_, dividend_list) in self.dividend_list_map.borrow().iter() {
            let _total = self.generate_table_tbody(&mut table, dividend_list);
        }
        table.push_str("</tbody></table>");

        Ok(table)
    }
}
