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
