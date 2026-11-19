use serde::{Deserialize, Deserializer, Serializer};
use time::{Date, format_description::well_known::Iso8601};

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
    Date::parse(&value, &Iso8601::DATE).map_err(serde::de::Error::custom)
}
