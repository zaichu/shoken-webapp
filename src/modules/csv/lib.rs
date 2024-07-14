use csv::StringRecord;
use encoding_rs::{Encoding, SHIFT_JIS};
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

pub struct CSVAccessor;

impl CSVAccessor {
    pub fn read(filepath: &Path) -> Result<Vec<StringRecord>, Box<dyn Error>> {
        let file = File::open(filepath)?;
        let mut reader = BufReader::new(file);

        let encoding = Self::detect_encoding(&mut reader);
        // ファイルの先頭に戻す
        reader.seek(SeekFrom::Start(0))?;

        let transcoded_reader = DecodeReaderBytesBuilder::new()
            .encoding(encoding)
            .build(reader);

        let mut csv_reader = csv::Reader::from_reader(transcoded_reader);
        let mut result = Vec::new();
        for record in csv_reader.records() {
            result.push(record?);
        }
        Ok(result)
    }

    fn detect_encoding(reader: &mut BufReader<File>) -> Option<&'static Encoding> {
        // ファイルの先頭から最大3バイト読み込み、UTF-8として正しく解釈できるかを判断する
        let mut buf = [0; 3];
        if let Ok(bytes_read) = reader.read(&mut buf) {
            if let Ok(_) = std::str::from_utf8(&buf[..bytes_read]) {
                // UTF-8として正常に解釈できる場合、UTF-8と判定する
                return None;
            }
        }
        Some(SHIFT_JIS)
    }
}
