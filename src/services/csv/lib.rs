use anyhow::{anyhow, Result};
use csv::StringRecord;
use encoding_rs::SHIFT_JIS;

pub struct CSVAccessor;

impl CSVAccessor {
    pub fn read(bytes: Vec<u8>) -> Result<Vec<StringRecord>> {
        let (cow, _, had_errors) = SHIFT_JIS.decode(&bytes);
        if had_errors {
            return Err(anyhow!("Error decoding Shift-JIS"));
        }
        let utf8_string = cow.into_owned();
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(utf8_string.as_bytes());

        let mut result = Vec::new();
        for record in rdr.records() {
            result.push(record?);
        }
        Ok(result)
    }
}
