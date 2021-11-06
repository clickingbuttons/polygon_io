extern crate serde_json;
extern crate ureq;

use super::Candle;
use crate::client::Client;
use serde::{Deserialize, Serialize};
use std::io::{self, Error, ErrorKind};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrevResponse {
  pub query_count: usize,
  pub results_count: usize,
  pub adjusted: bool,
  #[serde(default)] // On 2020-12-07 started being omitted instead of empty
  pub results: Vec<Candle>,
  // For debugging
  pub status: String,
  pub uri: Option<String>
}

impl Client {
  pub fn get_prev(&mut self, symbol: &str) -> io::Result<PrevResponse> {
    let uri = format!(
      "{}/v2/aggs/ticker/{}/prev?apikey={}",
      self.api_uri, symbol, self.key
    );

    let resp = self.get_response(&uri)?;
    let mut resp = resp.into_json::<PrevResponse>()?;
    resp.uri = Some(uri);

    if resp.results.len() != 1 {
      return Err(Error::new(
        ErrorKind::UnexpectedEof,
        format!("Results has length {} (expected 1)", resp.results.len())
      ));
    }

    let is_equity = !symbol.contains(":");
    // Polygon returns GMT milliseconds for this endpoint
    if is_equity {
      // Subtract 5h of milliseconds
      resp.results[0].ts -= 5 * 60 * 60 * 1_000;
    }
    // Convert to ns
    resp.results[0].ts *= 1_000_000;

    Ok(resp)
  }
}

#[cfg(test)]
mod prev {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();
    let prev = client.get_prev("AAPL").unwrap();
    assert_eq!(prev.results.len(), 1);
  }
}
