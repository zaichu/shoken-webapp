use csv::StringRecord;
use encoding_rs::SHIFT_JIS;
use thiserror::Error;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, File};

#[derive(Debug, Error)]
pub enum CSVError {
    #[error("デコードに失敗")]
    DecodeError,

    #[error("ファイル読み込み失敗: {0}")]
    FileReadError(String),

    #[error("CSV読み込み失敗: {0}")]
    CSVReadError(#[from] csv::Error),
}

pub async fn read_file(file: &File) -> Result<Vec<u8>, CSVError> {
    let array_buffer = JsFuture::from(file.array_buffer())
        .await
        .map_err(|e| CSVError::FileReadError(format!("{:?}", e)))?;
    Ok(js_sys::Uint8Array::new(&array_buffer).to_vec())
}

pub fn read_csv(bytes: Vec<u8>) -> Result<Vec<StringRecord>, CSVError> {
    let (cow, _, had_errors) = SHIFT_JIS.decode(&bytes);
    if had_errors {
        return Err(CSVError::DecodeError);
    }
    let utf8_string = cow.into_owned();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(utf8_string.as_bytes());
    rdr.records()
        .collect::<Result<Vec<_>, csv::Error>>()
        .map_err(CSVError::CSVReadError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv::{ReaderBuilder, StringRecord};

    fn read_csv_for_test(bytes: Vec<u8>) -> Result<Vec<StringRecord>, CSVError> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(&bytes[..]);
        rdr.records()
            .collect::<Result<Vec<_>, csv::Error>>()
            .map_err(CSVError::CSVReadError)
    }

    #[test]
    fn test_read_csv_valid() {
        let csv_data = "名前,年齢,住所\nテスト太郎,30,東京都\nテスト花子,25,大阪府"
            .as_bytes()
            .to_vec();

        let result = read_csv_for_test(csv_data);

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 2);

        assert_eq!(records[0].get(0), Some("テスト太郎"));
        assert_eq!(records[0].get(1), Some("30"));
        assert_eq!(records[0].get(2), Some("東京都"));

        assert_eq!(records[1].get(0), Some("テスト花子"));
        assert_eq!(records[1].get(1), Some("25"));
        assert_eq!(records[1].get(2), Some("大阪府"));
    }

    #[test]
    fn test_read_csv_empty() {
        let csv_data = "名前,年齢,住所\n".as_bytes().to_vec();

        let result = read_csv_for_test(csv_data);

        assert!(result.is_ok());
        let records = result.unwrap();
        assert_eq!(records.len(), 0);
    }

    #[test]
    fn test_read_csv_malformed() {
        let csv_data = "名前,年齢,住所\nテスト太郎,30\nテスト花子,25,大阪府,追加データ"
            .as_bytes()
            .to_vec();

        let result = read_csv_for_test(csv_data);

        if result.is_ok() {
            let records = result.unwrap();
            assert_eq!(records.len(), 2);
            assert_eq!(records[0].get(0), Some("テスト太郎"));
            assert_eq!(records[1].get(0), Some("テスト花子"));
        }
    }

    #[test]
    fn test_csv_error_display() {
        let decode_error = CSVError::DecodeError;
        assert_eq!(decode_error.to_string(), "デコードに失敗");

        let file_error = CSVError::FileReadError("テストエラー".to_string());
        assert_eq!(file_error.to_string(), "ファイル読み込み失敗: テストエラー");
    }
}
