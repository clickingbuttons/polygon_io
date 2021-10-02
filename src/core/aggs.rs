extern crate serde_json;
extern crate ureq;

use super::Candle;
use crate::{
  client::Client,
  helpers::make_params,
  with_param
};
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct AggResponse {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol: String,
  #[serde(rename(deserialize = "queryCount"))]
  pub query_count: usize,
  #[serde(rename(deserialize = "resultsCount"))]
  pub results_count: usize,
  pub adjusted: bool,
  #[serde(default)] // On 2020-12-07 started being omitted instead of empty
  pub results: Vec<Candle>,
  // For debugging
  pub request_id: String,
  pub uri: Option<String>
}

pub struct AggsParams<'a> {
  pub params: HashMap<&'a str, String>
}

impl<'a> AggsParams<'a> {
  pub fn new() -> Self {
    Self {
      params: HashMap::with_capacity(3)
    }
  }

  with_param!(unadjusted, bool);
  with_param!(sort, &str);
  with_param!(limit, i32);
}

impl Client {
  pub fn get_aggs(
    &self,
    symbol: &str,
    multiplier: i64,
    timespan: Timespan,
    from: NaiveDate,
    to: NaiveDate,
    params: Option<&HashMap<&str, String>>
  ) -> io::Result<AggResponse> {
    let uri = format!(
      "{}/v2/aggs/ticker/{}/range/{}/{}/{}/{}?apikey={}{}",
      self.api_uri,
      symbol,
      multiplier,
      format!("{:?}", timespan).to_lowercase(),
      from.format("%Y-%m-%d"),
      to.format("%Y-%m-%d"),
      self.key,
      match params {
        Some(p) => make_params(p),
        None => String::new()
      }
    );
    let resp = self.get_response(&uri)?;
    let mut resp = resp.into_json::<AggResponse>()?;
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
  use crate::{client::Client, core::aggs::AggsParams};
  use chrono::NaiveDate;
  use std::io::ErrorKind;

  #[test]
  fn aapl() {
    let client = Client::new();
    let from = NaiveDate::from_ymd(2020, 11, 5);
    let to = NaiveDate::from_ymd(2020, 11, 5);
    let sym = String::from("AAPL");
    let resp = client
      .get_aggs(&sym, 1, Timespan::Minute, from, to, None)
      .unwrap();
    assert_eq!(resp.results.len(), 941);
    assert_eq!(resp.results.len(), resp.results_count);
    assert_eq!(resp.results.len(), resp.query_count);
    assert_eq!(resp.results[0].symbol, sym);
  }

  #[test]
  fn mac() {
    let client = Client::new();
    let from = NaiveDate::from_ymd(2004, 1, 1);
    let to = NaiveDate::from_ymd(2020, 2, 1);
    let sym = String::from("MAC");
    let params = AggsParams::new().unadjusted(true).params;
    client.get_aggs(&sym, 1, Timespan::Minute, from, to, Some(&params)).unwrap();
  }

  #[test]
  fn no_range_errors() {
    let client = Client::new();
    let from = NaiveDate::from_ymd(2008, 11, 1);
    let to = NaiveDate::from_ymd(2008, 12, 1);
    let params = AggsParams::new().limit(50_000).params;
    for _ in 0..10 {
      match client.get_aggs("AAPL", 1, Timespan::Minute, from, to, Some(&params)) {
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

  #[test]
  fn empty_results() {
    let client = Client::new();
    let from = NaiveDate::from_ymd(2004, 3, 1);
    let to = NaiveDate::from_ymd(2004, 3, 31);
    let sym = String::from("CINpJ");
    let params = AggsParams::new().params;
    match client.get_aggs(&sym, 1, Timespan::Minute, from, to, Some(&params)) {
      Ok(_) => panic!("CINpJ should not have agg1m in 2004-03"),
      Err(e) => assert_eq!(e.kind(), ErrorKind::UnexpectedEof)
    }
  }
}
