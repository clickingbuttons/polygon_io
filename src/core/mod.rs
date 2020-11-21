use serde::{de, Deserialize, Serialize};
use std::fmt;

pub mod aggs;
pub mod grouped;
pub mod last;

fn f64_to_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
  D: de::Deserializer<'de>
{
  struct JsonNumberVisitor;

  impl<'de> de::Visitor<'de> for JsonNumberVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("a u64")
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
      E: de::Error
    {
      Ok(value)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
      E: de::Error
    {
      let res = value as u64;
      if value == res as f64 {
        Ok(value as u64)
      } else {
        Err(E::custom(format!("cannot convert {} to u64", value)))
      }
    }
  }
  deserializer.deserialize_any(JsonNumberVisitor)
}

fn default_num_ticks() -> u64 { u64::MAX }

fn default_vwap() -> f32 { f32::NAN }
// This is shared between these two structures:
// 1. Grouped:
// { o, h, l, c, v, t, vw, n, T }
// 2. Aggs:
// { o, h, l, c, v, t, vw, n }
#[derive(Debug, Serialize, Deserialize)]
pub struct Candle {
  #[serde(rename(deserialize = "t"))]
  pub ts:        i64,
  #[serde(rename(deserialize = "T"), default)]
  pub symbol:    String,
  #[serde(rename(deserialize = "o"))]
  pub open:      f32,
  #[serde(rename(deserialize = "h"))]
  pub high:      f32,
  #[serde(rename(deserialize = "l"))]
  pub low:       f32,
  #[serde(rename(deserialize = "c"))]
  pub close:     f32,
  #[serde(rename(deserialize = "v"), deserialize_with = "f64_to_u64")]
  pub volume:    u64,
  #[serde(rename(deserialize = "vw"), default = "default_vwap")]
  pub vwap:      f32,
  #[serde(
    rename(deserialize = "n"),
    default = "default_num_ticks",
    deserialize_with = "f64_to_u64"
  )]
  pub num_ticks: u64
}

#[derive(Deserialize)]
pub struct CandleResponse {
  pub results: Vec<Candle>
}
