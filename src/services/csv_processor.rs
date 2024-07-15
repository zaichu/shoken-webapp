use actix_multipart::Field;
use anyhow::{anyhow, Result};
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

pub async fn process_csv(field: &mut Field, csv_type: &str) -> Result<String> {
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
    let cleaned_headers: Vec<String> = headers.iter().map(|h| clean_header(h)).collect();

    let mut table = String::from("<table>");
    table.push_str("<thead><tr>");
    for header in &cleaned_headers {
        table.push_str(&format!("<th>{}</th>", header));
    }
    table.push_str("</tr></thead><tbody>");

    let money_regex = Regex::new(r"^-?[\d,]+(\.\d+)?$").unwrap();
    let mut current_group = Vec::new();
    let mut group_totals: HashMap<String, Vec<String>> = HashMap::new();

    let money_start_index = if csv_type == "dividend" { 6 } else { 7 };

    for result in rdr.records() {
        let record = result.map_err(|e| anyhow!("Error parsing CSV record: {}", e))?;

        if record.len() == cleaned_headers.len() {
            // Regular row
            table.push_str("<tr>");
            for (i, field) in record.iter().enumerate() {
                let formatted_field =
                    if i >= money_start_index && money_regex.is_match(field.trim()) {
                        if field.trim() == "-" {
                            field.to_string()
                        } else {
                            let formatted = format_number(&field.replace(",", ""));
                            if formatted.starts_with('-') {
                                format!("<span class=\"negative-amount\">{}</span>", formatted)
                            } else {
                                format!("{}", formatted)
                            }
                        }
                    } else {
                        field.to_string()
                    };
                table.push_str(&format!("<td>{}</td>", formatted_field));
            }
            table.push_str("</tr>");
            current_group.push(record);
        } else if record.len() == 3 {
            // Total row
            if !current_group.is_empty() {
                table.push_str("<tr class=\"group-total\">");
                for _ in 0..cleaned_headers.len() - 3 {
                    table.push_str("<td></td>");
                }
                for field in record.iter() {
                    let formatted_field = if money_regex.is_match(field.trim()) {
                        if field.trim() == "-" {
                            field.to_string()
                        } else {
                            let formatted = format_number(&field.replace(",", ""));
                            if formatted.starts_with('-') {
                                format!("<span class=\"negative-amount\">{}</span>", formatted)
                            } else {
                                format!("{}", formatted)
                            }
                        }
                    } else {
                        field.to_string()
                    };
                    table.push_str(&format!("<td>{}</td>", formatted_field));
                }
                table.push_str("</tr>");

                // Store group totals
                group_totals.insert(
                    current_group[0][0].to_string(),
                    record.iter().map(|s| s.to_string()).collect(),
                );

                current_group.clear();
            }
        }
    }

    table.push_str("</tbody></table>");

    Ok(table)
}
