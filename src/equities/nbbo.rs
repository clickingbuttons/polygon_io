extern crate serde_json;
extern crate ureq;

use crate::helpers::{get_response,make_param};
use crate::client::Client;
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::io::{self, Error, ErrorKind};

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

impl Client {
  pub fn get_nbbo(
    &self,
    symbol: &str,
    date: NaiveDate,
    timestamp: Option<i64>,
    timestamp_limit: Option<i64>,
    reverse: Option<bool>,
    limit: Option<usize>
  ) -> io::Result<NBBOsResponse> {
    let uri = format!(
      "{}/v2/ticks/stocks/nbbo/{}/{}?apikey={}{}{}{}{}",
      self.api_uri,
      symbol,
      date.format("%Y-%m-%d"),
      self.key,
      make_param("timestamp", timestamp),
      make_param("timestamp_limit", timestamp_limit),
      make_param("reverse", reverse),
      make_param("limit", limit)
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
  use chrono::NaiveDate;

  #[test]
  fn works() {
    let client = Client::new();
    let date = NaiveDate::from_ymd(2005, 01, 03);
    let limit = 500;
    let nbbo = client
      .get_nbbo("AAPL", date, None, None, None, Some(limit))
      .unwrap();
    assert_eq!(nbbo.results_count, limit);
    assert_eq!(nbbo.results.len(), limit);
  }
}
