pub use shared::domain::DividendSummary as DividendTotals;
pub use shared::normalize::normalize_security_code;

use crate::support::list_search::is_valid_iso_date;

pub fn create_year_month_key(value: &str) -> String {
    if is_valid_iso_date(value) {
        value[..7].to_string()
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests;
