extern crate serde_json;
extern crate ureq;

use crate::{client::Client, helpers::*};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Split {
  #[serde(rename(deserialize = "ticker"))]
  pub symbol:        String,
  #[serde(
    deserialize_with = "string_to_naive_date",
    serialize_with = "naive_date_to_string"
  )]
  pub ex_date:       NaiveDate,
  #[serde(
    deserialize_with = "string_to_naive_date",
    serialize_with = "naive_date_to_string"
  )]
  pub payment_date:  NaiveDate,
  #[serde(
    deserialize_with = "option_string_to_naive_date",
    serialize_with = "option_naive_date_to_string",
    default
  )]
  pub declared_date: Option<NaiveDate>,
  pub ratio:         f64,
  #[serde(rename(deserialize = "forfactor"))]
  pub for_factor:    Option<i32>,
  #[serde(rename(deserialize = "tofactor"))]
  pub to_factor:     Option<i32>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SplitsResponse {
  pub count:   usize,
  pub results: Vec<Split>,
  // For debugging
  pub status:  String
}

impl Client {
  pub fn get_splits(&self, symbol: &str) -> std::io::Result<SplitsResponse> {
    let uri = format!(
      "{}/v2/reference/splits/{}?apikey={}",
      self.api_uri, symbol, self.key
    );

    let resp = get_response(&uri)?;
    let resp = resp.into_json_deserialize::<SplitsResponse>()?;

    Ok(resp)
  }
}

#[cfg(test)]
mod splits {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let splits = client.get_splits("AAPL").unwrap();
    assert!(splits.results.len() > 3);
  }
}
