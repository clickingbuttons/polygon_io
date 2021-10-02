extern crate serde_json;
extern crate ureq;

use super::Candle;
use crate::{
  client::Client,
  helpers::make_params,
  with_param
};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::{
  collections::HashMap,
  io::{self, Error, ErrorKind}
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedResponse {
  pub query_count: usize,
  pub results_count: usize,
  pub adjusted: bool,
  #[serde(default)] // On 2020-12-07 started being omitted instead of empty
  pub results: Vec<Candle>,
  // For debugging
  pub status: String,
  pub uri: Option<String>
}

#[derive(Debug)]
pub enum Locale {
  Global,
  US,
  GB,
  CA,
  NL,
  GR,
  SP,
  DE,
  BE,
  DK,
  FI,
  IE,
  PT,
  IN,
  MX,
  FR,
  CN,
  CH,
  SE
}

#[derive(Debug)]
pub enum Market {
  Stocks,
  Crypto,
  Bonds,
  MF,
  MMF,
  Indices,
  FX
}

pub struct GroupedParams<'a> {
  pub params: HashMap<&'a str, String>
}

impl<'a> GroupedParams<'a> {
  pub fn new() -> Self {
    Self {
      params: HashMap::with_capacity(1)
    }
  }

  with_param!(unadjusted, bool);
}

impl Client {
  pub fn get_grouped(
    &self,
    locale: Locale,
    market: Market,
    date: NaiveDate,
    params: Option<&HashMap<&str, String>>
  ) -> io::Result<GroupedResponse> {
    let uri = format!(
      "{}/v2/aggs/grouped/locale/{}/market/{}/{}?apikey={}{}",
      self.api_uri,
      format!("{:?}", locale).to_lowercase(),
      format!("{:?}", market).to_lowercase(),
      date.format("%Y-%m-%d"),
      self.key,
      match params {
        Some(p) => make_params(p),
        None => String::new()
      }
    );

    let resp = self.get_response(&uri)?;
    let mut resp = resp.into_json::<GroupedResponse>()?;
    resp.uri = Some(uri);

    if resp.results.len() == 0 {
      return Err(Error::new(ErrorKind::UnexpectedEof, "Results is empty"));
    }

    // Polygon returns the exchange opening time in milliseconds since epoch
    let ts = date.and_hms(0, 0, 0).timestamp_nanos();
    for row in resp.results.iter_mut() {
      if NaiveDateTime::from_timestamp(row.ts / 1000, 0).date() != date {
        return Err(Error::new(
          ErrorKind::BrokenPipe,
          format!(
            "ts {} is out of range for date {}",
            ts,
            date.format("%Y-%m-%d")
          )
        ));
      }
      row.ts = ts;
    }

    Ok(resp)
  }
}

#[cfg(test)]
mod grouped {
  use super::{Locale, Market, GroupedParams};
  use crate::client::Client;
  use chrono::NaiveDate;
  use std::io::ErrorKind;

  #[test]
  fn start() {
    let client = Client::new();
    let date = NaiveDate::from_ymd(2004, 01, 02);
    let params = GroupedParams::new().unadjusted(true).params;
    let grouped = client
      .get_grouped(Locale::US, Market::Stocks, date, Some(&params))
      .unwrap();
    assert_eq!(grouped.query_count, grouped.results_count);
    assert_eq!(grouped.query_count, 7670);
    assert_eq!(grouped.results.len(), 7670);
  }

  #[test]
  fn no_bad_ranges() {
    let client = Client::new();
    let date = NaiveDate::from_ymd(2004, 01, 02);
    let params = GroupedParams::new().unadjusted(true).params;
    for _ in 0..50 {
      client
        .get_grouped(Locale::US, Market::Stocks, date, Some(&params))
        .unwrap();
    }
  }

  #[test]
  fn no_garbage_tickers() {
    let client = Client::new();
    let bad_dates = vec![
      NaiveDate::from_ymd(2020, 04, 07),
      NaiveDate::from_ymd(2020, 04, 08),
      NaiveDate::from_ymd(2020, 04, 09),
      NaiveDate::from_ymd(2020, 04, 13),
      NaiveDate::from_ymd(2020, 04, 14),
    ];
    let params = GroupedParams::new().unadjusted(true).params;
    for date in bad_dates {
      let grouped = client
        .get_grouped(Locale::US, Market::Stocks, date, Some(&params))
        .unwrap();
      let mut has_bad_ticker = false;
      for candle in grouped.results {
        if !candle.symbol.chars().all(|c| c.is_ascii_graphic()) {
          eprintln!("Bad ticker {}", candle.symbol);
          has_bad_ticker = true;
        }
      }
      assert!(!has_bad_ticker);
    }
  }

  #[test]
  fn no_missing_vw() {
    let client = Client::new();
    let bad_dates = vec![
      NaiveDate::from_ymd(2020, 04, 07),
      NaiveDate::from_ymd(2020, 04, 08),
      NaiveDate::from_ymd(2020, 04, 09),
      NaiveDate::from_ymd(2020, 04, 13),
      NaiveDate::from_ymd(2020, 04, 14),
    ];
    let params = GroupedParams::new().unadjusted(true).params;
    for date in bad_dates {
      let grouped = client
        .get_grouped(Locale::US, Market::Stocks, date, Some(&params))
        .unwrap();
      let mut missing_vwap = false;
      for candle in grouped.results {
        if candle.volume > 0 && candle.vwap.is_nan() {
          eprintln!("Bad ticker {}", candle.symbol);
          missing_vwap = true;
        }
      }
      assert!(!missing_vwap);
    }
  }

  #[test]
  fn empty_results() {
    let client = Client::new();
    let date = NaiveDate::from_ymd(2004, 1, 1);
    let params = GroupedParams::new().unadjusted(true).params;
    match client.get_grouped(Locale::US, Market::Stocks, date, Some(&params)) {
      Ok(_) => panic!("CINpJ should not have agg1m in 2004-03"),
      Err(e) => assert_eq!(e.kind(), ErrorKind::UnexpectedEof)
    }
  }
}
