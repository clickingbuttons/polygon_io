extern crate serde_json;
extern crate ureq;

use crate::{client::Client, helpers::*};
use crate::with_param;
use chrono::{NaiveDate, DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Codes {
  pub cik:     Option<String>,
  pub figiuid: Option<String>,
  pub scfigi:  Option<String>,
  pub cfigi:   Option<String>,
  pub figi:    Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol:       String,
  pub name:         String,
  pub market:       String,
  pub locale:       String,
  pub r#type:       Option<String>,
  pub currency:     String,
  pub primary_exch: String,
  #[serde(
    deserialize_with = "string_to_naive_date",
    serialize_with = "naive_date_to_string"
  )]
  pub updated:      NaiveDate,
  pub codes:        Option<Codes>
  // Skip URL because it's broken
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickerVx {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol:       String,
  pub name:         String,
  pub market:       String,
  pub locale:       String,
  pub primary_exchange: String,
  pub r#type:       Option<String>,
  pub active:       bool,
  pub currency_name:     String,
  pub cik:     Option<String>,
  pub composite_figi:     Option<String>,
  pub share_class_figi:     Option<String>,
  #[serde(
    deserialize_with = "string_to_datetime",
    serialize_with = "datetime_to_string"
  )]
  pub last_updated_utc:      DateTime<FixedOffset>,
  #[serde(
    deserialize_with = "option_string_to_datetime",
    serialize_with = "option_datetime_to_string",
    default
  )]
  pub delisted_utc:      Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TickersResponse {
  pub page:     usize,
  pub per_page: usize,
  pub count:    usize,
  pub tickers:  Vec<Ticker>,
  // For debugging
  pub status:   String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickersResponseVx {
  pub results:  Vec<TickerVx>,
  pub count: usize,
  pub next_page_path: String,
  // For debugging
  pub status:   String,
  pub request_id: String
}

pub struct TickersParams<'a> {
  pub params: HashMap<&'a str, String>
}

impl<'a> TickersParams<'a> {
  pub fn new() -> Self {
    Self {
      params: HashMap::with_capacity(8)
    }
  }

  with_param!(sort, &str);
  with_param!(r#type, &str);
  with_param!(market, &str);
  with_param!(locale, &str);
  with_param!(search, &str);
  with_param!(perpage, usize);
  with_param!(page, usize);
  with_param!(active, bool);
}

pub struct TickersParamsVx<'a> {
  pub params: HashMap<&'a str, String>
}

impl<'a> TickersParamsVx<'a> {
  pub fn new() -> Self {
    Self {
      params: HashMap::with_capacity(8)
    }
  }

  with_param!(ticker, &str);
  with_param!(r#type, &str);
  with_param!(exchange, &str);
  with_param!(cusip, &str);
  with_param!(date, &str);
  with_param!(active, bool);
  with_param!(sort, &str);
  with_param!(order, &str);
  with_param!(limit, usize);
}

impl Client {
  pub fn get_tickers(
    &self,
    params: Option<&HashMap<&str, String>>
  ) -> std::io::Result<TickersResponse> {
    let uri = format!(
      "{}/v2/reference/tickers?apikey={}{}",
      self.api_uri,
      self.key,
      match params {
        Some(p) => make_params(p),
        None => String::new()
      }
    );

    let resp = get_response(&uri)?;
    let resp = resp.into_json_deserialize::<TickersResponse>()?;

    Ok(resp)
  }

  pub fn get_us_tickers(&self, page: usize) -> std::io::Result<TickersResponse> {
    self.get_tickers(Some(
      &TickersParams::new()
        .locale("us")
        .market("stocks")
        .page(page)
        .params
    ))
  }

  pub fn get_tickers_vx(
    &self,
    params: Option<&HashMap<&str, String>>
  ) -> std::io::Result<TickersResponseVx> {
    let uri = format!(
      "{}/vX/reference/tickers?apikey={}{}",
      self.api_uri,
      self.key,
      match params {
        Some(p) => make_params(p),
        None => String::new()
      }
    );

    let resp = get_response(&uri)?;
    let resp = resp.into_json_deserialize::<TickersResponseVx>()?;

    Ok(resp)
  }
}

#[cfg(test)]
mod tickers {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let resp = client.get_tickers(None).unwrap();
    assert!(resp.tickers.len() == 50);
  }

  #[test]
  fn us_works() {
    let client = Client::new();
    let resp = client.get_us_tickers(1).unwrap();
    assert!(resp.tickers.len() == 50);
    assert_eq!(resp.tickers[0].market, String::from("STOCKS"));
    assert_eq!(resp.tickers[0].locale, String::from("US"));
  }

  #[test]
  fn works_vx() {
    let client = Client::new();
    let resp = client.get_tickers_vx(None).unwrap();
    assert!(resp.results.len() == 100);
  }
}
