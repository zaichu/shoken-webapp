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
    pub(super) slug: &'static str,
    pub(super) list_path: &'static str,
    pub(super) preview_path: &'static str,
    pub(super) import_path: &'static str,
    pub(super) columns: &'static [ColumnSpec],
    // ヘッダー集計の項目。(ラベル, 損益系なら負数で赤文字にするか)
    pub(super) header_items: [(&'static str, bool); 3],
    pub(super) empty_hint: &'static str,
    // 一覧の日付見出しと約定金額の見出し。配当は金額を持たない
    pub(super) date_label: &'static str,
    pub(super) amount_label: Option<&'static str>,
    // 月見出しの右に出す主役の見出しと帯の主見出し
    pub(super) headline_label: &'static str,
    pub(super) summary_primary_label: &'static str,
    // 狭い表で残す列の見出し。columns の部分集合にする
    pub(super) compact_headers: &'static [&'static str],
    // CSV の見出し自動検出。配当だけ表題行つき形式に対応する
    pub(super) csv_header_marker: Option<&'static str>,
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
        slug: "dividend",
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
        date_label: labels::PAYMENT_DATE,
        amount_label: None,
        headline_label: labels::DIVIDEND_AMOUNT,
        summary_primary_label: labels::RECEIVED_DIVIDEND_AFTER_TAX,
        compact_headers: &[
            labels::PAYMENT_DATE,
            labels::SECURITY_CODE,
            labels::SECURITY_NAME,
            labels::ACCOUNT,
            labels::QUANTITY,
            labels::DIVIDEND_AMOUNT,
            labels::TAX,
            labels::AFTER_TAX,
        ],
        csv_header_marker: Some(labels::PAYMENT_DATE),
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
        slug: "domesticstock",
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
        date_label: labels::TRADE_DATE,
        amount_label: Some(labels::SELLING_AMOUNT),
        headline_label: labels::REALIZED_PROFIT_LOSS,
        summary_primary_label: labels::REALIZED_AFTER_TAX,
        compact_headers: &[
            labels::TRADE_DATE,
            labels::SECURITY_CODE,
            labels::SECURITY_NAME,
            labels::ACCOUNT,
            labels::QUANTITY,
            labels::SELLING_AMOUNT,
            labels::ACQUISITION_PRICE,
            labels::REALIZED_PROFIT_LOSS,
            labels::TAX,
            labels::AFTER_TAX,
        ],
        csv_header_marker: None,
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
        slug: "mutualfund",
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
        date_label: labels::TRADE_DATE,
        amount_label: Some(labels::REDEMPTION_AMOUNT),
        headline_label: labels::REALIZED_PROFIT_LOSS,
        summary_primary_label: labels::REALIZED_AFTER_TAX,
        compact_headers: &[
            labels::TRADE_DATE,
            labels::FUND_NAME,
            labels::ACCOUNT,
            labels::QUANTITY,
            labels::REDEMPTION_AMOUNT,
            labels::ACQUISITION_PRICE,
            labels::REALIZED_PROFIT_LOSS,
            labels::TAX,
            labels::AFTER_TAX,
        ],
        csv_header_marker: None,
        string_fields: &[ReceiptRow::name, ReceiptRow::account],
        search_amounts: &[],
        product_category: false,
        account_category: false,
        reorder_rules: &[],
        reorder_fixed: 0,
    },
];
