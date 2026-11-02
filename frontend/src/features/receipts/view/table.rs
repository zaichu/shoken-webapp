use super::TAB_IDS;
use super::cards::{
    CardRowData, CompactReceiptGroup, MobileCardGroup, amount_kind_for_label, card_row_data,
    is_negative_labeled_value, is_positive_labeled_value, signed_text_for_profit,
};
use crate::features::receipts::filter::{column_order, promoted_column};
use crate::features::receipts::kind::CardRole;
use crate::features::receipts::kind::ColumnSpec;
use crate::features::receipts::kind::ColumnTier;
use crate::features::receipts::kind::TableGroup;
use crate::features::receipts::{ReceiptCell, ReceiptRow, ReceiptsTab};
use crate::ui::amount::Amount;
use crate::ui::badge::AccountBadge;
use crate::ui::labels::{
    self, ACCOUNT, ACQUISITION_PRICE, AFTER_TAX, DIVIDEND_AMOUNT, FUND_NAME, PAYMENT_DATE,
    QUANTITY, REALIZED_PROFIT_LOSS, REDEMPTION_AMOUNT, SECURITY_CODE, SECURITY_NAME,
    SELLING_AMOUNT, TAX, TRADE_DATE,
};
use crate::ui::security_link::{CopyableInstrumentName, TableSecurityCode};
use leptos::prelude::*;
use std::collections::HashSet;

pub fn displayed_tiers(
    tab: ReceiptsTab,
    order: &[usize],
    promoted: Option<usize>,
) -> Vec<ColumnTier> {
    order
        .iter()
        .map(|&column| {
            if promoted == Some(column) {
                ColumnTier::Core
            } else {
                tab.columns()[column].tier
            }
        })
        .collect()
}

// PC の表は lg 以上でしか出ないため、段は「常時」と「xl から」の2つにまとめる。
// 口座は lg でも出す(この幅ではモバイル行も出ないため口座が見えなくなる)。
// 売買の内訳金額は xl まで隠し、lg でも銘柄列の幅を確保する
fn compact_tier(header: &str) -> ColumnTier {
    match header {
        s if s == SELLING_AMOUNT || s == REDEMPTION_AMOUNT || s == ACQUISITION_PRICE => {
            ColumnTier::Wide
        }
        _ => ColumnTier::Core,
    }
}

fn compact_tiers(
    order: &[usize],
    promoted: Option<usize>,
    columns: &[ColumnSpec],
) -> Vec<ColumnTier> {
    order
        .iter()
        .map(|&column| {
            if promoted == Some(column) {
                ColumnTier::Core
            } else {
                compact_tier(columns[column].header)
            }
        })
        .collect()
}

#[component]
pub fn ReceiptTable(
    tab: ReceiptsTab,
    rows: Vec<ReceiptRow>,
    all_rows: Vec<ReceiptRow>,
    query: String,
    expanded_ids: RwSignal<HashSet<String>>,
    #[prop(optional)] compact_mobile: bool,
    #[prop(optional)] sort_order: Option<crate::features::receipts::filter::ReceiptSortOrder>,
) -> impl IntoView {
    let columns = tab.columns();
    let order = sort_order.unwrap_or_default();
    let groups: Vec<TableGroup> = tab.table_groups(&rows, &all_rows, &query, order);
    let order = column_order(tab, &rows, &query);
    let promoted = promoted_column(tab, &rows, &query);
    let fields = tab.card_fields();
    let labels = tab.summary_labels();
    let slug = TAB_IDS[tab as usize];
    let card_groups: Vec<_> = groups
        .iter()
        .enumerate()
        .map(|(group_index, group)| {
            let summary: Vec<(&'static str, String)> = labels
                .iter()
                .copied()
                .zip(group.summary.iter().cloned())
                .collect();
            let cards: Vec<CardRowData> = group
                .rows
                .iter()
                .map(|(_, _, cells)| card_row_data(tab, cells, &order, fields))
                .collect();
            (
                group_index,
                group.key.clone(),
                group.label.clone(),
                group.rows.len(),
                summary,
                cards,
            )
        })
        .collect();
    let desktop_order: Vec<usize> = if compact_mobile {
        columns
            .iter()
            .enumerate()
            .filter_map(|(index, column)| {
                let visible = match tab {
                    ReceiptsTab::Dividend => [
                        PAYMENT_DATE,
                        SECURITY_CODE,
                        SECURITY_NAME,
                        ACCOUNT,
                        QUANTITY,
                        DIVIDEND_AMOUNT,
                        TAX,
                        AFTER_TAX,
                    ]
                    .contains(&column.header),
                    ReceiptsTab::DomesticStock => [
                        TRADE_DATE,
                        SECURITY_CODE,
                        SECURITY_NAME,
                        ACCOUNT,
                        QUANTITY,
                        SELLING_AMOUNT,
                        ACQUISITION_PRICE,
                        REALIZED_PROFIT_LOSS,
                        TAX,
                        AFTER_TAX,
                    ]
                    .contains(&column.header),
                    ReceiptsTab::MutualFund => [
                        TRADE_DATE,
                        FUND_NAME,
                        ACCOUNT,
                        QUANTITY,
                        REDEMPTION_AMOUNT,
                        ACQUISITION_PRICE,
                        REALIZED_PROFIT_LOSS,
                        TAX,
                        AFTER_TAX,
                    ]
                    .contains(&column.header),
                };
                visible.then_some(index)
            })
            .collect()
    } else {
        order.clone()
    };
    let headers: Vec<_> = desktop_order.iter().map(|i| columns[*i].header).collect();
    let tiers = if compact_mobile {
        compact_tiers(&desktop_order, promoted, columns)
    } else {
        displayed_tiers(tab, &desktop_order, promoted)
    };
    let cell_classes: Vec<String> = desktop_order
        .iter()
        .zip(tiers.iter())
        .map(|(i, tier)| format!("{}{}", columns[*i].align.class(), tier.class()))
        .collect();
    let span_masks: [(&'static str, ColumnTier); 5] = [
        (" receipt-span-sm", ColumnTier::Core),
        (" receipt-span-mid", ColumnTier::Md),
        (" receipt-span-lg", ColumnTier::Core),
        (" receipt-span-xl", ColumnTier::Wide),
        (" receipt-span-all", ColumnTier::Wider),
    ];
    let group_label_spans: Vec<(&'static str, usize)> = span_masks
        .into_iter()
        .map(|(class, max)| {
            let visible = tiers.iter().filter(|tier| **tier <= max).count();
            (class, visible.saturating_sub(labels.len()))
        })
        .collect();
    let surface_class = if compact_mobile {
        "receipt-list-surface"
    } else {
        "table-card"
    };
    // 印刷は幅に関係なく表を出し、行リストは出さない
    let desktop_class = if compact_mobile {
        "hidden lg:block print:block"
    } else {
        "hidden sm:block print:block"
    };
    let mobile_class = if compact_mobile {
        "lg:hidden print:hidden"
    } else {
        "sm:hidden print:hidden"
    };
    let table_frame_class = if compact_mobile {
        ""
    } else {
        "table-frame overflow-clip"
    };
    let table_class = if compact_mobile {
        "receipts-table"
    } else {
        "receipt-table"
    };
    view! {
        <div class=surface_class data-testid="receipt-table-card">
            <div class="p-0" data-testid="receipt-table-body">
                <div class=desktop_class>
                    <div class=table_frame_class>
                        <table class=table_class>
                            <thead>
                                <tr>
                                    {headers
                                        .iter()
                                        .zip(tiers.iter())
                                        .enumerate()
                                        .map(|(col_index, (header, tier))| {
                                            let align =
                                                columns[desktop_order[col_index]].align.class();
                                            let col_role = columns[desktop_order[col_index]].card_role;
                                            let extra_class = match col_role {
                                                CardRole::Name => " receipt-instrument-col",
                                                CardRole::Account => " receipt-account-col",
                                                _ => "",
                                            };
                                            view! {
                                                <th
                                                    class=format!("{align}{}{extra_class}", tier.class())
                                                    scope="col"
                                                    title=*header
                                                >
                                                    {*header}
                                                </th>
                                            }
                                        })
                                        .collect_view()}
                                </tr>
                            </thead>
                            <tbody>
                                {groups
                                    .iter()
                                    .enumerate()
                                    .map(|(group_index, group)| {
                                        let count = group.rows.len();
                                        let top = if group_index > 0 {
                                            if compact_mobile { "" } else { " border-t-2 border-border-strong" }
                                        } else {
                                            ""
                                        };
                                        let group_class = if compact_mobile {
                                            "receipts-table-group"
                                        } else {
                                            "whitespace-normal bg-surface-raised text-text-soft font-semibold border-l-4 border-accent"
                                        };
                                        view! {
                                            <tr>
                                                {group_label_spans
                                                    .iter()
                                                    .map(|(class, span)| {
                                                        let label_class = if compact_mobile {
                                                            " receipts-table-group-label"
                                                        } else {
                                                            ""
                                                        };
                                                        view! {
                                                            <td
                                                                colspan=*span + if group.summary.is_empty() { labels.len() } else { 0 }
                                                                class=format!(
                                                                    "col-text {group_class}{top}{class}{label_class}"
                                                                )
                                                            >
                                                                <span class="font-bold">{group.label.clone()}</span>
                                                                <span class="ml-2 font-normal">
                                                                    {format!("· {}", labels::count(count))}
                                                                </span>
                                                            </td>
                                                        }
                                                    })
                                                    .collect_view()}
                                                {group
                                                    .summary
                                                    .iter()
                                                    .zip(labels.iter())
                                                    .map(|(value, label)| {
                                                        let negative = is_negative_labeled_value(
                                                            tab, label, value,
                                                        );
                                                        let positive = is_positive_labeled_value(
                                                            tab, label, value,
                                                        );
                                                        let text =
                                                            signed_text_for_profit(tab, label, value);
                                                        let kind = amount_kind_for_label(
                                                            tab, label, value,
                                                        );
                                                        let bold = compact_mobile
                                                            && *label == AFTER_TAX;
                                                        view! {
                                                            <td
                                                                class=format!(
                                                                    "col-num whitespace-nowrap {}{}{top}",
                                                                    if compact_mobile { "receipts-table-group" } else { "bg-surface-raised text-text font-semibold" },
                                                                    if bold { " font-bold" } else { "" },
                                                                )
                                                                title=text.clone()
                                                                data-negative=negative.then_some("true")
                                                            >
                                                                <Amount
                                                                    kind=kind
                                                                    text=text.clone()
                                                                    negative=negative
                                                                    positive=positive
                                                                />
                                                            </td>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </tr>
                                            {group
                                                .rows
                                                .iter()
                                                .map(|(_, _, cells)| {
                                                    let cells: Vec<_> = desktop_order.iter().map(|i| cells[*i].clone()).collect();
                                                    view! {
                                                        <tr>
                                                            {cells
                                                                .into_iter()
                                                                .enumerate()
                                                                .map(|(col_index, cell)| {
                                                                    let align = cell_classes[col_index].clone();
                                                                    match cell {
                                                                        ReceiptCell::SecurityCode(code) => {
                                                                            let title = code.clone();
                                                                            view! {
                                                                                <td class=align title=title>
                                                                                    <TableSecurityCode code=code />
                                                                                </td>
                                                                            }
                                                                            .into_any()
                                                                        },
                                                                        ReceiptCell::InstrumentName { name, code } => {
                                                                            let title = name.clone();
                                                                            let code = code.unwrap_or_default();
                                                                            view! {
                                                                                <td
                                                                                    class=format!("{align} receipt-instrument-cell")
                                                                                    title=title
                                                                                >
                                                                                    <CopyableInstrumentName name=name code=code />
                                                                                </td>
                                                                            }
                                                                            .into_any()
                                                                        }
                                                                        ReceiptCell::Text(value) => {
                                                                            let header = columns[desktop_order[col_index]].header;
                                                                            let negative = is_negative_labeled_value(
                                                                                tab,
                                                                                header,
                                                                                &value,
                                                                            );
                                                                            let positive = is_positive_labeled_value(
                                                                                tab,
                                                                                header,
                                                                                &value,
                                                                            );
                                                                            let text = signed_text_for_profit(
                                                                                tab, header, &value,
                                                                            );
                                                                            let kind = amount_kind_for_label(
                                                                                tab, header, &value,
                                                                            );
                                                                            let is_trio = [
                                                                                DIVIDEND_AMOUNT,
                                                                                REALIZED_PROFIT_LOSS,
                                                                                TAX,
                                                                                AFTER_TAX,
                                                                            ]
                                                                            .contains(&header);
                                                                            let is_net = compact_mobile
                                                                                && header == AFTER_TAX;
                                                                            let bold_class = if is_net {
                                                                                " font-bold"
                                                                            } else {
                                                                                ""
                                                                            };
                                                                            // 対象の3列以外は従来のまま(日付・数量は薄色)
                                                                            let muted_class = if !is_trio
                                                                                && compact_mobile
                                                                                && [
                                                                                    PAYMENT_DATE,
                                                                                    TRADE_DATE,
                                                                                    QUANTITY,
                                                                                ]
                                                                                .contains(&header) {
                                                                                " text-text-soft"
                                                                            } else {
                                                                                ""
                                                                            };
                                                                            let title = text.clone();
                                                                            let content = if header == ACCOUNT {
                                                                                view! { <AccountBadge label=value.clone() /> }
                                                                                    .into_any()
                                                                            } else if is_trio {
                                                                                view! {
                                                                                    <Amount
                                                                                        kind=kind
                                                                                        text=text.clone()
                                                                                        negative=negative
                                                                                        positive=positive
                                                                                    />
                                                                                }
                                                                                    .into_any()
                                                                            } else {
                                                                                text.clone().into_any()
                                                                            };
                                                                            let account_class = if header == ACCOUNT {
                                                                                " receipt-account-col"
                                                                            } else {
                                                                                ""
                                                                            };
                                                                            view! {
                                                                                <td
                                                                                    class=format!(
                                                                                        "{align}{bold_class}{muted_class}{account_class}",
                                                                                    )
                                                                                    title=title
                                                                                    data-negative=negative.then_some("true")
                                                                                >
                                                                                    {content}
                                                                                </td>
                                                                            }
                                                                            .into_any()
                                                                        }
                                                                    }
                                                                })
                                                                .collect_view()}
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()}
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    </div>
                </div>
                <div class=mobile_class data-testid="receipt-card-list">
                    {if compact_mobile {
                        view! {
                            <div>
                                {card_groups
                                    .into_iter()
                                    .map(|(_, _, label, count, summary, cards)| {
                                        view! {
                                            <CompactReceiptGroup
                                                tab=tab
                                                label=label
                                                count=count
                                                summary=summary
                                                cards=cards
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="space-y-3 p-2">
                                {card_groups
                                    .into_iter()
                                    .map(|(group_index, key, label, count, summary, cards)| {
                                        view! {
                                            <MobileCardGroup
                                                tab=tab
                                                label=label
                                                count=count
                                                summary=summary
                                                cards=cards
                                                id_prefix=format!("receipt-{slug}-group-{group_index}")
                                                expanded_key=format!("{slug}:g:{key}")
                                                expanded_ids=expanded_ids
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                        .into_any()
                    }}
                </div>
            </div>
        </div>
    }
}
