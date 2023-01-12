extern crate serde_json;
extern crate ureq;

use crate::client::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dividend {
	#[serde(rename(deserialize = "ticker"))]
	pub symbol:       String,
	pub ex_date:      String,
	pub payment_date: String,
	pub record_date:  String,
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
	pub fn get_dividends(&mut self, symbol: &str) -> std::io::Result<DividendsResponse> {
		let uri = format!("{}/v2/reference/dividends/{}", self.api_uri, symbol);

		let resp = self.get_response::<DividendsResponse>(&uri)?;

		Ok(resp)
	}
}

#[cfg(test)]
mod dividends {
	use crate::client::Client;

	#[test]
	fn works() {
		let mut client = Client::new().unwrap();
		let splits = client.get_dividends("AAPL").unwrap();
		assert!(splits.results.len() > 60);
	}
}
