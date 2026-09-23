use crate::receipts::{ReceiptItem, ReceiptsTab};
use crate::receipts_search::{
    create_year_options, filter_by_config, get_unique_values, FilterConfig, SearchOption,
};
use crate::receipts_search_support::{
    create_search_options, reorder_columns_by_search, ColumnReorderRule,
};
use rust_decimal::Decimal;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReceiptSearch {
    pub query: String,
    pub selections: [String; 4],
}
impl ReceiptSearch {
    pub fn select(&mut self, index: usize, value: String) {
        self.selections[index] = value;
        self.query = [0, 3, 1, 2]
            .into_iter()
            .filter_map(|i| {
                let value =
                    self.selections[i].trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
                if value.is_empty() {
                    None
                } else if value.chars().any(char::is_whitespace) {
                    Some(format!("\"{}\"", value.replace('"', "\\\"")))
                } else {
                    Some(value.to_string())
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchCategories {
    pub securities: Vec<SearchOption>,
    pub products: Vec<SearchOption>,
    pub accounts: Vec<SearchOption>,
    pub years: Vec<SearchOption>,
}

impl ReceiptItem {
    pub fn date(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.settlement_date,
            Self::DomesticStock(r) => &r.trade_date,
            Self::MutualFund(r) => &r.trade_date,
        }
    }
    pub fn code(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.security_code,
            Self::DomesticStock(r) => &r.security_code,
            Self::MutualFund(_) => "",
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.security_name,
            Self::DomesticStock(r) => &r.security_name,
            Self::MutualFund(r) => &r.fund_name,
        }
    }
    pub fn account(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.account,
            Self::DomesticStock(r) => &r.account,
            Self::MutualFund(r) => &r.account,
        }
    }
    pub fn product(&self) -> &str {
        match self {
            Self::Dividend(r) => &r.product,
            _ => "",
        }
    }
}

pub fn filter_receipts(tab: ReceiptsTab, rows: &[ReceiptItem], query: &str) -> Vec<ReceiptItem> {
    let mut config = FilterConfig {
        string_fields: Some(vec![
            ReceiptItem::code,
            ReceiptItem::name,
            ReceiptItem::account,
        ]),
        partial_string_fields: None,
        date_field: Some(ReceiptItem::date),
        year_search: true,
        year_month_search: true,
        date_search: true,
        date_range_search: true,
        amount_fields: None,
    };
    match tab {
        ReceiptsTab::Dividend => {
            config
                .string_fields
                .as_mut()
                .unwrap()
                .push(ReceiptItem::product);
            config.amount_fields = Some(vec![
                |r| match r {
                    ReceiptItem::Dividend(r) => r.unit_price,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::Dividend(r) => r.shares,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::Dividend(r) => r.dividends_before_tax,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::Dividend(r) => r.taxes,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::Dividend(r) => r.net_amount_received,
                    _ => Decimal::ZERO,
                },
            ]);
        }
        ReceiptsTab::DomesticStock => {
            config.amount_fields = Some(vec![
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.shares,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.asked_price,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.proceeds,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.purchase_price,
                    _ => Decimal::ZERO,
                },
                |r| match r {
                    ReceiptItem::DomesticStock(r) => r.realized_profit_and_loss,
                    _ => Decimal::ZERO,
                },
            ]);
        }
        ReceiptsTab::MutualFund => {
            config.string_fields = Some(vec![ReceiptItem::name, ReceiptItem::account]);
        }
    }
    filter_by_config(rows, query, &config)
        .into_iter()
        .cloned()
        .collect()
}

pub fn search_categories(tab: ReceiptsTab, rows: &[ReceiptItem]) -> SearchCategories {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| b.date().cmp(a.date()));
    let to_options = |values: Vec<String>| {
        values
            .into_iter()
            .map(|value| SearchOption {
                label: value.clone(),
                value,
            })
            .collect()
    };
    SearchCategories {
        securities: create_search_options(
            &sorted,
            ReceiptItem::code,
            ReceiptItem::name,
            true,
            Some(ReceiptItem::date),
        ),
        products: if tab == ReceiptsTab::Dividend {
            to_options(get_unique_values(&sorted, ReceiptItem::product))
        } else {
            vec![]
        },
        accounts: if tab != ReceiptsTab::MutualFund {
            to_options(get_unique_values(&sorted, ReceiptItem::account))
        } else {
            vec![]
        },
        years: create_year_options(&sorted, ReceiptItem::date),
    }
}

pub fn column_order(tab: ReceiptsTab, rows: &[ReceiptItem], query: &str) -> Vec<usize> {
    let base: Vec<_> = (0..if tab == ReceiptsTab::DomesticStock {
        11
    } else {
        10
    })
        .collect();
    let account = |r: &ReceiptItem, q: &str| r.account().to_lowercase().contains(q);
    match tab {
        ReceiptsTab::Dividend => reorder_columns_by_search(
            &base,
            rows,
            query,
            &[
                ColumnReorderRule {
                    column_key: 1,
                    matches: |r: &ReceiptItem, q| r.product().to_lowercase().contains(q),
                },
                ColumnReorderRule {
                    column_key: 2,
                    matches: account,
                },
            ],
            1,
        ),
        ReceiptsTab::DomesticStock => reorder_columns_by_search(
            &base,
            rows,
            query,
            &[ColumnReorderRule {
                column_key: 3,
                matches: account,
            }],
            2,
        ),
        ReceiptsTab::MutualFund => base,
    }
}

#[cfg(test)]
pub(crate) mod tests;
