extern crate serde_json;
extern crate ureq;

use crate::{
  client::Client,
  helpers::{make_params, to_conditions},
  with_param
};
use chrono::NaiveDate;
use serde::{de, Deserialize, Serialize};
use std::{
  collections::HashMap,
  fmt,
  io::{self, Error, ErrorKind}
};

// Trade Corrections (NYSE)
// Modifier	Indicator
// 00	Regular trade which was not corrected, changed or signified as cancel or error.
// 01	Original trade which was late corrected (This record contains the original time - HHMM and the corrected data for the trade).
// 07	Original trade which was later marked as erroreous
// 08	Original trade which was later cancelled
// 10	Cancel record (This record follows '08' records)
// 11	Error record (This record follows '07' records)
// 12	Correction record (This record follows'01' records and contains the correction time and the original "incorrect" data). The final correction will be published.
fn to_error<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
  D: de::Deserializer<'de>
{
  struct JsonNumberVisitor;

  impl<'de> de::Visitor<'de> for JsonNumberVisitor {
    type Value = u8;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("an int in [0, 1, 7, 8, 10, 11, 12]")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
      E: de::Error
    {
      match v {
        0 => Ok(0),
        1 => Ok(1),
        7 => Ok(7),
        8 => Ok(8),
        10 => Ok(10),
        11 => Ok(11),
        12 => Ok(12),
        c => Err(de::Error::custom(&format!("bad correction {}", c)))
      }
    }
  }
  deserializer.deserialize_any(JsonNumberVisitor)
}

// Trade ID:
// Up to 8 char string in 2015
// Gone in 2017
// Back as u64 string in 2018
fn to_id<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
  D: de::Deserializer<'de>
{
  struct JsonStringVisitor;

  impl<'de> de::Visitor<'de> for JsonStringVisitor {
    type Value = u64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("a string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: de::Error
    {
      let mut res = [0; 8];
      if v.len() <= 8 {
        let bytes = &v.as_bytes();
        res[0..bytes.len()].copy_from_slice(bytes);
        let res = u64::from_be_bytes(res);
        Ok(res)
      } else if v.len() <= 20 {
        let res = v.parse::<u64>().expect("Parseable u64");
        Ok(res)
      } else {
        Err(de::Error::custom(&format!("bad trade id {}", v)))
      }
    }
  }
  deserializer.deserialize_any(JsonStringVisitor)
}

// v3 returns things like "size":2.216834e+06
fn f64_to_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
  D: de::Deserializer<'de>
{
  struct JsonNumberVisitor;

  impl<'de> de::Visitor<'de> for JsonNumberVisitor {
    type Value = u32;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("a number castable to u32")
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
      E: de::Error
    {
      Ok(v as u32)
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
      E: de::Error
    {
      Ok(v as u32)
    }
  }
  deserializer.deserialize_any(JsonNumberVisitor)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
  #[serde(rename(deserialize = "sip_timestamp"))]
  pub ts: i64,
  #[serde(rename(deserialize = "participant_timestamp"))]
  pub ts_participant: Option<i64>,
  #[serde(rename(deserialize = "trf_timestamp"))]
  pub ts_trf: Option<i64>,
  #[serde(default)]
  pub symbol: String,
  #[serde(deserialize_with = "f64_to_u32", default)]
  pub size: u32,
  #[serde(default)]
  pub price: f64,
  #[serde(deserialize_with = "to_conditions", default)]
  pub conditions: u32,
  #[serde(
    rename(deserialize = "correction"),
    deserialize_with = "to_error",
    default
  )]
  pub error: u8,
  pub exchange: u8,
  pub tape: u8,
  #[serde(deserialize_with = "to_id", default)]
  pub id: u64,
  #[serde(rename(deserialize = "sequence_number"))]
  pub seq_id: u64
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradesResponse {
  pub results:  Vec<Trade>,
  pub next_url: Option<String>,
  pub status:   String, // For debugging
  pub uri:      Option<String>
}

pub struct TradesParams<'a> {
  pub params: HashMap<&'a str, String>
}

impl<'a> TradesParams<'a> {
  with_param!(timestamp, &str);

  with_param!(timestamp_lt, &str);

  with_param!(timestamp_lte, &str);

  with_param!(timestamp_gt, &str);

  with_param!(timestamp_gte, &str);

  with_param!(order, &str);

  with_param!(reverse, bool);

  with_param!(limit, usize);

  // Undocumented but appears in next_page_path
  with_param!(cursor, &str);

  pub fn new() -> Self {
    Self {
      params: HashMap::with_capacity(4)
    }
  }
}

impl Client {
  pub fn get_trades(
    &mut self,
    symbol: &str,
    params: Option<&HashMap<&str, String>>
  ) -> io::Result<TradesResponse> {
    let uri = format!(
      "{}/v3/trades/{}{}",
      self.api_uri,
      symbol,
      make_params(params),
    );

    let mut resp = self.get_response::<TradesResponse>(&uri)?;
    resp.uri = Some(uri);

    if resp.results.len() == 0 {
      return Err(Error::new(ErrorKind::UnexpectedEof, "Results is empty"));
    }

    for row in resp.results.iter_mut() {
      row.symbol = symbol.to_string();
    }

    Ok(resp)
  }

  pub fn get_all_trades(&mut self, symbol: &str, date: NaiveDate) -> io::Result<Vec<Trade>> {
    let limit: usize = 50_000;
    let mut params = TradesParams::new()
      .limit(limit)
      .timestamp(&date.format("%Y-%m-%d").to_string());
    let mut res = Vec::<Trade>::new();
    loop {
      let page = self.get_trades(symbol, Some(&params.params))?;
      res.extend(page.results.into_iter());
      match page.next_url {
        Some(next_url) => {
          let split = next_url.split("cursor=").collect::<Vec<&str>>();
          if split.len() != 2 {
            let msg = format!("no cursor in next_url {}", next_url);
            return Err(Error::new(ErrorKind::UnexpectedEof, msg));
          }
          let cursor = split[1];
          params = TradesParams::new().cursor(cursor);
        }
        None => break
      };
    }

    Ok(res)
  }
}

#[cfg(test)]
mod trades {
  use crate::{client::Client, equities::trades::TradesParams};
  use chrono::NaiveDate;

  #[test]
  fn appl_2004_works() {
    let mut client = Client::new();
    let params = TradesParams::new()
      .timestamp("2004-01-02")
      .limit(50_000)
      .params;
    let trades = client.get_trades("AAPL", Some(&params)).unwrap();
    let count = 7_452;
    assert_eq!(trades.results.len(), count);
  }

  #[test]
  fn limit_works() {
    let mut client = Client::new();
    let limit = 500;
    let params = TradesParams::new()
      .limit(limit)
      .timestamp("2004-01-02")
      .params;
    let trades = client.get_trades("AAPL", Some(&params)).unwrap();
    assert_eq!(trades.results.len(), limit);
  }

  #[test]
  fn get_all_works() {
    let mut client = Client::new();
    let date = NaiveDate::from_ymd(2020, 01, 02);
    let trades = client.get_all_trades("AAPL", date).unwrap();
    let count = 283_504;
    assert_eq!(trades.len(), count);
  }
}
