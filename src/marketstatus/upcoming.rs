use crate::client::{Client, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct MarketHolidayResponse {
	pub exchange: String,
	pub name:     String,
	pub date:     String,
	pub status:   String,
	pub open:     Option<String>,
	pub close:    Option<String>
}

impl Client {
	pub fn get_market_status_upcoming(&mut self) -> Result<Vec<MarketHolidayResponse>> {
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
		let mut client = Client::new().unwrap();

		let resp = client.get_market_status_upcoming();
		assert!(resp.is_ok());
	}
}
