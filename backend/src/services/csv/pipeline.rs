use crate::errors::{ApiError, CsvError};
use crate::services::csv::util::{decode_bytes, CsvRowView, HeaderIndex};

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
}

pub fn parse_csv_with_config(bytes: &[u8], config: &CsvParserConfig) -> Result<CsvTable, ApiError> {
    if bytes.is_empty() {
        return Err(CsvError::Empty.into());
    }

    let content = decode_bytes(bytes);
    let stripped = strip_header_rows(&content, config.skip_header_rows);
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

fn is_all_empty_record(record: &csv::StringRecord) -> bool {
    record.iter().all(|value| value.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::csv::util::CsvCells;

    const BASIC_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 0,
        exclude_row_fn: None,
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
