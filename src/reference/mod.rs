pub mod dividends;
pub mod financials;
pub mod splits;
pub mod tickers;
pub mod types;
pub mod markets;
pub mod locales;

use serde::{Deserialize, de, Serializer};
use chrono::NaiveDate;
use std::fmt;

const POLYGON_DATE_FORMAT: &str = "%Y-%m-%d";

pub fn string_to_naive_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
  D: de::Deserializer<'de>
{
  struct StringVisitor;

  impl<'de> de::Visitor<'de> for StringVisitor {
    type Value = NaiveDate;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str(&format!("a string of format {}", POLYGON_DATE_FORMAT))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
      E: de::Error
    {
      match NaiveDate::parse_from_str(&value, POLYGON_DATE_FORMAT) {
        Ok(date) => Ok(date),
        Err(_e) => Err(E::custom("Could not parse date"))
      }
    }
  }
  deserializer.deserialize_any(StringVisitor)
}

pub fn option_string_to_naive_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
  D: de::Deserializer<'de>
{
    #[derive(Deserialize)]
    struct Wrapper(#[serde(deserialize_with = "string_to_naive_date")] NaiveDate);

    let v = Option::deserialize(deserializer)?;
    Ok(v.map(|Wrapper(a)| a))
}

pub fn naive_date_to_string<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer
{
  serializer.serialize_str(&date.format(POLYGON_DATE_FORMAT).to_string())
}

pub fn option_naive_date_to_string<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer
{
  let string = match date {
    Some(date) => date.format(POLYGON_DATE_FORMAT).to_string(),
    None => String::from("null")
  };
  serializer.serialize_str(&string)
}
