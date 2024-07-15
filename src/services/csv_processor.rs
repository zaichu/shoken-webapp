use actix_multipart::Field;
use actix_web::web;
use anyhow::{anyhow, Result};
use chardetng::EncodingDetector;
use futures_util::StreamExt;

pub async fn process_csv(field: &mut Field) -> Result<String> {
    let mut bytes = web::BytesMut::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk.map_err(|e| anyhow!("Error reading multipart field: {}", e))?;
        bytes.extend_from_slice(&chunk);
    }

    let mut detector = EncodingDetector::new();
    detector.feed(&bytes, true);
    let encoding = detector.guess(None, true);
    let (decoded, _, _) = encoding.decode(&bytes);
    let utf8_string = decoded.into_owned();

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(utf8_string.as_bytes());

    let mut table = String::from("<table border=\"1\">");
    for result in rdr.records() {
        let record = result.map_err(|e| anyhow!("Error parsing CSV record: {}", e))?;
        table.push_str("<tr>");
        for field in record.iter() {
            table.push_str(&format!("<td>{}</td>", field));
        }
        table.push_str("</tr>");
    }
    table.push_str("</table>");

    Ok(table)
}
