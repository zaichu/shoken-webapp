use crate::dto::{Dividend, Mutualfund};
use crate::receipts::search_group_key::{create_group_key_fn, GroupKeyRule};
use crate::receipts::search_support::group_and_summarize;
use crate::receipts::{ReceiptCell, ReceiptItem, ReceiptsTab};
use crate::receipts_domain::{
    calculate_domestic_daily, create_year_month_key, format_currency, sort_dividends,
    sort_domestic_stocks, sort_mutual_funds,
};
use std::collections::HashMap;

pub(crate) struct TableGroup {
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) summary: Vec<String>,
    pub(crate) rows: Vec<(String, String, Vec<ReceiptCell>)>,
}

pub(crate) fn group_label(key: &str) -> String {
    let is_date = (key.len() == 7 || key.len() == 10)
        && key.bytes().enumerate().all(|(i, b)| {
            if i == 4 || i == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        });
    if !is_date {
        return key.to_string();
    }
    let parts: Vec<_> = key.split('-').collect();
    match parts.as_slice() {
        [year, month, day] => format!(
            "{year}年{}月{}日",
            month.parse::<u32>().unwrap_or(0),
            day.parse::<u32>().unwrap_or(0)
        ),
        [year, month] => format!("{year}年{}月", month.parse::<u32>().unwrap_or(0)),
        _ => key.to_string(),
    }
}

pub(crate) fn table_groups(
    tab: ReceiptsTab,
    rows: &[ReceiptItem],
    all_rows: &[ReceiptItem],
    query: &str,
) -> Vec<TableGroup> {
    match tab {
        ReceiptsTab::Dividend => {
            let typed: Vec<_> = rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::Dividend(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let sorted = sort_dividends(&typed);
            let mut latest: HashMap<&str, &Dividend> = HashMap::new();
            for item in all_rows {
                if let ReceiptItem::Dividend(row) = item {
                    let current = latest.entry(&row.security_code).or_insert(row);
                    if row.settlement_date > current.settlement_date {
                        *current = row;
                    }
                }
            }
            let security_key = |row: &Dividend| {
                latest
                    .get(row.security_code.as_str())
                    .map(|r| r.security_name.clone())
                    .unwrap_or_else(|| row.security_name.clone())
            };
            let rules = [
                GroupKeyRule {
                    test: |r: &Dividend, t| {
                        r.security_code.to_lowercase() == t || r.security_name.to_lowercase() == t
                    },
                    key_fn: &security_key,
                },
                GroupKeyRule {
                    test: |r: &Dividend, t| r.product.to_lowercase() == t,
                    key_fn: &|r: &Dividend| r.product.clone(),
                },
                GroupKeyRule {
                    test: |r: &Dividend, t| r.account.to_lowercase() == t,
                    key_fn: &|r: &Dividend| r.account.clone(),
                },
            ];
            let key =
                create_group_key_fn(query, |r| create_year_month_key(&r.settlement_date), &rules);
            group_and_summarize(
                &sorted,
                &key,
                &[
                    |r| r.dividends_before_tax,
                    |r| r.taxes,
                    |r| r.net_amount_received,
                ],
                true,
            )
            .into_iter()
            .map(|summary| {
                let group_rows = sorted
                    .iter()
                    .filter(|row| key(row) == summary.filter)
                    .cloned()
                    .map(ReceiptItem::Dividend)
                    .map(|item| (item.id().to_string(), item.raw_key(), item.cells()))
                    .collect();
                TableGroup {
                    key: summary.filter.clone(),
                    label: group_label(&summary.filter),
                    summary: vec![
                        format_currency(summary.values[0]),
                        format_currency(summary.values[1]),
                        format_currency(summary.values[2]),
                    ],
                    rows: group_rows,
                }
            })
            .collect()
        }
        ReceiptsTab::DomesticStock => {
            let typed: Vec<_> = rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::DomesticStock(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let sorted = sort_domestic_stocks(&typed);
            calculate_domestic_daily(&sorted)
                .into_iter()
                .map(|summary| {
                    let group_rows = sorted
                        .iter()
                        .filter(|row| row.trade_date == summary.filter)
                        .cloned()
                        .map(ReceiptItem::DomesticStock)
                        .map(|item| (item.id().to_string(), item.raw_key(), item.cells()))
                        .collect();
                    TableGroup {
                        key: summary.filter.clone(),
                        label: group_label(&summary.filter),
                        summary: vec![
                            format_currency(summary.total_realized_profit_and_loss),
                            format_currency(summary.total_taxes),
                            format_currency(summary.total_realized_profit_and_loss_after_tax),
                        ],
                        rows: group_rows,
                    }
                })
                .collect()
        }
        ReceiptsTab::MutualFund => {
            let typed: Vec<_> = rows
                .iter()
                .filter_map(|item| match item {
                    ReceiptItem::MutualFund(row) => Some(row.clone()),
                    _ => None,
                })
                .collect();
            let sorted = sort_mutual_funds(&typed);
            let rules = [GroupKeyRule {
                test: |r: &Mutualfund, t| r.fund_name.to_lowercase().contains(t),
                key_fn: &|r: &Mutualfund| r.fund_name.clone(),
            }];
            let key = create_group_key_fn(query, |r| create_year_month_key(&r.trade_date), &rules);
            group_and_summarize(
                &sorted,
                &key,
                &[
                    |r| r.realized_profit_and_loss,
                    |r| r.taxes,
                    |r| r.realized_profit_and_loss_after_tax,
                ],
                true,
            )
            .into_iter()
            .map(|summary| {
                let group_rows = sorted
                    .iter()
                    .filter(|row| key(row) == summary.filter)
                    .cloned()
                    .map(ReceiptItem::MutualFund)
                    .map(|item| (item.id().to_string(), item.raw_key(), item.cells()))
                    .collect();
                TableGroup {
                    key: summary.filter.clone(),
                    label: group_label(&summary.filter),
                    summary: vec![
                        format_currency(summary.values[0]),
                        format_currency(summary.values[1]),
                        format_currency(summary.values[2]),
                    ],
                    rows: group_rows,
                }
            })
            .collect()
        }
    }
}
