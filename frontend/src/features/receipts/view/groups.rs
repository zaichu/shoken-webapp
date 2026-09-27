pub(crate) use crate::features::receipts::kind::TableGroup;

use crate::features::receipts::{ReceiptRow, ReceiptsTab};

pub(crate) fn table_groups(
    tab: ReceiptsTab,
    rows: &[ReceiptRow],
    all_rows: &[ReceiptRow],
    query: &str,
) -> Vec<TableGroup> {
    tab.table_groups(rows, all_rows, query)
}
