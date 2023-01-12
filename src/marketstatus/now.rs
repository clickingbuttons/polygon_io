use crate::client::{Client, Result};
use serde::{Deserialize, Serialize};

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
	pub server_time: String,
	pub exchanges:   ExchangeStatusResponse,
	pub currencies:  CurrencyStatusResponse
}

impl Client {
	pub fn get_market_status_now(&self) -> Result<MarketStatusNowResponse> {
		let uri = format!("{}/v1/marketstatus/now", self.api_uri);
		let resp = self.get_response::<MarketStatusNowResponse>(&uri)?;

		Ok(resp)
	}
}

#[cfg(test)]
mod market_status_now {
	use crate::client::Client;

	#[test]
	fn works() {
		let client = Client::new().unwrap();

		let resp = client.get_market_status_now();
		assert!(resp.is_ok());
	}
}
