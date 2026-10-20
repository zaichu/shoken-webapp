use crate::api::dto::AssetBalance;
use crate::support::csv_flow::CsvChunking;
use crate::support::row::Row;
use rust_decimal::Decimal;
use serde::Deserialize;

pub const LIST_PATH: &str = "/api/v1/asset-balances";
pub const PREVIEW_PATH: &str = "/api/v1/asset-balance-import-validations";
pub const IMPORT_PATH: &str = "/api/v1/asset-balance-imports";

// asset_balance は証券会社の前置き行(口座数で行数が変わる)を挟んで「銘柄コード」の
// 見出しが続く形式。WriteMode::Replace のため import を分割すると後続チャンクが
// 前の行を消すので、preview だけ分割し import は単一リクエストのままにする
pub const CSV_CHUNKING: CsvChunking = CsvChunking {
    header_lines: 7,
    import_chunked: false,
    header_marker: Some("銘柄コード"),
};

// プレビュー行は backend が CreateAssetBalanceRequest をシリアライズしたもので id・タイムスタンプを持たないため、全フィールドを lenient に受け取る
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct AssetBalanceCsvRow {
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub security_name: String,
    /// 口座区分。CSV の「口座合計」区切りが無い・区切りより前の行は None
    #[serde(default)]
    pub account: Option<String>,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub average_purchase_price: Decimal,
    #[serde(default)]
    pub total_purchase_amount: Decimal,
    #[serde(default)]
    pub current_price: Decimal,
    #[serde(default)]
    pub daily_change: Option<Decimal>,
}

/// 一覧行と CSV プレビュー行を束ねる。プレビュー行は id・タイムスタンプを持たないため
/// `AssetBalance` に寄せず `Row::Preview` のまま扱う。
pub(crate) type AssetBalanceRow = Row<AssetBalance, AssetBalanceCsvRow>;

/// `AssetBalance`(保存済み) と `AssetBalanceCsvRow`(CSV プレビュー) の共通読み出し面。
/// 検証に通らない値も差し替えず画面に出すため、文字列は生値を返す。
pub(crate) trait AssetBalanceRowData {
    fn security_code(&self) -> &str;
    fn security_name(&self) -> &str;
    /// 口座区分。CSV の区切りから取れなかった行・旧データは None
    fn account(&self) -> Option<&str>;
    fn shares(&self) -> Decimal;
    fn average_purchase_price(&self) -> Decimal;
    fn total_purchase_amount(&self) -> Decimal;
    fn current_price(&self) -> Decimal;
    /// 1株あたりの前日比。前日終値がない(上場直後・取得失敗)銘柄は None
    fn day_change(&self) -> Option<Decimal>;
    /// 現在値の基準日。CSV プレビュー行には無い
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
    fn shares(&self) -> Decimal {
        self.shares
    }
    fn average_purchase_price(&self) -> Decimal {
        self.average_purchase_price
    }
    fn total_purchase_amount(&self) -> Decimal {
        self.total_purchase_amount
    }
    fn current_price(&self) -> Decimal {
        self.current_price
    }
    fn day_change(&self) -> Option<Decimal> {
        // daily_change はビューで共有価格確定行を 0 に丸めるため、
        // prev_close で「前日終値なし」を判定する。CSV 取込値の行はそのまま使う
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

impl AssetBalanceRowData for AssetBalanceCsvRow {
    fn security_code(&self) -> &str {
        &self.security_code
    }
    fn security_name(&self) -> &str {
        &self.security_name
    }
    fn account(&self) -> Option<&str> {
        self.account.as_deref()
    }
    fn shares(&self) -> Decimal {
        self.shares
    }
    fn average_purchase_price(&self) -> Decimal {
        self.average_purchase_price
    }
    fn total_purchase_amount(&self) -> Decimal {
        self.total_purchase_amount
    }
    fn current_price(&self) -> Decimal {
        self.current_price
    }
    fn day_change(&self) -> Option<Decimal> {
        self.daily_change
    }
    fn price_as_of(&self) -> Option<&str> {
        None
    }
}

impl AssetBalanceRow {
    fn inner(&self) -> &dyn AssetBalanceRowData {
        match self {
            Self::Saved(row) => row,
            Self::Preview(row) => row,
        }
    }

    /// 詳細ページのパスに使う保存済みレコード ID。CSV プレビュー行には無い
    pub(crate) fn saved_id(&self) -> Option<String> {
        self.saved().map(|row| row.id.to_string())
    }
}

impl AssetBalanceRowData for AssetBalanceRow {
    fn security_code(&self) -> &str {
        self.inner().security_code()
    }
    fn security_name(&self) -> &str {
        self.inner().security_name()
    }
    fn account(&self) -> Option<&str> {
        self.inner().account()
    }
    fn shares(&self) -> Decimal {
        self.inner().shares()
    }
    fn average_purchase_price(&self) -> Decimal {
        self.inner().average_purchase_price()
    }
    fn total_purchase_amount(&self) -> Decimal {
        self.inner().total_purchase_amount()
    }
    fn current_price(&self) -> Decimal {
        self.inner().current_price()
    }
    fn day_change(&self) -> Option<Decimal> {
        self.inner().day_change()
    }
    fn price_as_of(&self) -> Option<&str> {
        self.inner().price_as_of()
    }
}

#[cfg(test)]
mod tests;
