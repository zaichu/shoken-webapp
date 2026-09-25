use rust_decimal::Decimal;
use std::collections::HashSet;

type StringFieldFn<T> = fn(&T) -> &str;
type AmountFieldFn<T> = fn(&T) -> Decimal;

pub struct FilterConfig<T> {
    pub string_fields: Option<Vec<StringFieldFn<T>>>,
    pub partial_string_fields: Option<Vec<StringFieldFn<T>>>,
    pub date_field: Option<StringFieldFn<T>>,
    pub year_search: bool,
    pub year_month_search: bool,
    pub date_search: bool,
    pub date_range_search: bool,
    pub amount_fields: Option<Vec<AmountFieldFn<T>>>,
}

pub(crate) fn is_js_whitespace(c: char) -> bool {
    matches!(
        c,
        '\u{0009}'
            ..='\u{000D}'
                | '\u{0020}'
                | '\u{00A0}'
                | '\u{1680}'
                | '\u{2000}'..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

fn is_js_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{2028}' | '\u{2029}')
}

fn parse_quoted_token(query: &str, mut index: usize) -> Option<(String, usize)> {
    let mut token = String::new();

    while index < query.len() {
        let current = query[index..].chars().next()?;
        if current == '"' {
            return Some((token, index + current.len_utf8()));
        }
        if current == '\\' {
            let escaped_index = index + current.len_utf8();
            let escaped = query[escaped_index..].chars().next()?;
            if is_js_line_terminator(escaped) {
                return None;
            }
            token.push(current);
            token.push(escaped);
            index = escaped_index + escaped.len_utf8();
            continue;
        }
        token.push(current);
        index += current.len_utf8();
    }

    None
}

fn normalize_search_token(raw_token: &str) -> Option<String> {
    let token = raw_token
        .replace(r#"\""#, "\"")
        .trim_matches(is_js_whitespace)
        .to_lowercase();
    if token.is_empty() {
        return None;
    }

    let label = token.strip_suffix(':').or_else(|| token.strip_suffix('：'));
    if let Some(code) = label.filter(|code| {
        !code.is_empty()
            && code
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte.is_ascii_lowercase())
    }) {
        return Some(code.to_string());
    }

    Some(token)
}

pub fn parse_search_tokens(query: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < query.len() {
        while index < query.len() {
            let current = query[index..].chars().next().unwrap();
            if !is_js_whitespace(current) {
                break;
            }
            index += current.len_utf8();
        }
        if index == query.len() {
            break;
        }

        let start = index;
        let current = query[index..].chars().next().unwrap();
        let (raw_token, next_index) = if current == '"' {
            parse_quoted_token(query, index + current.len_utf8()).unwrap_or_else(|| {
                let mut end = start;
                while end < query.len() {
                    let c = query[end..].chars().next().unwrap();
                    if is_js_whitespace(c) {
                        break;
                    }
                    end += c.len_utf8();
                }
                (query[start..end].to_string(), end)
            })
        } else {
            let mut end = start;
            while end < query.len() {
                let c = query[end..].chars().next().unwrap();
                if is_js_whitespace(c) {
                    break;
                }
                end += c.len_utf8();
            }
            (query[start..end].to_string(), end)
        };
        index = next_index;

        let Some(token) = normalize_search_token(&raw_token) else {
            continue;
        };
        tokens.push(token);
    }

    tokens
}

pub(crate) fn is_valid_iso_date(value: &str) -> bool {
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

fn matches_token<T>(item: &T, token: &str, config: &FilterConfig<T>) -> bool {
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

pub fn filter_by_config<'a, T>(data: &'a [T], query: &str, config: &FilterConfig<T>) -> Vec<&'a T> {
    let trimmed = query.trim_matches(is_js_whitespace);
    if trimmed.is_empty() {
        return data.iter().collect();
    }

    let tokens = parse_search_tokens(trimmed);
    if tokens.is_empty() {
        return data.iter().collect();
    }

    data.iter()
        .filter(|item| {
            tokens
                .iter()
                .all(|token| matches_token(*item, token, config))
        })
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
        let trimmed = value.trim_matches(is_js_whitespace);
        if !trimmed.is_empty() && seen.insert(value.to_string()) {
            result.push(value.to_string());
        }
    }

    result
}

#[cfg(test)]
mod tests;
