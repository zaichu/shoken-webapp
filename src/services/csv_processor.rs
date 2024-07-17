use actix_multipart::Field;
use anyhow::{anyhow, Result};
use chrono::NaiveDate;
use encoding_rs::SHIFT_JIS;
use futures_util::StreamExt;
use regex::Regex;
use std::collections::HashMap;

fn format_number(s: &str) -> String {
    let is_negative = s.starts_with('-');
    let s = if is_negative { &s[1..] } else { s };
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

fn clean_header(header: &str) -> String {
    let re = Regex::new(r"\[.*?\]").unwrap();
    re.replace_all(header, "").trim().to_string()
}

pub async fn process_csv(field: &mut Field, _csv_type: &str) -> Result<String> {
    let mut bytes = Vec::new();
    while let Some(chunk) = field.next().await {
        let chunk = chunk.map_err(|e| anyhow!("Error reading multipart field: {}", e))?;
        bytes.extend_from_slice(&chunk);
    }

    let (cow, _, had_errors) = SHIFT_JIS.decode(&bytes);
    if had_errors {
        return Err(anyhow!("Error decoding Shift-JIS"));
    }
    let utf8_string = cow.into_owned();

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

    let mut table = String::from("<table>");
    table.push_str("<thead><tr>");
    for header in &output_headers {
        table.push_str(&format!("<th>{}</th>", header));
    }
    table.push_str("</tr></thead><tbody>");

    let money_regex = Regex::new(r"^-?[\d,]+(\.\d+)?$").unwrap();
    let mut current_group = Vec::new();
    let mut group_totals: HashMap<String, (f64, f64, f64)> = HashMap::new();

    for result in rdr.records() {
        let record = result.map_err(|e| anyhow!("Error parsing CSV record: {}", e))?;

        if record.len() == headers_len {
            // Regular row
            table.push_str("<tr>");
            for (i, field) in record.iter().enumerate() {
                let formatted_field = if i >= 6 && money_regex.is_match(field.trim()) {
                    if field.trim() == "-" {
                        field.to_string()
                    } else {
                        let formatted = format_number(field);
                        if formatted.starts_with('-') {
                            format!("<span class=\"negative-amount\">¥{}</span>", formatted)
                        } else {
                            format!("¥{}", formatted)
                        }
                    }
                } else {
                    field.to_string()
                };
                table.push_str(&format!("<td>{}</td>", formatted_field));
            }
            for _ in 0..3 {
                table.push_str("<td></td>");
            }
            table.push_str("</tr>");
            current_group.push(record.clone());

            // Update group totals
            if let Ok(date) = NaiveDate::parse_from_str(&record[0], "%Y/%m/%d") {
                let key = date.format("%Y/%m").to_string();
                let dividend: f64 = record[8].replace(",", "").parse().unwrap_or(0.0);
                let tax: f64 = record[9].replace(",", "").parse().unwrap_or(0.0);
                let received: f64 = record[10].replace(",", "").parse().unwrap_or(0.0);
                group_totals
                    .entry(key)
                    .and_modify(|e| {
                        e.0 += dividend;
                        e.1 += tax;
                        e.2 += received;
                    })
                    .or_insert((dividend, tax, received));
            }
        }
    }

    // Display group totals
    let mut sorted_keys: Vec<_> = group_totals.keys().collect();
    sorted_keys.sort_by(|a, b| b.cmp(a)); // Sort in descending order

    for key in sorted_keys {
        let (dividend, tax, received) = group_totals[key];
        table.push_str("<tr class=\"group-total\">");
        table.push_str(&format!("<td>{}</td>", key));
        for _ in 0..10 {
            table.push_str("<td></td>");
        }
        table.push_str(&format!(
            "<td>¥{}</td>",
            format_number(&dividend.to_string())
        ));
        table.push_str(&format!("<td>¥{}</td>", format_number(&tax.to_string())));
        table.push_str(&format!(
            "<td>¥{}</td>",
            format_number(&received.to_string())
        ));
        table.push_str("</tr>");
    }

    table.push_str("</tbody></table>");

    Ok(table)
}
