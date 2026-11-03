use crate::api::dto::AssetBalance;
use crate::support::csv_flow::CsvChunking;
use crate::ui::labels;

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
    header_marker: Some(labels::SECURITY_CODE),
};

pub(crate) type AssetBalanceRow = AssetBalance;

#[cfg(test)]
mod tests;
