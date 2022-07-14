extern crate serde_json;
extern crate ureq;

use crate::{client::Client, helpers::*, with_param};
use chrono::{DateTime, FixedOffset, NaiveDate};
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
pub struct Ticker {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol: String,
  pub name: String,
  pub market: String,
  pub locale: String,
  pub primary_exchange: Option<String>,
  pub r#type: Option<String>,
  pub active: bool,
  pub currency_name: Option<String>,
  pub cik: Option<String>,
  pub composite_figi: Option<String>,
  pub share_class_figi: Option<String>,
  #[serde(
    deserialize_with = "string_to_datetime",
    serialize_with = "datetime_to_string"
  )]
  pub last_updated_utc: DateTime<FixedOffset>,
  #[serde(
    deserialize_with = "option_string_to_datetime",
    serialize_with = "option_datetime_to_string",
    default
  )]
  pub delisted_utc: Option<DateTime<FixedOffset>>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TickersResponse {
  pub results: Vec<Ticker>,
  pub count: usize,
  pub next_url: Option<String>,
  // For debugging
  pub status: String,
  pub request_id: String
}

pub struct TickersParams<'a> {
  pub params: HashMap<&'a str, String>
}

impl<'a> TickersParams<'a> {
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
  with_param!(cursor, &str);

  pub fn new() -> Self {
    Self {
      params: HashMap::with_capacity(8)
    }
  }
}

impl Client {
  pub fn get_tickers(
    &mut self,
    params: Option<&HashMap<&str, String>>
  ) -> std::io::Result<TickersResponse> {
    let uri = format!(
      "{}/v3/reference/tickers?{}",
      self.api_uri,
      make_params(params),
    );

    let resp = self.get_response::<TickersResponse>(&uri)?;

    Ok(resp)
  }

  pub fn get_all_tickers(&mut self, date: &NaiveDate) -> std::io::Result<Vec<Ticker>> {
    let limit: usize = 500;
    // Use default params since next_page_path does as well
    let mut params = TickersParams::new()
      .market("stocks")
      .limit(limit)
      .order("asc")
      .sort("ticker")
      .date(&date.format("%Y-%m-%d").to_string());
    let mut res = Vec::<Ticker>::new();
    loop {
      let page = self.get_tickers(Some(&params.params))?;
      res.extend(page.results.into_iter());
      if page.next_url.is_none() {
        break;
      }
      let path = page.next_url.unwrap();
      let param_name = "cursor=";
      let marker_start: usize = path.find(param_name).unwrap_or_else(|| {
        panic!("No cursor in {}", path);
      });
      let marker_end = match path[marker_start..].find("&") {
        Some(i) => marker_start + i,
        None => path.len()
      };
      let marker = path[marker_start + param_name.len()..marker_end].to_string();
      params = TickersParams::new().cursor(&marker);
    }

    Ok(res)
  }
}

#[cfg(test)]
mod tickers {
  use crate::client::Client;
  use chrono::NaiveDate;

  #[test]
  fn works() {
    let mut client = Client::new();
    let resp = client.get_tickers(None).unwrap();
    assert_eq!(resp.results.len(), 100);
  }

  #[test]
  fn works_day() {
    let mut client = Client::new();
    let results = client
      .get_all_tickers(&NaiveDate::from_ymd(2004, 01, 02))
      .unwrap();
    assert_eq!(results.len(), 8163);
  }
}
