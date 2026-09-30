pub use shared::domain::DividendSummary as DividendTotals;
pub use shared::format::{format_currency, format_number};
pub use shared::normalize::{normalize_display_name, normalize_security_code};

fn valid_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return false;
    }
    let year = value[..4].parse::<u32>().unwrap_or(0);
    let month = value[5..7].parse::<u32>().unwrap_or(0);
    let day = value[8..].parse::<u32>().unwrap_or(0);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let maximum_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1..=maximum_day).contains(&day)
}

pub fn format_date(value: &str) -> String {
    if valid_iso_date(value) {
        format!("{}/{}/{}", &value[..4], &value[5..7], &value[8..])
    } else {
        "—".to_string()
    }
}

pub fn create_year_month_key(value: &str) -> String {
    if valid_iso_date(value) {
        value[..7].to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests;
