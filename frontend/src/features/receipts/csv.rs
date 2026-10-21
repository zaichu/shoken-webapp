use crate::api::ApiError;
use crate::api::dto::CsvPreviewResponse;
use crate::features::receipts::ReceiptsTab;
use crate::support::csv_flow::CsvChunking;
use rust_decimal::Decimal;
use serde::Deserialize;

// backend の各 receipts 系 CsvParserConfig は skip_header_rows=0(先頭行がヘッダ)
const CSV_CHUNKING: CsvChunking = CsvChunking {
    header_lines: 1,
    import_chunked: true,
    preview_chunked: true,
    header_marker: None,
};

/// backend の配当CSVは表題行つき形式に対応するため見出しを自動検出する。
/// 分割送信の前置行も同じ規則でファイルごとに求める
pub fn chunking_for(tab: ReceiptsTab) -> CsvChunking {
    CsvChunking {
        header_marker: match tab {
            ReceiptsTab::Dividend => Some("入金日"),
            _ => None,
        },
        ..CSV_CHUNKING
    }
}

// プレビュー行は backend が Create*Request をシリアライズしたもので id・タイムスタンプを持たないため、全フィールドを lenient に受け取る(銘柄コード・口座は未検証の生値)
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct DividendCsvRow {
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub security_name: String,
    #[serde(default)]
    pub unit_price: Decimal,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub dividends_before_tax: Decimal,
    #[serde(default)]
    pub taxes: Decimal,
    #[serde(default)]
    pub net_amount_received: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct DomesticStockCsvRow {
    #[serde(default)]
    pub trade_date: String,
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub security_code: String,
    #[serde(default)]
    pub security_name: String,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub asked_price: Decimal,
    #[serde(default)]
    pub proceeds: Decimal,
    #[serde(default)]
    pub purchase_price: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss: Decimal,
    #[serde(default)]
    pub taxes: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct MutualfundCsvRow {
    #[serde(default)]
    pub trade_date: String,
    #[serde(default)]
    pub settlement_date: String,
    #[serde(default)]
    pub fund_name: String,
    #[serde(default)]
    pub dividends: Option<String>,
    #[serde(default)]
    pub account: String,
    #[serde(default)]
    pub shares: Decimal,
    #[serde(default)]
    pub exchange_rate: Decimal,
    #[serde(default)]
    pub cancellation_unit_price_yen: Decimal,
    #[serde(default)]
    pub cancellation_amount_yen: Decimal,
    #[serde(default)]
    pub average_acquisition_price_yen: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss: Decimal,
    #[serde(default)]
    pub taxes: Decimal,
    #[serde(default)]
    pub realized_profit_and_loss_after_tax: Decimal,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CsvPreviewRow {
    Dividend(DividendCsvRow),
    DomesticStock(DomesticStockCsvRow),
    MutualFund(MutualfundCsvRow),
}

/// タブ固有の行型で受け取り `CsvPreviewRow` に包む
fn wrap_rows<R>(
    response: CsvPreviewResponse<R>,
    wrap: fn(R) -> CsvPreviewRow,
) -> CsvPreviewResponse<CsvPreviewRow> {
    CsvPreviewResponse {
        total_rows: response.total_rows,
        valid_rows: response.valid_rows,
        errors: response.errors,
        rows: response.rows.into_iter().map(wrap).collect(),
    }
}

pub async fn fetch_preview(
    tab: ReceiptsTab,
    file: &web_sys::File,
) -> Result<CsvPreviewResponse<CsvPreviewRow>, ApiError> {
    async fn fetch<R: serde::de::DeserializeOwned>(
        tab: ReceiptsTab,
        file: &web_sys::File,
        wrap: fn(R) -> CsvPreviewRow,
    ) -> Result<CsvPreviewResponse<CsvPreviewRow>, ApiError> {
        let response: CsvPreviewResponse<R> =
            crate::support::csv_flow::preview_csv(tab.preview_path(), file, chunking_for(tab))
                .await?;
        Ok(wrap_rows(response, wrap))
    }
    match tab {
        ReceiptsTab::Dividend => fetch(tab, file, CsvPreviewRow::Dividend).await,
        ReceiptsTab::DomesticStock => fetch(tab, file, CsvPreviewRow::DomesticStock).await,
        ReceiptsTab::MutualFund => fetch(tab, file, CsvPreviewRow::MutualFund).await,
    }
}

#[cfg(test)]
mod tests;
