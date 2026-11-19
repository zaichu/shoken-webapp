use serde::{Deserialize, Deserializer, Serializer};
use time::Date;

const ISO_DATE: &[time::format_description::FormatItem<'static>] =
    time::macros::format_description!("[year]-[month padding:none]-[day padding:none]");

pub fn serialize<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_str(date)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    Date::parse(&value, ISO_DATE).map_err(serde::de::Error::custom)
}
