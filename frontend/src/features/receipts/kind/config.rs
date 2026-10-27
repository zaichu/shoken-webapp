use super::{
    CardRole, ColumnAlign, ColumnSpec, ColumnTier, ReceiptRow, account_matches, product_matches,
};
use crate::support::list_search::support::ColumnReorderRule;
use crate::ui::labels::{
    ACCOUNT, ACQUISITION_PRICE, AFTER_TAX, DIVIDEND_AMOUNT, DOMESTIC_STOCK, FUND_NAME, MUTUAL_FUND,
    PAYMENT_DATE, PER_SHARE_DIVIDEND, PRODUCT, QUANTITY, REALIZED_PROFIT_LOSS, REDEMPTION_AMOUNT,
    REDEMPTION_UNIT_PRICE, SECURITY_CODE, SECURITY_NAME, SELLING_AMOUNT, SELLING_UNIT_PRICE, TAX,
    TRADE_DATE,
};
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
        label: DIVIDEND_AMOUNT,
        list_path: "/api/v1/dividends",
        preview_path: "/api/v1/dividend-import-validations",
        import_path: "/api/v1/dividend-imports",
        columns: &[
            card_col(
                PAYMENT_DATE,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Date,
            ),
            card_col(
                SECURITY_CODE,
                ColumnTier::Wide,
                ColumnAlign::Left,
                CardRole::Code,
            ),
            card_col(
                SECURITY_NAME,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Name,
            ),
            card_col(
                ACCOUNT,
                ColumnTier::Wider,
                ColumnAlign::Left,
                CardRole::Account,
            ),
            col(PRODUCT, ColumnTier::Wider, ColumnAlign::Left),
            col(PER_SHARE_DIVIDEND, ColumnTier::Wider, ColumnAlign::Right),
            col(QUANTITY, ColumnTier::Md, ColumnAlign::Right),
            col(DIVIDEND_AMOUNT, ColumnTier::Core, ColumnAlign::Right),
            col(TAX, ColumnTier::Core, ColumnAlign::Right),
            col(AFTER_TAX, ColumnTier::Core, ColumnAlign::Right),
        ],
        header_items: [(DIVIDEND_AMOUNT, false), (TAX, false), (AFTER_TAX, false)],
        empty_hint: "配当金明細をCSVで追加してください",
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
        label: DOMESTIC_STOCK,
        list_path: "/api/v1/domestic-stock-transactions",
        preview_path: "/api/v1/domestic-stock-import-validations",
        import_path: "/api/v1/domestic-stock-imports",
        columns: &[
            card_col(
                TRADE_DATE,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Date,
            ),
            card_col(
                SECURITY_CODE,
                ColumnTier::Wide,
                ColumnAlign::Left,
                CardRole::Code,
            ),
            card_col(
                SECURITY_NAME,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Name,
            ),
            card_col(
                ACCOUNT,
                ColumnTier::Wider,
                ColumnAlign::Left,
                CardRole::Account,
            ),
            col(QUANTITY, ColumnTier::Wide, ColumnAlign::Right),
            col(SELLING_UNIT_PRICE, ColumnTier::Wider, ColumnAlign::Right),
            col(SELLING_AMOUNT, ColumnTier::Wider, ColumnAlign::Right),
            col(ACQUISITION_PRICE, ColumnTier::Wider, ColumnAlign::Right),
            col(REALIZED_PROFIT_LOSS, ColumnTier::Core, ColumnAlign::Right),
            col(TAX, ColumnTier::Core, ColumnAlign::Right),
            col(AFTER_TAX, ColumnTier::Core, ColumnAlign::Right),
        ],
        header_items: [
            (REALIZED_PROFIT_LOSS, true),
            (TAX, false),
            (AFTER_TAX, true),
        ],
        empty_hint: "国内株式明細をCSVで追加してください",
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
        label: MUTUAL_FUND,
        list_path: "/api/v1/mutual-fund-transactions",
        preview_path: "/api/v1/mutual-fund-import-validations",
        import_path: "/api/v1/mutual-fund-imports",
        columns: &[
            card_col(
                TRADE_DATE,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Date,
            ),
            card_col(
                FUND_NAME,
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Name,
            ),
            card_col(
                ACCOUNT,
                ColumnTier::Wider,
                ColumnAlign::Left,
                CardRole::Account,
            ),
            col(QUANTITY, ColumnTier::Wide, ColumnAlign::Right),
            col(REDEMPTION_UNIT_PRICE, ColumnTier::Wider, ColumnAlign::Right),
            col(REDEMPTION_AMOUNT, ColumnTier::Wide, ColumnAlign::Right),
            col(ACQUISITION_PRICE, ColumnTier::Wider, ColumnAlign::Right),
            col(REALIZED_PROFIT_LOSS, ColumnTier::Core, ColumnAlign::Right),
            col(TAX, ColumnTier::Core, ColumnAlign::Right),
            col(AFTER_TAX, ColumnTier::Core, ColumnAlign::Right),
        ],
        header_items: [
            (REALIZED_PROFIT_LOSS, true),
            (TAX, false),
            (AFTER_TAX, true),
        ],
        empty_hint: "投資信託明細をCSVで追加してください",
        string_fields: &[ReceiptRow::name, ReceiptRow::account],
        search_amounts: &[],
        product_category: false,
        account_category: false,
        reorder_rules: &[],
        reorder_fixed: 0,
    },
];
