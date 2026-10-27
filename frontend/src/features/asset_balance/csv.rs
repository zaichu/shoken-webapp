use crate::api::dto::AssetBalance;
use crate::support::csv_flow::CsvChunking;
use crate::support::row::Row;

pub const LIST_PATH: &str = "/api/v1/asset-balances";
pub const PREVIEW_PATH: &str = "/api/v1/asset-balance-import-validations";
pub const IMPORT_PATH: &str = "/api/v1/asset-balance-imports";

// asset_balance は証券会社の前置き行(口座数で行数が変わる)を挟んで「銘柄コード」の
// 見出しが続く形式。WriteMode::Replace のため import を分割すると後続チャンクが
// 前の行を消すので、import は単一リクエストのままにする。口座は区切り行の文脈に
// 依存するため preview も分割しない
pub const CSV_CHUNKING: CsvChunking = CsvChunking {
    header_lines: 7,
    import_chunked: false,
    preview_chunked: false,
    header_marker: Some("銘柄コード"),
};

pub(crate) type AssetBalanceRow = Row<AssetBalance>;

/// `AssetBalance`(保存済み)の読み出し面。
pub(crate) trait AssetBalanceRowData {
    fn security_code(&self) -> &str;
    fn security_name(&self) -> &str;
    /// 口座区分。旧データは None
    fn account(&self) -> Option<&str>;
    fn shares(&self) -> rust_decimal::Decimal;
    fn average_purchase_price(&self) -> rust_decimal::Decimal;
    fn total_purchase_amount(&self) -> rust_decimal::Decimal;
    fn current_price(&self) -> rust_decimal::Decimal;
    /// 1株あたりの前日比。前日終値がない(上場直後・取得失敗)銘柄は None
    fn day_change(&self) -> Option<rust_decimal::Decimal>;
    /// 現在値の基準日
    fn price_as_of(&self) -> Option<&str>;
}

impl AssetBalanceRowData for AssetBalance {
    fn security_code(&self) -> &str {
        self.security_code.as_str()
    }
    fn security_name(&self) -> &str {
        &self.security_name
    }
    fn account(&self) -> Option<&str> {
        self.account.as_ref().map(|a| a.as_str())
    }
    fn shares(&self) -> rust_decimal::Decimal {
        self.shares
    }
    fn average_purchase_price(&self) -> rust_decimal::Decimal {
        self.average_purchase_price
    }
    fn total_purchase_amount(&self) -> rust_decimal::Decimal {
        self.total_purchase_amount
    }
    fn current_price(&self) -> rust_decimal::Decimal {
        self.current_price
    }
    fn day_change(&self) -> Option<rust_decimal::Decimal> {
        // daily_change はビューで共有価格確定行を 0 に丸めるため、
        // prev_close で「前日終値なし」を判定する
        if self.price_as_of.is_some() && self.prev_close.is_none() {
            None
        } else {
            Some(self.daily_change)
        }
    }
    fn price_as_of(&self) -> Option<&str> {
        self.price_as_of.as_deref()
    }
}

impl AssetBalanceRow {
    /// 詳細ページのパスに使う保存済みレコード ID
    pub(crate) fn saved_id(&self) -> Option<String> {
        self.saved().map(|row| row.id.to_string())
    }
}

impl AssetBalanceRowData for AssetBalanceRow {
    fn security_code(&self) -> &str {
        self.saved().map(|row| row.security_code()).unwrap_or("")
    }
    fn security_name(&self) -> &str {
        self.saved().map(|row| row.security_name()).unwrap_or("")
    }
    fn account(&self) -> Option<&str> {
        self.saved().and_then(|row| row.account())
    }
    fn shares(&self) -> rust_decimal::Decimal {
        self.saved()
            .map(|row| row.shares())
            .unwrap_or(rust_decimal::Decimal::ZERO)
    }
    fn average_purchase_price(&self) -> rust_decimal::Decimal {
        self.saved()
            .map(|row| row.average_purchase_price())
            .unwrap_or(rust_decimal::Decimal::ZERO)
    }
    fn total_purchase_amount(&self) -> rust_decimal::Decimal {
        self.saved()
            .map(|row| row.total_purchase_amount())
            .unwrap_or(rust_decimal::Decimal::ZERO)
    }
    fn current_price(&self) -> rust_decimal::Decimal {
        self.saved()
            .map(|row| row.current_price())
            .unwrap_or(rust_decimal::Decimal::ZERO)
    }
    fn day_change(&self) -> Option<rust_decimal::Decimal> {
        self.saved().and_then(|row| row.day_change())
    }
    fn price_as_of(&self) -> Option<&str> {
        self.saved().and_then(|row| row.price_as_of())
    }
}
