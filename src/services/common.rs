use anyhow::anyhow;
use chrono::NaiveDate;

pub fn parse_date(date_str: Option<&str>) -> Option<NaiveDate> {
    match date_str {
        Some(date_str) => {
            let date = NaiveDate::parse_from_str(&date_str, "%Y/%m/%d")
                .map_err(|e| anyhow!("Failed to parse date '{}': {}", date_str, e));

            match date {
                Ok(date) => Some(date),
                Err(e) => {
                    println!("{e}");
                    None
                }
            }
        }
        None => None,
    }
}

pub fn parse_int(num_str: Option<&str>) -> Option<i32> {
    match num_str {
        Some(s) => match s.replace(",", "").parse::<i32>() {
            Ok(n) => Some(n),
            Err(e) => {
                println!("Failed to parse integer '{}': {}", s, e);
                None
            }
        },
        None => None,
    }
}

pub fn parse_float(num_str: Option<&str>) -> Option<f64> {
    match num_str {
        Some(s) => match s.replace(",", "").parse::<f64>() {
            Ok(n) => Some(n),
            Err(e) => {
                println!("Failed to parse float '{}': {}", s, e);
                None
            }
        },
        None => None,
    }
}

pub fn parse_string(value: Option<&str>) -> Option<String> {
    match value {
        Some(s) => Some(s.to_string()),
        None => None,
    }
}
