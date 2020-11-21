extern crate serde_json;
extern crate ureq;

use crate::{client::Client, helpers::*};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dividend {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol:       String,
  #[serde(
    deserialize_with = "string_to_naive_date",
    serialize_with = "naive_date_to_string"
  )]
  pub ex_date:      NaiveDate,
  #[serde(
    deserialize_with = "string_to_naive_date",
    serialize_with = "naive_date_to_string"
  )]
  pub payment_date: NaiveDate,
  #[serde(
    deserialize_with = "string_to_naive_date",
    serialize_with = "naive_date_to_string"
  )]
  pub record_date:  NaiveDate,
  pub amount:       f32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DividendsResponse {
  pub count:   usize,
  pub results: Vec<Dividend>,
  // For debugging
  pub status:  String
}

impl Client {
  pub fn get_dividends(&self, symbol: &str) -> std::io::Result<DividendsResponse> {
    let uri = format!(
      "{}/v2/reference/dividends/{}?apikey={}",
      self.api_uri, symbol, self.key
    );

    let resp = get_response(&uri)?;
    let resp = resp.into_json_deserialize::<DividendsResponse>()?;

    Ok(resp)
  }
}

#[cfg(test)]
mod dividends {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let splits = client.get_dividends("AAPL").unwrap();
    assert!(splits.results.len() > 60);
  }
}
