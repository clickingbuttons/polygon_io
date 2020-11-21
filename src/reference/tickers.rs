extern crate serde_json;
extern crate ureq;

use crate::client::Client;
use crate::helpers::*;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Codes {
  pub cik: Option<String>,
  pub figiuid: Option<String>,
  pub scfigi: Option<String>,
  pub cfigi: Option<String>,
  pub figi: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol: String,
  pub name: String,
  pub market: String,
  pub locale: String,
  pub r#type: Option<String>,
  pub currency: String,
  pub primary_exch: String,
  #[serde(deserialize_with="string_to_naive_date", serialize_with="naive_date_to_string")]
  pub updated: NaiveDate,
  pub codes: Option<Codes>
  // Skip URL because it's broken
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TickersResponse {
  pub page: usize,
  pub per_page: usize,
  pub count: usize,
  pub tickers: Vec<Ticker>,
  // For debugging
  pub status: String,
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
 
  pub fn with_sort(mut self, sort: &str) -> Self {
    self.params.insert("sort", sort.to_string());
    self
  }

  pub fn with_type(mut self, r#type: &str) -> Self {
    self.params.insert("type", r#type.to_string());
    self
  }

  pub fn with_market(mut self, market: &str) -> Self {
    self.params.insert("sort", market.to_string());
    self
  }

  pub fn with_locale(mut self, locale: &str) -> Self {
    self.params.insert("locale", locale.to_string());
    self
  }

  pub fn with_search(mut self, search: &str) -> Self {
    self.params.insert("search", search.to_string());
    self
  }

  pub fn with_perpage(mut self, perpage: usize) -> Self {
    self.params.insert("perpage", perpage.to_string());
    self
  }

  pub fn with_page(mut self, page: usize) -> Self {
    self.params.insert("page", page.to_string());
    self
  }

  pub fn with_active(mut self, active: bool) -> Self {
    self.params.insert("active", active.to_string());
    self
  }
}

impl Client {
  pub fn get_tickers(&self, params: Option<&HashMap<&str, String>>) -> std::io::Result<TickersResponse> {
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
    self.get_tickers(
      Some(&TickersParams::new().with_locale("us").with_market("stocks").with_page(page).params)
    )
  }
}

#[cfg(test)]
mod tickers {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let resp = client
      .get_tickers(None)
      .unwrap();
    assert!(resp.tickers.len() == 50);
  }

  #[test]
  fn us_works() {
    let client = Client::new();
    let resp = client
      .get_us_tickers(1)
      .unwrap();
    assert!(resp.tickers.len() == 50);
    assert_eq!(resp.tickers[0].market, String::from("STOCKS"));
    assert_eq!(resp.tickers[0].locale, String::from("US"));
  }
}

