use crate::features::receipts::ReceiptsTab;
use crate::support::csv_flow::CsvChunking;

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
