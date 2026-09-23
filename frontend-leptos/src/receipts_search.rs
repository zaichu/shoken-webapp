use rust_decimal::Decimal;
use std::collections::HashSet;

pub struct FilterConfig<T> {
    pub string_fields: Option<Vec<fn(&T) -> &str>>,
    pub partial_string_fields: Option<Vec<fn(&T) -> &str>>,
    pub date_field: Option<fn(&T) -> &str>,
    pub year_search: bool,
    pub year_month_search: bool,
    pub date_search: bool,
    pub date_range_search: bool,
    pub amount_fields: Option<Vec<fn(&T) -> Decimal>>,
}

const LABEL_CODE_TOKEN_RE: &str = r"^([0-9a-z]+)[:：]$";

pub fn parse_search_tokens(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let token_regex = regex::Regex::new(r#""((?:\\.|[^"\\])*)"|(\S+)"#).unwrap();

    for cap in token_regex.captures_iter(query) {
        let raw_token = cap.get(1).or(cap.get(2)).map(|m| m.as_str()).unwrap_or("");
        let token = raw_token.replace(r#"\""#, "\"").trim().to_lowercase();
        if token.is_empty() {
            continue;
        }
        let label_code_re = regex::Regex::new(LABEL_CODE_TOKEN_RE).unwrap();
        let final_token = if let Some(caps) = label_code_re.captures(&token) {
            caps.get(1).map(|m| m.as_str()).unwrap_or(&token).to_string()
        } else {
            token
        };
        tokens.push(final_token);
    }
    tokens
}

fn is_valid_iso_date(value: &str) -> bool {
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

fn matches_year(date: &str, query: &str) -> bool {
    if date.len() < 4 {
        return false;
    }
    &date[..4] == query
}

fn matches_year_month(date: &str, query: &str) -> bool {
    if date.len() < 7 {
        return false;
    }
    &date[..7] == query
}

fn matches_date(date: &str, query: &str) -> bool {
    date == query
}

fn matches_date_range(date: &str, query: &str) -> bool {
    let sep_idx = query.find("..");
    if sep_idx.is_none() {
        return false;
    }
    let sep_idx = sep_idx.unwrap();
    let start = &query[..sep_idx];
    let end = &query[sep_idx + 2..];
    if !start.is_empty() && !is_valid_iso_date(start) {
        return false;
    }
    if !end.is_empty() && !is_valid_iso_date(end) {
        return false;
    }
    if start.is_empty() && end.is_empty() {
        return false;
    }
    if !start.is_empty() && !end.is_empty() && start > end {
        return false;
    }
    if !start.is_empty() && !end.is_empty() {
        date >= start && date <= end
    } else if !start.is_empty() {
        date >= start
    } else {
        date <= end
    }
}

fn matches_token<T>(
    item: &T,
    token: &str,
    config: &FilterConfig<T>,
) -> bool {
    if let Some(string_fields) = &config.string_fields {
        for getter in string_fields {
            if getter(item).to_lowercase() == token {
                return true;
            }
        }
    }

    if let Some(partial_string_fields) = &config.partial_string_fields {
        for getter in partial_string_fields {
            if getter(item).to_lowercase().contains(token) {
                return true;
            }
        }
    }

    if let Some(date_field) = config.date_field {
        let date = date_field(item);
        if config.year_search && matches_year(date, token) {
            return true;
        }
        if config.year_month_search && matches_year_month(date, token) {
            return true;
        }
        if config.date_search && matches_date(date, token) {
            return true;
        }
        if config.date_range_search && matches_date_range(date, token) {
            return true;
        }
    }

    if let Some(amount_fields) = &config.amount_fields {
        for getter in amount_fields {
            if getter(item).to_string().contains(token) {
                return true;
            }
        }
    }

    false
}

fn is_whitespace_char(c: char) -> bool {
    c.is_whitespace() || c == '\u{FEFF}'
}

pub fn filter_by_config<'a, T>(data: &'a [T], query: &str, config: &FilterConfig<T>) -> Vec<&'a T> {
    let trimmed = query.trim_matches(is_whitespace_char);
    if trimmed.is_empty() || trimmed.chars().all(is_whitespace_char) {
        return data.iter().collect();
    }

    let tokens = parse_search_tokens(trimmed);
    if tokens.is_empty() {
        return data.iter().collect();
    }

    data.iter()
        .filter(|item| tokens.iter().all(|token| matches_token(*item, token, config)))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchOption {
    pub value: String,
    pub label: String,
}

pub fn create_year_options<T>(data: &[T], date_getter: impl Fn(&T) -> &str) -> Vec<SearchOption> {
    let mut seen = HashSet::new();
    let mut options = Vec::new();

    for item in data {
        let date = date_getter(item);
        if date.len() >= 4 {
            let year = &date[..4];
            if seen.insert(year.to_string()) {
                options.push(SearchOption {
                    value: year.to_string(),
                    label: format!("{}年", year),
                });
            }
        }
    }

    options.sort_by(|a, b| a.value.cmp(&b.value));
    options
}

pub fn get_unique_values<T>(data: &[T], getter: impl Fn(&T) -> &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for item in data {
        let value = getter(item);
        let trimmed = value.trim();
        if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
            result.push(trimmed.to_string());
        }
    }

    result
}

#[cfg(test)]
mod tests;