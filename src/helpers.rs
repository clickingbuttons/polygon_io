use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{de, de::SeqAccess, Deserialize, Serializer};
use std::{collections::HashMap, fmt};

pub fn make_params(params: Option<&HashMap<&str, String>>) -> String {
  if params.is_none() {
    return String::new();
  }
  let params = params.unwrap();
  let mut res = String::from("?");
  let kvs = params
    .iter()
    .map(|(key, val)| format!("{}={}", key.to_lowercase(), val))
    .collect::<Vec<String>>();

  if kvs.len() == 0 {
    return String::new();
  }

  res.push_str(&kvs.join("&"));
  return res;
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

#[macro_export]
macro_rules! with_param {
  ($param:ident, $_type: ty) => {
    pub fn $param(mut self, $param: $_type) -> Self {
      self.params.insert(stringify!($param), $param.to_string());
      self
    }
  };
}

// 4 conditions, each 1 byte
// 1: Settlement type
//     @, C,  N,  R,  Y
//     0, 7, 20, 29, 36
// 2: Reason for trade through exempt / other reason
//     F,  O,  O,  4,  5, 6,  7,  8,  9
//    14, 17, 25, 10, 28, 8, 53, 59, 38
// 3: Extended hours / sequence type
//     L,  T,  U,  Z
//    30, 12, 13, 33
// 4: Self regulatory organization trade detail
//     A, B,  D,  E, E, G,  H,  I,  K,  M,  P,  Q,  S, W, X
//     1, 4, 11, 56, 3, 5, 21, 37, 23, 15, 22, 16, 34, 2, 9
//
// Polygon has extra conditions like CAP election, Held, etc.
//
// https://www.utpplan.com/DOC/UtpBinaryOutputSpec.pdf Page 45
// https://www.ctaplan.com/publicdocs/ctaplan/notifications/trader-update/CTS_BINARY_OUTPUT_SPECIFICATION.pdf Page 64
// https://polygon.io/glossary/us/stocks/conditions-indicators has 0-59 (much less than 256)
pub fn to_conditions<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
  D: de::Deserializer<'de>
{
  struct JsonNumberArrVisitor;

  impl<'de> de::Visitor<'de> for JsonNumberArrVisitor {
    type Value = u32;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("an array of u8")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: SeqAccess<'de>
    {
      let mut conditions: [u32; 4] = [
        seq.next_element()?.unwrap_or(0),
        seq.next_element()?.unwrap_or(0),
        seq.next_element()?.unwrap_or(0),
        seq.next_element()?.unwrap_or(0)
      ];
      conditions.sort();
      if seq.next_element::<u32>()?.is_some() {
        return Err(de::Error::custom("trades must have 4 or less conditions"));
      }
      let mut res: u32 = 0;
      for (i, c) in conditions.iter().enumerate() {
        if *c > 256 {
          return Err(de::Error::custom(&format!(
            "condition {} should be < 256",
            *c
          )));
        }
        res |= c << (8 * i);
      }
      Ok(res)
    }
  }
  deserializer.deserialize_any(JsonNumberArrVisitor)
}
