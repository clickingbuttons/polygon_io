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
  pub primary_exch: Option<String>,
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
  pub primary_exchange: Option<String>,
  pub r#type:       Option<String>,
  pub active:       bool,
  pub currency_name:     Option<String>,
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
  pub next_page_path: Option<String>,
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
  with_param!(market, &str);
  with_param!(exchange, &str);
  with_param!(cusip, &str);
  with_param!(date, &str);
  with_param!(active, bool);
  with_param!(sort, &str);
  with_param!(order, &str);
  with_param!(limit, usize);
  // Undocumented but appears in next_page_path
  with_param!(page_marker, &str);
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

  pub fn get_tickers_v3(
    &self,
    params: Option<&HashMap<&str, String>>
  ) -> std::io::Result<TickersResponseVx> {
    let uri = format!(
      "{}/v3/reference/tickers?apikey={}{}",
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

  pub fn get_all_tickers_vx(
    &self,
    date: &NaiveDate
  ) -> std::io::Result<Vec<TickerVx>> {
    let limit: usize = 500;
    // Use default params since next_page_path does as well
    let mut params = TickersParamsVx::new()
      .market("stocks")
      .limit(limit)
      .order("asc")
      .sort("ticker")
      .date(&date.format("%Y-%m-%d").to_string());
    let mut res = Vec::<TickerVx>::new();
    loop {
      let page = self.get_tickers_v3(Some(&params.params))?;
      res.extend(page.results.into_iter());
      if page.next_page_path.is_none() {
        break;
      }
      let path = page.next_page_path.unwrap();
      let param_name = "page_marker=";
      let marker_start: usize = path.find(param_name).unwrap_or_else(|| {
        panic!("No page marker in {}", path);
      });
      let marker_end = match path[marker_start..].find("&") {
        Some(i) => marker_start + i,
        None => path.len()
      };
      let marker = path[marker_start + param_name.len()..marker_end].to_string();
      params = params.page_marker(&marker);
    }

    Ok(res)
  }
}

#[cfg(test)]
mod tickers {
  use chrono::NaiveDate;
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let resp = client.get_tickers(None).unwrap();
    assert_eq!(resp.tickers.len(), 50);
  }

  #[test]
  fn us_works() {
    let client = Client::new();
    let resp = client.get_us_tickers(1).unwrap();
    assert_eq!(resp.tickers.len(), 50);
    assert_eq!(resp.tickers[0].market, String::from("STOCKS"));
    assert_eq!(resp.tickers[0].locale, String::from("US"));
  }

  #[test]
  fn works_vx() {
    let client = Client::new();
    let resp = client.get_tickers_vx(None).unwrap();
    assert!(resp.results.len() > 0)
  }

  #[test]
  fn works_vx_day() {
    let client = Client::new();
    let results = client.get_all_tickers_vx(&NaiveDate::from_ymd(2004, 01, 02)).unwrap();
    assert_eq!(results.len(), 8163);
  }
}

