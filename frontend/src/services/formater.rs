use crate::setting::*;

pub fn format_value(key: &str, value: &str) -> String {
    if YEN_FORMAT_KEYS.contains(key) {
        value.format_yen()
    } else if NUMBER_FORMAT_KEYS.contains(key) {
        value.format_number()
    } else if DATE_FORMAT_KEYS.contains(key) {
        value.format_date()
    } else {
        value.to_string()
    }
}

pub trait StrFormater {
    fn format_date(&self) -> String;
    fn format_number(&self) -> String;
    fn format_yen(&self) -> String;
}

impl StrFormater for &str {
    fn format_date(&self) -> String {
        self.replace("-", "/")
    }

    fn format_number(&self) -> String {
        let (sign, s) = self.strip_prefix('-').map_or(("", *self), |s| ("-", s));
        let parts: Vec<&str> = s.split('.').collect();
        let integer_part = parts[0]
            .chars()
            .rev()
            .enumerate()
            .fold(String::new(), |mut acc, (i, c)| {
                if i > 0 && i % 3 == 0 {
                    acc.push(',');
                }
                acc.push(c);
                acc
            })
            .chars()
            .rev()
            .collect::<String>();
        format!(
            "{}{}{}",
            sign,
            integer_part,
            parts.get(1).map_or(String::new(), |&s| format!(".{}", s))
        )
    }

    fn format_yen(&self) -> String {
        if self.is_empty() {
            return "".to_string();
        }
        if *self == "-" {
            return "-".to_string();
        }
        format!("¥ {}", self.format_number())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date() {
        assert_eq!("2023/12/31", "2023-12-31".format_date());
        assert_eq!("2020/01/01", "2020-01-01".format_date());
    }

    #[test]
    fn test_format_number() {
        assert_eq!("1,234,567", "1234567".format_number());
        assert_eq!("-1,234,567", "-1234567".format_number());
        assert_eq!("1,234.56", "1234.56".format_number());
        assert_eq!("123", "123".format_number());
    }

    #[test]
    fn test_format_yen() {
        assert_eq!("¥ 1,000", "1000".format_yen());
        assert_eq!("¥ -1,000", "-1000".format_yen());
        assert_eq!("", "".format_yen());
        assert_eq!("-", "-".format_yen());
    }
}
