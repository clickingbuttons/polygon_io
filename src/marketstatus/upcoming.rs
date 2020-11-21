use crate::{client::Client, helpers::*};
use chrono::{DateTime, FixedOffset, NaiveDate};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketHolidayResponse {
  pub exchange: String,
  pub name:     String,
  #[serde(
    deserialize_with = "string_to_naive_date",
    serialize_with = "naive_date_to_string"
  )]
  pub date:     NaiveDate,
  pub status:   String,
  #[serde(
    serialize_with = "option_datetime_to_string",
    deserialize_with = "option_string_to_datetime",
    default
  )]
  pub open:     Option<DateTime<FixedOffset>>,
  #[serde(
    serialize_with = "option_datetime_to_string",
    deserialize_with = "option_string_to_datetime",
    default
  )]
  pub close:    Option<DateTime<FixedOffset>>
}

impl Client {
  pub fn get_market_status_upcoming(&self) -> io::Result<Vec<MarketHolidayResponse>> {
    let uri = format!(
      "{}/v1/marketstatus/upcoming?apikey={}",
      self.api_uri, self.key
    );
    let resp = get_response(&uri)?;
    let resp = resp.into_json_deserialize::<Vec<MarketHolidayResponse>>()?;

    Ok(resp)
  }
}

#[cfg(test)]
mod market_status_upcoming {
  use crate::client::Client;

  #[test]
  fn works() {
    let client = Client::new();

    let resp = client.get_market_status_upcoming();
    assert!(resp.is_ok());
  }
}
