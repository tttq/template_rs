use chrono::{DateTime, NaiveDateTime, Utc, TimeZone};
use serde::{self, Deserialize, Serializer, Deserializer};

const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&date.format(FORMAT).to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDateTime::parse_from_str(&s, FORMAT)
        .map_err(serde::de::Error::custom)
        .map(|dt| Utc.from_utc_datetime(&dt))
}

pub mod option {
    use super::*;

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(dt) => serializer.serialize_str(&dt.format(FORMAT).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<String>::deserialize(deserializer)?;
        match opt {
            Some(s) => {
                NaiveDateTime::parse_from_str(&s, FORMAT)
                    .map_err(serde::de::Error::custom)
                    .map(|dt| Some(Utc.from_utc_datetime(&dt)))
            }
            None => Ok(None),
        }
    }
}
