use actix_multipart::Field;
use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use encoding_rs::SHIFT_JIS;
use futures_util::StreamExt;
use regex::Regex;
use std::collections::HashMap;

const DATE_FORMAT: &str = "%Y/%m/%d";
const MONTH_FORMAT: &str = "%Y/%m";
const MONEY_REGEX: &str = r"^-?[\d,]+(\.\d+)?$";

struct Record {
    date: NaiveDate,
    fields: Vec<String>,
    dividend: f64,
    tax: f64,
    received: f64,
}

fn format_number(s: &str) -> String {
    let is_negative = s.starts_with('-');
    let s = if is_negative { &s[1..] } else { s };
    let s = s.replace(",", ""); // カンマを除去
    let parts: Vec<&str> = s.split('.').collect();
    let mut result = String::new();
    for (i, c) in parts[0].chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result = result.chars().rev().collect();
    if parts.len() > 1 {
        result.push('.');
        result.push_str(parts[1]);
    }
    if is_negative {
        format!("-{}", result)
    } else {
        result
    }
}

pub async fn read_csv_data(field: &mut Field) -> Result<String> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk.map_err(|e| anyhow!("Error reading multipart field: {}", e))?;
        bytes.extend_from_slice(&chunk);
    }

    let (cow, _, had_errors) = SHIFT_JIS.decode(&bytes);
    if had_errors {
        return Err(anyhow!("Error decoding Shift-JIS"));
    }
    Ok(cow.into_owned())
}

fn parse_csv_record(record: &csv::StringRecord, money_regex: &Regex) -> Result<Record> {
    let date = NaiveDate::parse_from_str(&record[0], DATE_FORMAT)
        .map_err(|e| anyhow!("Error parsing date: {}", e))?;

    let dividend: f64 = record[8]
        .replace(",", "")
        .parse()
        .map_err(|e| anyhow!("Error parsing dividend: {}", e))?;
    let tax: f64 = record[9]
        .replace(",", "")
        .parse()
        .map_err(|e| anyhow!("Error parsing tax: {}", e))?;
    let received: f64 = record[10]
        .replace(",", "")
        .parse()
        .map_err(|e| anyhow!("Error parsing received amount: {}", e))?;

    let fields = record
        .iter()
        .enumerate()
        .map(|(i, field)| {
            if i >= 6 && money_regex.is_match(field.trim()) {
                if field.trim() == "-" {
                    field.to_string()
                } else {
                    let formatted = format_number(field);
                    if formatted.starts_with('-') {
                        format!("<span class=\"negative-amount\">{}</span>", formatted)
                    } else {
                        formatted
                    }
                }
            } else {
                field.to_string()
            }
        })
        .collect();

    Ok(Record {
        date,
        fields,
        dividend,
        tax,
        received,
    })
}

fn generate_html_table(
    records: &[Record],
    group_totals: &HashMap<String, (f64, f64, f64)>,
    output_headers: &[&str],
) -> String {
    let mut table = String::from("<table><thead><tr>");
    for header in output_headers {
        table.push_str(&format!("<th>{}</th>", header));
    }
    table.push_str("</tr></thead><tbody>");

    for record in records {
        table.push_str("<tr>");
        for field in &record.fields {
            table.push_str(&format!("<td>{}</td>", field));
        }
        for _ in 0..3 {
            table.push_str("<td></td>");
        }
        table.push_str("</tr>");
    }

    let mut sorted_keys: Vec<_> = group_totals.keys().collect();
    sorted_keys.sort_by(|a, b| b.cmp(a));

    for key in sorted_keys {
        let (dividend, tax, received) = group_totals[key];
        table.push_str("<tr class=\"group-total\">");
        table.push_str(&format!("<td>{}</td>", key));
        for _ in 0..10 {
            table.push_str("<td></td>");
        }
        table.push_str(&format!(
            "<td>{}</td>",
            format_number(&dividend.to_string())
        ));
        table.push_str(&format!("<td>{}</td>", format_number(&tax.to_string())));
        table.push_str(&format!(
            "<td>{}</td>",
            format_number(&received.to_string())
        ));
        table.push_str("</tr>");
    }

    table.push_str("</tbody></table>");
    table
}

pub async fn process_csv(field: &mut Field, _csv_type: &str) -> Result<String> {
    let utf8_string = read_csv_data(field).await?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(utf8_string.as_bytes());

    let headers = rdr
        .headers()
        .map_err(|e| anyhow!("Error reading CSV headers: {}", e))?;
    let headers_len = headers.len();

    let output_headers = vec![
        "受渡日",
        "商品",
        "口座",
        "銘柄コード",
        "銘柄名",
        "受取通貨",
        "単価",
        "数量[株]",
        "配当・分配金(税引前)",
        "税額",
        "受取金額",
        "配当・分配金合計(税引前)",
        "税額合計",
        "受取金額",
    ];

    let money_regex = Regex::new(MONEY_REGEX).unwrap();
    let mut records = Vec::new();
    let mut group_totals: HashMap<String, (f64, f64, f64)> = HashMap::new();

    for result in rdr.records() {
        let csv_record = result.map_err(|e| anyhow!("Error parsing CSV record: {}", e))?;
        if csv_record.len() == headers_len {
            let record = parse_csv_record(&csv_record, &money_regex)?;
            let key = record.date.format(MONTH_FORMAT).to_string();
            group_totals
                .entry(key)
                .and_modify(|e| {
                    e.0 += record.dividend;
                    e.1 += record.tax;
                    e.2 += record.received;
                })
                .or_insert((record.dividend, record.tax, record.received));
            records.push(record);
        }
    }

    Ok(generate_html_table(
        &records,
        &group_totals,
        &output_headers,
    ))
}
