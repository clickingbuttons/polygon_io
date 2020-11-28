extern crate serde_json;
extern crate ureq;

use crate::{
  client::Client,
  helpers::{get_response, make_params}
};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize, de, de::SeqAccess};
use std::{
  collections::HashMap,
  io::{self, Error, ErrorKind},
  fmt
};

fn to_conditions<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
  D: de::Deserializer<'de>
{
  struct JsonNumberArrVisitor;

  impl<'de> de::Visitor<'de> for JsonNumberArrVisitor {
    type Value = i64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("an array of u32")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
      A: SeqAccess<'de>
    {
      let mut conditions: [i64; 4] = [
        seq.next_element()?.unwrap_or(0),
        seq.next_element()?.unwrap_or(0),
        seq.next_element()?.unwrap_or(0),
        seq.next_element()?.unwrap_or(0),
      ];
      conditions.sort();
      if seq.next_element::<i64>()?.is_some() {
        return Err(de::Error::custom("trades must have 4 or less conditions"));
      }
      let mut res: i64 = 0;
      for (i, c) in conditions.iter().enumerate() {
        if *c > 512 {
          return Err(de::Error::custom(&format!("condition {} should be < 512", *c)));
        }
        res |= c << (8 * i);
      }
      Ok(res)
    }
  }
  deserializer.deserialize_any(JsonNumberArrVisitor)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
  #[serde(rename(deserialize = "t"))]
  pub ts: i64,
  // #[serde(rename(deserialize = "y"))]
  // pub ts_participant: Option<i64>,
  // #[serde(rename(deserialize = "f"))]
  // pub ts_trf: Option<i64>,
  #[serde(default)]
  pub symbol: String,
  #[serde(rename(deserialize = "x"))]
  pub exchange: u8,
  #[serde(rename(deserialize = "s"))]
  pub size: u32,
  #[serde(rename(deserialize = "c"), deserialize_with = "to_conditions", default)]
  pub conditions: i64,
  #[serde(rename(deserialize = "p"))]
  pub price: f32,
  #[serde(rename(deserialize = "z"))]
  pub tape: u8
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TradesResponse {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol: String,
  pub results_count: usize,
  pub results: Vec<Trade>,
  // Map is useless
  // For debugging
  pub success: bool,
  pub db_latency: usize,
  pub uri: Option<String>
}

pub struct TradesParams<'a> {
  pub params: HashMap<&'a str, String>
}

impl<'a> TradesParams<'a> {
  pub fn new() -> Self {
    Self {
      params: HashMap::with_capacity(4)
    }
  }

  pub fn with_timestamp(mut self, timestamp: i64) -> Self {
    self.params.insert("timestamp", timestamp.to_string());
    self
  }

  pub fn with_timestamp_limit(mut self, timestamp_limit: i64) -> Self {
    self
      .params
      .insert("timestamp_limit", timestamp_limit.to_string());
    self
  }

  pub fn with_reverse(mut self, reverse: bool) -> Self {
    self.params.insert("reverse", reverse.to_string());
    self
  }

  pub fn with_limit(mut self, limit: usize) -> Self {
    self.params.insert("limit", limit.to_string());
    self
  }
}

impl Client {
  pub fn get_trades(
    &self,
    symbol: &str,
    date: NaiveDate,
    params: Option<&HashMap<&str, String>>
  ) -> io::Result<TradesResponse> {
    let uri = format!(
      "{}/v2/ticks/stocks/trades/{}/{}?apikey={}{}",
      self.api_uri,
      symbol,
      date.format("%Y-%m-%d"),
      self.key,
      match params {
        Some(p) => make_params(p),
        None => String::new()
      }
    );

    let resp = get_response(&uri)?;
    let mut resp = resp.into_json_deserialize::<TradesResponse>()?;
    resp.uri = Some(uri);

    if resp.results.len() == 0 {
      return Err(Error::new(ErrorKind::UnexpectedEof, "Results is empty"));
    }

    // Polygon returns the exchange opening time in GMT nanoseconds since epoch
    for row in resp.results.iter_mut() {
      // Only US equities are at "stocks" endpoint
      row.ts += 5 * 60 * 60 * 1_000_000_000;
      if NaiveDateTime::from_timestamp(row.ts / 1_000_000_000, 0).date() != date {
        return Err(Error::new(
          ErrorKind::BrokenPipe,
          format!(
            "ts {} is out of range for date {}",
            row.ts,
            date.format("%Y-%m-%d")
          )
        ));
      }
      // Add symbol
      row.symbol = resp.symbol.clone();
    }

    Ok(resp)
  }
}

#[cfg(test)]
mod trades {
  use crate::{client::Client, equities::trades::TradesParams};
  use chrono::NaiveDate;

  #[test]
  fn works() {
    let client = Client::new();
    let date = NaiveDate::from_ymd(2004, 01, 02);
    let limit = 500;
    let params = TradesParams::new().with_limit(limit).params;
    let trades = client.get_trades("AAPL", date, Some(&params)).unwrap();
    assert_eq!(trades.results_count, limit);
    assert_eq!(trades.results.len(), limit);
  }
}
