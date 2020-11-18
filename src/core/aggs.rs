extern crate serde_json;
extern crate ureq;

use super::Candle;
use crate::helpers::{get_response,make_param};
use crate::client::Client;
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::{
  fs::create_dir_all,
  io::{self, Error, ErrorKind}
};

static LOG_DIR_AGGS: &str = "logs/aggs";

#[derive(Debug)]
pub enum Timespan {
  Minute,
  Hour,
  Day,
  Week,
  Month,
  Quarter,
  Year
}

#[derive(Debug)]
pub enum Sort {
  Asc,
  Desc
}

impl ToString for Sort {
  fn to_string(&self) -> String {
    String::from(match self {
      Sort::Asc => "asc",
      Sort::Desc => "desc"
    })
  }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AggResponse {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol: String,
  #[serde(rename(deserialize = "queryCount"))]
  pub query_count: usize,
  #[serde(rename(deserialize = "resultsCount"))]
  pub results_count: usize,
  pub adjusted: bool,
  pub results: Vec<Candle>,
  // For debugging
  pub request_id: String,
  pub uri: Option<String>
}

impl Client {
  pub fn get_aggs(
    &self,
    symbol: &str,
    multiplier: i64,
    timespan: Timespan,
    from: NaiveDate,
    to: NaiveDate,
    adjusted: Option<bool>,
    sort: Option<Sort>,
    limit: Option<i32>
  ) -> io::Result<AggResponse> {
    let uri = format!(
      "{}/v2/aggs/ticker/{}/range/{}/{:?}/{}/{}?apikey={}{}{}{}",
      self.api_uri,
      symbol,
      multiplier,
      timespan,
      from.format("%Y-%m-%d"),
      to.format("%Y-%m-%d"),
      self.key,
      make_param("adjusted", adjusted),
      make_param("sort", sort),
      make_param("limit", limit),
    );
    let resp = get_response(&uri)?;
    let mut resp = resp.into_json_deserialize::<AggResponse>()?;
    resp.uri = Some(uri);

    if resp.results.len() == 0 {
      return Err(Error::new(ErrorKind::UnexpectedEof, "Results is empty"));
    }

    let is_equity = !symbol.contains(":");
    let mut min_ts = i64::MAX;
    let mut max_ts = i64::MIN;
    for candle in resp.results.iter_mut() {
      // Polygon returns GMT milliseconds for this endpoint
      if is_equity {
        // Subtract 5h of milliseconds
        candle.ts -= 5 * 60 * 60 * 1_000;
      }
      // Convert to ns
      candle.ts *= 1_000_000;
      if candle.ts > max_ts {
        max_ts = candle.ts;
      }
      if candle.ts < min_ts {
        min_ts = candle.ts;
      }
      // Add symbol
      candle.symbol = resp.symbol.clone();
    }

    let to = to.clone() + Duration::days(1);
    let too_small = min_ts < from.and_hms(0, 0, 0).timestamp_nanos();
    let too_big = max_ts > to.and_hms(0, 0, 0).timestamp_nanos();
    if too_small || too_big {
      // Write error to file
      create_dir_all(LOG_DIR_AGGS).unwrap();
      let bad_json = serde_json::to_string(&resp).unwrap();
      let filename = format!("{}/{}.json", LOG_DIR_AGGS, Utc::now().to_rfc3339());
      std::fs::write(filename, bad_json).expect("Could not write log file");

      if too_small {
        return Err(Error::new(
          ErrorKind::BrokenPipe,
          format!("ts {} is too small", min_ts)
        ));
      } else {
        return Err(Error::new(
          ErrorKind::BrokenPipe,
          format!("ts {} is too big", max_ts)
        ));
      }
    }

    Ok(resp)
  }
}

#[cfg(test)]
mod aggs {
  use super::Timespan;
  use crate::client::Client;
  use chrono::NaiveDate;
  use std::io::ErrorKind;

  #[test]
  fn aapl() {
    let client = Client::new();
    let from = NaiveDate::from_ymd(2020, 11, 5);
    let to = NaiveDate::from_ymd(2020, 11, 5);
    let sym = String::from("AAPL");
    let resp = client
      .get_aggs(&sym, 1, Timespan::Minute, from, to, None, None, None)
      .unwrap();
    assert_eq!(resp.results.len(), 941);
    assert_eq!(resp.results.len(), resp.results_count);
    assert_eq!(resp.results.len(), resp.query_count);
    assert_eq!(resp.results[0].symbol, sym);
  }

  #[test]
  fn no_range_errors() {
    let client = Client::new();
    let from = NaiveDate::from_ymd(2008, 11, 1);
    let to = NaiveDate::from_ymd(2008, 12, 1);
    for _ in 0..100 {
      match client.get_aggs(
        "AAPL",
        1,
        Timespan::Minute,
        from,
        to,
        None,
        None,
        Some(50_000)
      ) {
        Ok(_v) => {}
        Err(e) => match e.kind() {
          ErrorKind::BrokenPipe => {
            panic!("Range error {}", e.to_string());
          }
          _ => {}
        }
      };
    }
  }
}
