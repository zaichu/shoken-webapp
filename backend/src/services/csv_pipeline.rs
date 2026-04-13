use crate::errors::ApiError;
use crate::services::csv_util::decode_bytes;
use std::collections::HashMap;

pub type CsvRow = HashMap<String, String>;

#[derive(Clone, Copy)]
pub struct CsvParserConfig {
    pub skip_header_rows: usize,
    pub exclude_row_fn: Option<fn(&CsvRow) -> bool>,
    pub required_columns: &'static [&'static str],
}

pub fn parse_csv_with_config(
    bytes: &[u8],
    config: &CsvParserConfig,
) -> Result<Vec<CsvRow>, ApiError> {
    if bytes.is_empty() {
        return Err(ApiError::ValidationError("CSVが空です".to_string()));
    }

    let content = strip_header_rows(&decode_bytes(bytes), config.skip_header_rows);
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers = reader
        .headers()
        .map_err(|error| {
            ApiError::ValidationError(format!("CSVヘッダーの読み込みに失敗しました: {}", error))
        })?
        .iter()
        .map(|header| header.trim().to_string())
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    for record in reader.records() {
        let record = record.map_err(|error| {
            ApiError::ValidationError(format!("CSV行の読み込みに失敗しました: {}", error))
        })?;

        if is_all_empty_record(&record) {
            continue;
        }

        let row = build_row_map(&headers, &record, config.required_columns);
        if config
            .exclude_row_fn
            .is_some_and(|exclude_row| exclude_row(&row))
        {
            continue;
        }
        rows.push(row);
    }

    Ok(rows)
}

fn strip_header_rows(content: &str, skip_header_rows: usize) -> String {
    content
        .lines()
        .skip(skip_header_rows)
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_all_empty_record(record: &csv::StringRecord) -> bool {
    record.iter().all(|value| value.trim().is_empty())
}

fn build_row_map(
    headers: &[String],
    record: &csv::StringRecord,
    required_columns: &[&str],
) -> CsvRow {
    let mut row = headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            (
                header.clone(),
                record.get(index).unwrap_or("").trim().to_string(),
            )
        })
        .collect::<HashMap<_, _>>();

    for column in required_columns {
        row.entry((*column).to_string()).or_default();
    }

    row
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASIC_CONFIG: CsvParserConfig = CsvParserConfig {
        skip_header_rows: 0,
        exclude_row_fn: None,
        required_columns: &["name", "amount", "missing"],
    };

    fn exclude_total_row(row: &CsvRow) -> bool {
        row.get("name").is_some_and(|name| name.contains("合計"))
    }

    #[test]
    fn test_parse_csv_with_config() {
        let rows = parse_csv_with_config(
            "meta\nname,amount\nfoo,100\nbar\n,\n".as_bytes(),
            &CsvParserConfig {
                skip_header_rows: 1,
                ..BASIC_CONFIG
            },
        )
        .unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].get("name"), Some(&"foo".to_string()));
        assert_eq!(rows[1].get("amount"), Some(&"".to_string()));
        assert_eq!(rows[1].get("missing"), Some(&"".to_string()));

        let rows = parse_csv_with_config(
            "name,amount\nfoo,100\n特定口座合計,200\n".as_bytes(),
            &CsvParserConfig {
                exclude_row_fn: Some(exclude_total_row),
                ..BASIC_CONFIG
            },
        )
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("name"), Some(&"foo".to_string()));
    }
}
