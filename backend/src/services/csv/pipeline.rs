use crate::errors::{ApiError, CsvError};
use crate::services::csv::util::{CsvRowView, HeaderIndex, decode_bytes};

/// ヘッダー共有索引と StringRecord の行をそのまま保持する CSV テーブル。
/// 行ごとに HashMap<String, String> へ複製しない
pub struct CsvTable {
    index: HeaderIndex,
    records: Vec<csv::StringRecord>,
}

impl CsvTable {
    /// 共有索引付きの行ビューを先頭から順に返す
    pub fn rows(&self) -> impl Iterator<Item = CsvRowView<'_>> + '_ {
        self.records
            .iter()
            .map(move |record| CsvRowView::new(record, &self.index))
    }
}

#[derive(Clone, Copy)]
pub struct CsvParserConfig {
    pub skip_header_rows: usize,
    pub exclude_row_fn: Option<fn(&CsvRowView<'_>) -> bool>,
    /// 設定時、先頭 MAX_HEADER_SCAN_LINES 行以内でこの文字で始まる列を含む最初の行を
    /// 見出しとし、それより前の行(表題など)を読み飛ばす。skip_header_rows より優先
    pub header_marker: Option<&'static str>,
}

/// 見出し行の自動検出で走査する先頭からの行数
const MAX_HEADER_SCAN_LINES: usize = 10;

pub fn parse_csv_with_config(bytes: &[u8], config: &CsvParserConfig) -> Result<CsvTable, ApiError> {
    if bytes.is_empty() {
        return Err(CsvError::Empty.into());
    }

    let content = decode_bytes(bytes);
    let stripped = match config.header_marker {
        Some(marker) => match find_header_offset(&content, marker) {
            Some(offset) => &content[offset..],
            None => return Err(CsvError::HeaderNotFound(marker).into()),
        },
        None => strip_header_rows(&content, config.skip_header_rows),
    };
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(stripped.as_bytes());

    let index = HeaderIndex::new(
        reader
            .headers()
            .map_err(|error| CsvError::HeaderRead(error.to_string()))?
            .iter(),
    );

    let mut records = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|error| CsvError::RowRead(error.to_string()))?;

        if is_all_empty_record(&record) {
            continue;
        }

        let view = CsvRowView::new(&record, &index);
        if config
            .exclude_row_fn
            .is_some_and(|exclude_row| exclude_row(&view))
        {
            continue;
        }
        records.push(record);
    }

    Ok(CsvTable { index, records })
}

fn strip_header_rows(content: &str, skip_header_rows: usize) -> &str {
    let mut start = 0;
    for _ in 0..skip_header_rows {
        match content[start..].find('\n') {
            Some(pos) => start += pos + 1,
            None => return "",
        }
    }
    &content[start..]
}

/// 先頭 MAX_HEADER_SCAN_LINES 行のうち、marker で始まる列を含む最初の行の開始位置を返す
fn find_header_offset(content: &str, marker: &str) -> Option<usize> {
    let mut offset = 0;
    for line in content.split_inclusive('\n').take(MAX_HEADER_SCAN_LINES) {
        if line_has_column_starting_with(line.trim_end(), marker) {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

/// 1行を CSV レコードとして読み、marker で始まる列を含むかを返す
fn line_has_column_starting_with(line: &str, marker: &str) -> bool {
    let mut record = csv::StringRecord::new();
    csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(line.as_bytes())
        .read_record(&mut record)
        .is_ok_and(|read| read && record.iter().any(|field| field.trim().starts_with(marker)))
}

fn is_all_empty_record(record: &csv::StringRecord) -> bool {
    record.iter().all(|value| value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 0,
        exclude_row_fn: None,
        header_marker: None,
    };

    fn exclude_total_row(row: &CsvRowView<'_>) -> bool {
        row.cell("name").contains("合計")
    }

    fn collect(table: &CsvTable) -> Vec<CsvRowView<'_>> {
        table.rows().collect()
    }

    // csv::Reader へ渡す入力は常にメモリ上の UTF-8 バイト列で、flexible(true) も相まって
    // ヘッダー/レコード読み取りエラーはユニットテストで再現できなかったため、
    // 空入力の境界のみここで固定する。
    #[test]
    fn test_parse_csv_empty_bytes() {
        let result = parse_csv_with_config(b"", &BASIC_CONFIG);

        assert!(matches!(result, Err(ApiError::Csv(CsvError::Empty))));
    }

    #[test]
    fn test_parse_csv_skips_empty_lines() {
        let table = parse_csv_with_config(
            "name,amount\nfoo,100\n,\nbar,200\n".as_bytes(),
            &BASIC_CONFIG,
        )
        .unwrap();
        let rows = collect(&table);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].cell("name"), "foo");
        assert_eq!(rows[1].cell("name"), "bar");

        let table = parse_csv_with_config(
            "name,amount\nfoo,100\n , \n\nbar,200\n".as_bytes(),
            &BASIC_CONFIG,
        )
        .unwrap();
        let rows = collect(&table);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].cell("name"), "foo");
        assert_eq!(rows[1].cell("name"), "bar");
    }

    #[test]
    fn test_parse_csv_with_config() {
        let table = parse_csv_with_config(
            "meta\nname,amount\nfoo,100\nbar\n,\n".as_bytes(),
            &CsvParserConfig {
                skip_header_rows: 1,
                ..BASIC_CONFIG
            },
        )
        .unwrap();
        let rows = collect(&table);

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].cell("name"), "foo");
        assert_eq!(rows[1].cell("amount"), "");
        assert_eq!(rows[1].cell("missing"), "");

        let table = parse_csv_with_config(
            "name,amount\nfoo,100\n特定口座合計,200\n".as_bytes(),
            &CsvParserConfig {
                exclude_row_fn: Some(exclude_total_row),
                ..BASIC_CONFIG
            },
        )
        .unwrap();
        let rows = collect(&table);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cell("name"), "foo");
    }

    #[test]
    fn test_parse_csv_header_marker_detects_header_line() {
        let config = CsvParserConfig {
            header_marker: Some("入金日"),
            ..BASIC_CONFIG
        };
        // 表題行つき: 見出しより前の行は読み飛ばす
        let table = parse_csv_with_config(
            "配当金・分配金・利金一覧\n入金日(受渡日),商品\n\"2025/03/31\",\"国内株式\"\n"
                .as_bytes(),
            &config,
        )
        .unwrap();
        let rows = collect(&table);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cell("入金日(受渡日)"), "2025/03/31");
        assert_eq!(rows[0].cell("商品"), "国内株式");

        // 先頭行が見出しの従来形式もそのまま読める
        let table = parse_csv_with_config(
            "入金日,商品\n\"2025/03/31\",\"国内株式\"\n".as_bytes(),
            &config,
        )
        .unwrap();
        assert_eq!(collect(&table).len(), 1);
    }

    #[test]
    fn test_parse_csv_header_marker_not_found_is_single_error() {
        let config = CsvParserConfig {
            header_marker: Some("入金日"),
            ..BASIC_CONFIG
        };
        // 見出しに相当する行が無いファイルは行ごとのエラーではなく1件のエラー
        let result =
            parse_csv_with_config("商品,口座\n\"国内株式\",\"特定\"\n".as_bytes(), &config);
        assert!(matches!(
            result,
            Err(ApiError::Csv(CsvError::HeaderNotFound("入金日")))
        ));

        // 走査範囲(10行)を超えた位置にある見出しも見つからない扱い
        let content = format!("{}入金日,商品\nx,y\n", "メモ\n".repeat(10));
        let result = parse_csv_with_config(content.as_bytes(), &config);
        assert!(matches!(
            result,
            Err(ApiError::Csv(CsvError::HeaderNotFound(_)))
        ));
    }

    proptest::proptest! {
        #[test]
        fn prop_parse_csv_with_config_matches_reference(
            skip_header_rows in 0usize..3usize,
            headers in proptest::collection::hash_set("[a-z]{1,8}", 1..5usize),
            cells in proptest::collection::vec(
                proptest::collection::vec("[a-zA-Z0-9 ]{0,12}", 0..7usize),
                0..8usize
            ),
        ) {
            let headers: Vec<String> = headers.into_iter().collect();
            let mut lines: Vec<String> = (0..skip_header_rows)
                .map(|i| format!("meta{i}"))
                .collect();
            lines.push(headers.join(","));
            for row in &cells {
                lines.push(row.join(","));
            }
            let csv = lines.join("\n");

            let config = CsvParserConfig {
                skip_header_rows,
                exclude_row_fn: None,
                header_marker: None,
            };
            let table = parse_csv_with_config(csv.as_bytes(), &config).unwrap();

            let expected: Vec<&Vec<String>> = cells
                .iter()
                .filter(|record| !record.iter().all(|c| c.trim().is_empty()))
                .collect();
            proptest::prop_assert_eq!(table.rows().count(), expected.len());

            for (actual, record) in table.rows().zip(expected.iter()) {
                proptest::prop_assert_eq!(
                    actual.cell("required_missing"),
                    ""
                );
                for (index, header) in headers.iter().enumerate() {
                    let expected_cell = record
                        .get(index)
                        .map(|c| c.trim())
                        .unwrap_or_default();
                    proptest::prop_assert_eq!(
                        actual.cell(header),
                        expected_cell,
                        "header={}",
                        header
                    );
                }
            }
        }
    }
}
