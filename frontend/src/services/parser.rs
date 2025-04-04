use chrono::NaiveDate;
use std::str::FromStr;

pub trait OptionalStrParser {
    fn try_parse_date(&self) -> Option<NaiveDate>;
    fn try_parse_num<T: FromStr>(&self) -> Option<T>;
    fn try_parse_string(&self) -> Option<String>;
}

impl OptionalStrParser for Option<&str> {
    fn try_parse_date(&self) -> Option<NaiveDate> {
        self.and_then(|s| NaiveDate::parse_from_str(s, "%Y/%m/%d").ok())
    }

    fn try_parse_num<T: FromStr>(&self) -> Option<T> {
        self.and_then(|s| s.replace(",", "").parse().ok())
    }

    fn try_parse_string(&self) -> Option<String> {
        self.map(ToString::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_try_parse_date_valid() {
        let date_str = Some("2025/04/05");
        let expected_date = NaiveDate::from_ymd_opt(2025, 4, 5);
        assert_eq!(date_str.try_parse_date(), expected_date);
    }

    #[test]
    fn test_try_parse_date_invalid_format() {
        let date_str = Some("2025-04-05");
        assert_eq!(date_str.try_parse_date(), None);
    }

    #[test]
    fn test_try_parse_date_none() {
        let date_str: Option<&str> = None;
        assert_eq!(date_str.try_parse_date(), None);
    }

    #[test]
    fn test_try_parse_num_integer() {
        let num_str = Some("123");
        let expected: Option<i32> = Some(123);
        assert_eq!(num_str.try_parse_num::<i32>(), expected);
    }

    #[test]
    fn test_try_parse_num_float() {
        let num_str = Some("123.45");
        let expected: Option<f64> = Some(123.45);
        assert_eq!(num_str.try_parse_num::<f64>(), expected);
    }

    #[test]
    fn test_try_parse_num_with_commas() {
        let num_str = Some("1,234,567");
        let expected: Option<i32> = Some(1234567);
        assert_eq!(num_str.try_parse_num::<i32>(), expected);
    }

    #[test]
    fn test_try_parse_num_invalid() {
        let num_str = Some("abc");
        let expected: Option<i32> = None;
        assert_eq!(num_str.try_parse_num::<i32>(), expected);
    }

    #[test]
    fn test_try_parse_num_none() {
        let num_str: Option<&str> = None;
        let expected: Option<i32> = None;
        assert_eq!(num_str.try_parse_num::<i32>(), expected);
    }

    #[test]
    fn test_try_parse_string_some() {
        let str_val = Some("テスト文字列");
        assert_eq!(str_val.try_parse_string(), Some("テスト文字列".to_string()));
    }

    #[test]
    fn test_try_parse_string_none() {
        let str_val: Option<&str> = None;
        assert_eq!(str_val.try_parse_string(), None);
    }
}
