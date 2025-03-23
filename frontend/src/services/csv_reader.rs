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
