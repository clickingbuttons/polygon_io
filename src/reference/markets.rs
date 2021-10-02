extern crate serde_json;
extern crate ureq;

use crate::client::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Market {
  pub market: String,
  pub desc:   String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketsResponse {
  pub results: Vec<Market>,
  // For debugging
  pub status:  String
}

impl Client {
  pub fn get_markets(&self) -> std::io::Result<MarketsResponse> {
    let uri = format!("{}/v2/reference/markets?apikey={}", self.api_uri, self.key);

    let resp = self.get_response(&uri)?;
    let resp = resp.into_json::<MarketsResponse>()?;

    Ok(resp)
  }
}

#[cfg(test)]
mod markets {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let markets = client.get_markets().unwrap();
    assert!(markets
      .results
      .iter()
      .any(|res| res.market == String::from("STOCKS")));
  }
}
