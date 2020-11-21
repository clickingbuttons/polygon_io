use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{de, Deserialize, Serializer};
use std::{
  collections::HashMap,
  fmt,
  io::{Error, ErrorKind}
};
use ureq::Response;

pub fn make_params(params: &HashMap<&str, String>) -> String {
  params
    .iter()
    .map(|(key, val)| format!("&{}={}", key, val))
    .collect::<Vec<String>>()
    .join("")
}

pub fn get_response(uri: &str) -> std::io::Result<Response> {
  let resp = ureq::get(&uri)
    .timeout_connect(10_000)
    .timeout_read(10_000)
    .call();

  let status = resp.status();
  if status != 200 {
    return Err(Error::new(
      ErrorKind::NotConnected,
      format!("Server returned {}", status)
    ));
  }
  Ok(resp)
}

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

pub fn option_naive_date_to_string<S>(
  date: &Option<NaiveDate>,
  serializer: S
) -> Result<S::Ok, S::Error>
where
  S: Serializer
{
  let string = match date {
    Some(date) => date.format(POLYGON_DATE_FORMAT).to_string(),
    None => String::from("null")
  };
  serializer.serialize_str(&string)
}

pub fn string_to_datetime<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
where
  D: de::Deserializer<'de>
{
  struct StringVisitor;

  impl<'de> de::Visitor<'de> for StringVisitor {
    type Value = DateTime<FixedOffset>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("a rfc 3339 string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
      E: de::Error
    {
      match DateTime::parse_from_rfc3339(&value) {
        Ok(date) => Ok(date),
        Err(_e) => Err(E::custom("Could not parse date"))
      }
    }
  }
  deserializer.deserialize_any(StringVisitor)
}

pub fn datetime_to_string<S>(date: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer
{
  serializer.serialize_str(&date.to_rfc3339())
}

pub fn option_string_to_datetime<'de, D>(
  deserializer: D
) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
  D: de::Deserializer<'de>
{
  #[derive(Deserialize)]
  struct Wrapper(#[serde(deserialize_with = "string_to_datetime")] DateTime<FixedOffset>);

  let v = Option::deserialize(deserializer)?;
  Ok(v.map(|Wrapper(a)| a))
}

pub fn option_datetime_to_string<S>(
  date: &Option<DateTime<FixedOffset>>,
  serializer: S
) -> Result<S::Ok, S::Error>
where
  S: Serializer
{
  let string = match date {
    Some(date) => date.to_rfc3339(),
    None => String::from("null")
  };
  serializer.serialize_str(&string)
}
