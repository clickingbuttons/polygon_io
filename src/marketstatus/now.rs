use crate::{client::Client, helpers::*};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Deserialize, Serialize)]
pub struct ExchangeStatusResponse {
  pub nyse:   String,
  pub nasdaq: String,
  pub otc:    String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CurrencyStatusResponse {
  pub fx:     String,
  pub crypto: String
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketStatusNowResponse {
  pub market:      String,
  #[serde(
    serialize_with = "datetime_to_string",
    deserialize_with = "string_to_datetime"
  )]
  pub server_time: DateTime<FixedOffset>,
  pub exchanges:   ExchangeStatusResponse,
  pub currencies:  CurrencyStatusResponse
}

impl Client {
  pub fn get_market_status_now(&mut self) -> io::Result<MarketStatusNowResponse> {
    let uri = format!("{}/v1/marketstatus/now?apikey={}", self.api_uri, self.key);
    let resp = self.get_response::<MarketStatusNowResponse>(&uri)?;

    Ok(resp)
  }
}

#[cfg(test)]
mod market_status_now {
  use crate::client::Client;

  #[test]
  fn works() {
    let mut client = Client::new();

    let resp = client.get_market_status_now();
    assert!(resp.is_ok());
  }
}
