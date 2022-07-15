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
  pub fn get_market_status_upcoming(&mut self) -> io::Result<Vec<MarketHolidayResponse>> {
    let uri = format!("{}/v1/marketstatus/upcoming", self.api_uri);
    let resp = self.get_response::<Vec<MarketHolidayResponse>>(&uri)?;

    Ok(resp)
  }
}

#[cfg(test)]
mod market_status_upcoming {
  use crate::client::Client;

  #[test]
  fn works() {
    let mut client = Client::new();

    let resp = client.get_market_status_upcoming();
    assert!(resp.is_ok());
  }
}
