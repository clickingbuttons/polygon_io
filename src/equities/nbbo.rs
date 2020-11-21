extern crate serde_json;
extern crate ureq;

use crate::helpers::{get_response,make_params};
use crate::client::Client;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::io::{self, Error, ErrorKind};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct NBBO {
  #[serde(rename(deserialize = "t"))]
  pub ts:             i64,
  #[serde(rename(deserialize = "y"))]
  pub ts_participant: Option<i64>,
  #[serde(rename(deserialize = "f"))]
  pub ts_trf:         Option<i64>,
  #[serde(default)]
  pub symbol:         String,
  #[serde(rename(deserialize = "x"))]
  pub bid_exchange:       u32,
  #[serde(rename(deserialize = "X"))]
  pub ask_exchange:       u32,
  #[serde(rename(deserialize = "s"))]
  pub bid_lots:           u32,
  #[serde(rename(deserialize = "S"))]
  pub ask_lots:           u32,
  //#[serde(rename(deserialize = "c"))]
  // pub conditions:     i32,
  #[serde(rename(deserialize = "p"))]
  pub bid_price:          f32,
  #[serde(rename(deserialize = "P"))]
  pub ask_price:          f32,
  #[serde(rename(deserialize = "z"))]
  pub tape:           u32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NBBOsResponse {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol: String,
  pub results_count: usize,
  pub results: Vec<NBBO>,
  pub db_latency: usize,
  // For debugging
  pub success: bool,
  pub uri: Option<String>
}

pub struct NBBOsParams<'a> {
  pub params: HashMap<&'a str, String>
}

impl<'a> NBBOsParams<'a> {
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
    self.params.insert("timestamp_limit", timestamp_limit.to_string());
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
  pub fn get_nbbo(
    &self,
    symbol: &str,
    date: NaiveDate,
    params: Option<&HashMap<&str, String>>
  ) -> io::Result<NBBOsResponse> {
    let uri = format!(
      "{}/v2/ticks/stocks/nbbo/{}/{}?apikey={}{}",
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
    let mut resp = resp.into_json_deserialize::<NBBOsResponse>()?;
    resp.uri = Some(uri);

    if resp.results.len() == 0 {
      return Err(Error::new(ErrorKind::UnexpectedEof, "Results is empty"));
    }

    // Polygon returns the exchange opening time in nanoseconds since epoch
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
mod nbbo {
  use crate::client::Client;
  use crate::equities::nbbo::NBBOsParams;
  use chrono::NaiveDate;

  #[test]
  fn works() {
    let client = Client::new();
    let date = NaiveDate::from_ymd(2005, 01, 03);
    let limit = 500;
    let params = NBBOsParams::new().with_limit(limit).params;
    let nbbo = client
      .get_nbbo("AAPL", date, Some(&params))
      .unwrap();
    assert_eq!(nbbo.results_count, limit);
    assert_eq!(nbbo.results.len(), limit);
  }
}

