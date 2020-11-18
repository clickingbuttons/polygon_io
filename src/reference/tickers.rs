extern crate serde_json;
extern crate ureq;

use crate::client::Client;
use crate::helpers::get_response;
use crate::helpers::make_param;
use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use super::*;

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

impl Client {
  pub fn get_tickers(
    &self,
    sort: Option<&str>,
    r#type: Option<&str>,
    market: Option<&str>,
    locale: Option<&str>,
    search: Option<&str>,
    perpage: Option<usize>,
    page: Option<usize>,
    active: Option<bool>
  ) -> std::io::Result<TickersResponse> {
    let uri = format!(
      "{}/v2/reference/tickers?apikey={}{}{}{}{}{}{}{}{}",
      self.api_uri,
      self.key,
      make_param("sort", sort),
      make_param("type", r#type),
      make_param("market", market),
      make_param("locale", locale),
      make_param("search", search),
      make_param("perpage", perpage),
      make_param("page", page),
      make_param("active", active)
    );

    let resp = get_response(&uri)?;
    let resp = resp.into_json_deserialize::<TickersResponse>()?;

    Ok(resp)
  }

  pub fn get_us_tickers(&self, page: usize) -> std::io::Result<TickersResponse> {
    self.get_tickers(None, None, Some("stocks"), Some("us"), None, None, Some(page), None)
  }
}

#[cfg(test)]
mod tickers {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let resp = client
      .get_tickers(None, None, None, None, None, None, None, None)
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

