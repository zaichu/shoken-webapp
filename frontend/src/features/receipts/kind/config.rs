use super::{
    CardRole, ColumnAlign, ColumnSpec, ColumnTier, ReceiptRow, account_matches, product_matches,
};
use crate::support::list_search::support::ColumnReorderRule;
use crate::ui::labels;
use rust_decimal::Decimal;

// 列幅は ch ではなく font-size 基準の em で指定する。`0` の字幅はフォントで変わるが、
// `¥`・`/`・漢字を含む本文の幅は font-size に対して安定するため
pub(super) struct ReceiptKind {
    pub(super) label: &'static str,
    pub(super) list_path: &'static str,
    pub(super) preview_path: &'static str,
    pub(super) import_path: &'static str,
    pub(super) columns: &'static [ColumnSpec],
    // ヘッダー集計の項目。(ラベル, 損益系なら負数で赤文字にするか)
    pub(super) header_items: [(&'static str, bool); 3],
    pub(super) empty_hint: &'static str,
    pub(super) string_fields: &'static [fn(&ReceiptRow) -> &str],
    pub(super) search_amounts: &'static [fn(&ReceiptRow) -> Decimal],
    pub(super) product_category: bool,
    pub(super) account_category: bool,
    pub(super) reorder_rules: &'static [ColumnReorderRule<ReceiptRow>],
    pub(super) reorder_fixed: usize,
}

const fn col(header: &'static str, tier: ColumnTier, align: ColumnAlign) -> ColumnSpec {
    ColumnSpec {
        header,
        tier,
        align,
        card_role: CardRole::None,
    }
}

const fn card_col(
    header: &'static str,
    tier: ColumnTier,
    align: ColumnAlign,
    card_role: CardRole,
) -> ColumnSpec {
    ColumnSpec {
        header,
        tier,
        align,
        card_role,
    }
}

pub(super) const RECEIPT_KINDS: [ReceiptKind; 3] = [
    ReceiptKind {
        label: labels::DIVIDEND_AMOUNT,
        list_path: "/api/v1/dividends",
        preview_path: "/api/v1/dividend-import-validations",
        import_path: "/api/v1/dividend-imports",
        columns: &[
            card_col(
                labels::PAYMENT_DATE,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Date,
            ),
            card_col(
                labels::SECURITY_CODE,
                ColumnTier::Wide,
                ColumnAlign::Left,
                CardRole::Code,
            ),
            card_col(
                labels::SECURITY_NAME,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Name,
            ),
            card_col(
                labels::ACCOUNT,
                ColumnTier::Wider,
                ColumnAlign::Left,
                CardRole::Account,
            ),
            col(labels::PRODUCT, ColumnTier::Wider, ColumnAlign::Left),
            col(
                labels::PER_SHARE_DIVIDEND,
                ColumnTier::Wider,
                ColumnAlign::Right,
            ),
            col(labels::QUANTITY, ColumnTier::Md, ColumnAlign::Right),
            col(
                labels::DIVIDEND_AMOUNT,
                ColumnTier::Core,
                ColumnAlign::Right,
            ),
            col(labels::TAX, ColumnTier::Core, ColumnAlign::Right),
            col(labels::AFTER_TAX, ColumnTier::Core, ColumnAlign::Right),
        ],
        header_items: [
            (labels::DIVIDEND_AMOUNT, false),
            (labels::TAX, false),
            (labels::AFTER_TAX, true),
        ],
        empty_hint: labels::RECEIPT_DIVIDEND_EMPTY_HINT,
        string_fields: &[
            ReceiptRow::code,
            ReceiptRow::name,
            ReceiptRow::account,
            ReceiptRow::product,
        ],
        search_amounts: &[
            |row| row.search_amount(0),
            |row| row.search_amount(1),
            |row| row.search_amount(2),
            |row| row.search_amount(3),
            |row| row.search_amount(4),
        ],
        product_category: true,
        account_category: true,
        reorder_rules: &[
            ColumnReorderRule {
                column_key: 4,
                matches: product_matches,
            },
            ColumnReorderRule {
                column_key: 3,
                matches: account_matches,
            },
        ],
        reorder_fixed: 1,
    },
    ReceiptKind {
        label: labels::DOMESTIC_STOCK,
        list_path: "/api/v1/domestic-stock-transactions",
        preview_path: "/api/v1/domestic-stock-import-validations",
        import_path: "/api/v1/domestic-stock-imports",
        columns: &[
            card_col(
                labels::TRADE_DATE,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Date,
            ),
            card_col(
                labels::SECURITY_CODE,
                ColumnTier::Wide,
                ColumnAlign::Left,
                CardRole::Code,
            ),
            card_col(
                labels::SECURITY_NAME,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Name,
            ),
            card_col(
                labels::ACCOUNT,
                ColumnTier::Wider,
                ColumnAlign::Left,
                CardRole::Account,
            ),
            col(labels::QUANTITY, ColumnTier::Wide, ColumnAlign::Right),
            col(
                labels::SELLING_UNIT_PRICE,
                ColumnTier::Wider,
                ColumnAlign::Right,
            ),
            col(
                labels::SELLING_AMOUNT,
                ColumnTier::Wider,
                ColumnAlign::Right,
            ),
            col(
                labels::ACQUISITION_PRICE,
                ColumnTier::Wider,
                ColumnAlign::Right,
            ),
            col(
                labels::REALIZED_PROFIT_LOSS,
                ColumnTier::Core,
                ColumnAlign::Right,
            ),
            col(labels::TAX, ColumnTier::Core, ColumnAlign::Right),
            col(labels::AFTER_TAX, ColumnTier::Core, ColumnAlign::Right),
        ],
        header_items: [
            (labels::REALIZED_PROFIT_LOSS, true),
            (labels::TAX, false),
            (labels::AFTER_TAX, true),
        ],
        empty_hint: labels::RECEIPT_STOCK_EMPTY_HINT,
        string_fields: &[ReceiptRow::code, ReceiptRow::name, ReceiptRow::account],
        search_amounts: &[
            |row| row.search_amount(0),
            |row| row.search_amount(1),
            |row| row.search_amount(2),
            |row| row.search_amount(3),
            |row| row.search_amount(4),
        ],
        product_category: false,
        account_category: true,
        reorder_rules: &[ColumnReorderRule {
            column_key: 3,
            matches: account_matches,
        }],
        reorder_fixed: 2,
    },
    ReceiptKind {
        label: labels::MUTUAL_FUND,
        list_path: "/api/v1/mutual-fund-transactions",
        preview_path: "/api/v1/mutual-fund-import-validations",
        import_path: "/api/v1/mutual-fund-imports",
        columns: &[
            card_col(
                labels::TRADE_DATE,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Date,
            ),
            card_col(
                labels::FUND_NAME,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Name,
            ),
            card_col(
                labels::ACCOUNT,
                ColumnTier::Wider,
                ColumnAlign::Left,
                CardRole::Account,
            ),
            col(labels::QUANTITY, ColumnTier::Wide, ColumnAlign::Right),
            col(
                labels::REDEMPTION_UNIT_PRICE,
                ColumnTier::Wider,
                ColumnAlign::Right,
            ),
            col(
                labels::REDEMPTION_AMOUNT,
                ColumnTier::Wide,
                ColumnAlign::Right,
            ),
            col(
                labels::ACQUISITION_PRICE,
                ColumnTier::Wider,
                ColumnAlign::Right,
            ),
            col(
                labels::REALIZED_PROFIT_LOSS,
                ColumnTier::Core,
                ColumnAlign::Right,
            ),
            col(labels::TAX, ColumnTier::Core, ColumnAlign::Right),
            col(labels::AFTER_TAX, ColumnTier::Core, ColumnAlign::Right),
        ],
        header_items: [
            (labels::REALIZED_PROFIT_LOSS, true),
            (labels::TAX, false),
            (labels::AFTER_TAX, true),
        ],
        empty_hint: labels::RECEIPT_FUND_EMPTY_HINT,
        string_fields: &[ReceiptRow::name, ReceiptRow::account],
        search_amounts: &[],
        product_category: false,
        account_category: false,
        reorder_rules: &[],
        reorder_fixed: 0,
    },
];
