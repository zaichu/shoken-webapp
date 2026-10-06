use crate::support::list_search::{is_js_whitespace, parse_search_tokens};

pub struct GroupKeyRule<'a, T> {
    pub test: fn(&T, &str) -> bool,
    pub key_fn: &'a dyn Fn(&T) -> String,
}

pub fn create_group_key_fn<'a, T>(
    search_query: &'a str,
    date_key_fn: fn(&T) -> String,
    rules: &'a [GroupKeyRule<'a, T>],
) -> Box<dyn Fn(&T) -> String + 'a> {
    if search_query.is_empty() {
        return Box::new(move |item: &T| date_key_fn(item));
    }

    let tokens = parse_search_tokens(search_query);

    Box::new(move |item: &T| {
        for rule in rules {
            if tokens.iter().any(|t| (rule.test)(item, t)) {
                return (rule.key_fn)(item);
            }
        }
        date_key_fn(item)
    })
}

pub fn derive_security_code_from_query<T>(
    query: &str,
    data: &[T],
    code_getter: fn(&T) -> &str,
    name_getter: fn(&T) -> &str,
) -> String {
    if query.is_empty() {
        return String::new();
    }

    let trimmed_start = query.trim_start_matches(is_js_whitespace);
    let code_end = trimmed_start
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_alphanumeric())
        .map(|(index, c)| index + c.len_utf8())
        .last()
        .unwrap_or(0);
    let label_match = (code_end > 0)
        .then(|| {
            let suffix = trimmed_start[code_end..].trim_start_matches(is_js_whitespace);
            suffix
                .starts_with([':', '：'])
                .then(|| trimmed_start[..code_end].to_ascii_lowercase())
        })
        .flatten();

    if let Some(lower_code) = label_match
        && let Some(item) = data
            .iter()
            .find(|item| code_getter(item).to_lowercase() == lower_code)
    {
        return code_getter(item).to_string();
    }

    let tokens = parse_search_tokens(query);
    if let Some(item) = data.iter().find(|item| {
        tokens.iter().any(|t| {
            code_getter(item).to_lowercase() == *t
                || shared::normalize::normalize_display_name(name_getter(item)).to_lowercase() == *t
        })
    }) {
        return code_getter(item).to_string();
    }

    String::new()
}

#[cfg(test)]
mod tests;
