pub use shared::domain::DividendSummary as DividendTotals;
pub use shared::normalize::normalize_security_code;

use crate::support::list_search::is_valid_iso_date;

pub fn format_date(value: &str) -> String {
    if is_valid_iso_date(value) {
        format!("{}/{}/{}", &value[..4], &value[5..7], &value[8..])
    } else {
        "—".to_string()
    }
}

pub fn create_year_month_key(value: &str) -> String {
    if is_valid_iso_date(value) {
        value[..7].to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests;
