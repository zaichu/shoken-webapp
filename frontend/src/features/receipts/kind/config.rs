use super::{
    CardRole, ColumnAlign, ColumnSpec, ColumnTier, ReceiptRow, account_matches, product_matches,
};
use crate::support::list_search::support::ColumnReorderRule;
use rust_decimal::Decimal;

// 列幅は ch ではなく font-size 基準の em で指定する。`0` の字幅はフォントで変わるが、
// `¥`・`/`・漢字を含む本文の幅は font-size に対して安定するため
pub(super) struct ReceiptKind {
    pub(super) label: &'static str,
    pub(super) list_path: &'static str,
    pub(super) preview_path: &'static str,
    pub(super) import_path: &'static str,
    pub(super) columns: &'static [ColumnSpec],
    pub(super) summary_labels: [&'static str; 3],
    // ヘッダー集計の項目。(ラベル, 損益系なら負数で赤文字にするか)
    pub(super) header_items: [(&'static str, bool); 3],
    pub(super) empty_hint: &'static str,
    pub(super) csv_input_id: &'static str,
    pub(super) string_fields: &'static [fn(&ReceiptRow) -> &str],
    pub(super) search_amounts: &'static [fn(&ReceiptRow) -> Decimal],
    pub(super) product_category: bool,
    pub(super) account_category: bool,
    pub(super) reorder_rules: &'static [ColumnReorderRule<ReceiptRow>],
    pub(super) reorder_fixed: usize,
}

const fn col(
    header: &'static str,
    width: &'static str,
    tier: ColumnTier,
    align: ColumnAlign,
) -> ColumnSpec {
    ColumnSpec {
        header,
        width,
        tier,
        align,
        card_role: CardRole::None,
    }
}

const fn card_col(
    header: &'static str,
    width: &'static str,
    tier: ColumnTier,
    align: ColumnAlign,
    card_role: CardRole,
) -> ColumnSpec {
    ColumnSpec {
        header,
        width,
        tier,
        align,
        card_role,
    }
}

pub(super) const RECEIPT_KINDS: [ReceiptKind; 3] = [
    ReceiptKind {
        label: "配当金",
        list_path: "/api/v1/dividends",
        preview_path: "/api/v1/dividend-import-validations",
        import_path: "/api/v1/dividend-imports",
        columns: &[
            card_col(
                "入金日",
                "9em",
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Date,
            ),
            col("商品", "7.5em", ColumnTier::Wider, ColumnAlign::Left),
            card_col(
                "口座",
                "7.5em",
                ColumnTier::Wider,
                ColumnAlign::Left,
                CardRole::Account,
            ),
            col("銘柄コード", "10em", ColumnTier::Wide, ColumnAlign::Center),
            card_col(
                "銘柄名",
                "",
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Name,
            ),
            col("単価", "10em", ColumnTier::Wider, ColumnAlign::Right),
            col("数量", "7em", ColumnTier::Md, ColumnAlign::Right),
            col("配当金", "10em", ColumnTier::Core, ColumnAlign::Right),
            col("税額", "10em", ColumnTier::Core, ColumnAlign::Right),
            col("税引後", "10em", ColumnTier::Core, ColumnAlign::Right),
        ],
        summary_labels: ["配当金", "税額", "税引後"],
        header_items: [("配当金", false), ("税額", false), ("税引後", false)],
        empty_hint: "配当金明細をCSVで追加してください",
        csv_input_id: "csv-file-input-dividend",
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
                column_key: 1,
                matches: product_matches,
            },
            ColumnReorderRule {
                column_key: 2,
                matches: account_matches,
            },
        ],
        reorder_fixed: 1,
    },
    ReceiptKind {
        label: "国内株式",
        list_path: "/api/v1/domestic-stock-transactions",
        preview_path: "/api/v1/domestic-stock-import-validations",
        import_path: "/api/v1/domestic-stock-imports",
        columns: &[
            card_col(
                "約定日",
                "9em",
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Date,
            ),
            col("銘柄コード", "10em", ColumnTier::Wide, ColumnAlign::Center),
            card_col(
                "銘柄名",
                "",
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Name,
            ),
            card_col(
                "口座",
                "7.5em",
                ColumnTier::Wider,
                ColumnAlign::Left,
                CardRole::Account,
            ),
            col("数量", "7em", ColumnTier::Wide, ColumnAlign::Right),
            col("売却単価", "10em", ColumnTier::Wider, ColumnAlign::Right),
            col("売却額", "10em", ColumnTier::Wider, ColumnAlign::Right),
            col("取得価額", "10em", ColumnTier::Wider, ColumnAlign::Right),
            col("実現損益", "10em", ColumnTier::Core, ColumnAlign::Right),
            col("税額", "10em", ColumnTier::Core, ColumnAlign::Right),
            col("税引後", "10em", ColumnTier::Core, ColumnAlign::Right),
        ],
        summary_labels: ["実現損益", "税額", "税引後"],
        header_items: [("実現損益", true), ("税額", false), ("税引後", true)],
        empty_hint: "国内株式明細をCSVで追加してください",
        csv_input_id: "csv-file-input-domesticstock",
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
        label: "投資信託",
        list_path: "/api/v1/mutual-fund-transactions",
        preview_path: "/api/v1/mutual-fund-import-validations",
        import_path: "/api/v1/mutual-fund-imports",
        columns: &[
            card_col(
                "約定日",
                "9em",
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Date,
            ),
            card_col(
                "ファンド名",
                "",
                ColumnTier::Core,
                ColumnAlign::Left,
                CardRole::Name,
            ),
            card_col(
                "口座",
                "7.5em",
                ColumnTier::Wider,
                ColumnAlign::Left,
                CardRole::Account,
            ),
            col("数量", "7em", ColumnTier::Wide, ColumnAlign::Right),
            col("解約単価", "10em", ColumnTier::Wider, ColumnAlign::Right),
            col("解約額", "10em", ColumnTier::Wide, ColumnAlign::Right),
            col("取得価額", "10em", ColumnTier::Wider, ColumnAlign::Right),
            col("実現損益", "10em", ColumnTier::Core, ColumnAlign::Right),
            col("税額", "10em", ColumnTier::Core, ColumnAlign::Right),
            col("税引後", "10em", ColumnTier::Core, ColumnAlign::Right),
        ],
        summary_labels: ["実現損益", "税額", "税引後"],
        header_items: [("実現損益", true), ("税額", false), ("税引後", true)],
        empty_hint: "投資信託明細をCSVで追加してください",
        csv_input_id: "csv-file-input-mutualfund",
        string_fields: &[ReceiptRow::name, ReceiptRow::account],
        search_amounts: &[],
        product_category: false,
        account_category: false,
        reorder_rules: &[],
        reorder_fixed: 0,
    },
];
