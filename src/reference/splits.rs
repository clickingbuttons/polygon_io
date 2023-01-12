extern crate serde_json;
extern crate ureq;

use crate::client::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Split {
	#[serde(rename(deserialize = "ticker"))]
	pub symbol:        String,
	pub ex_date:       String,
	pub payment_date:  String,
	pub declared_date: Option<String>,
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
	pub fn get_splits(&mut self, symbol: &str) -> std::io::Result<SplitsResponse> {
		let uri = format!("{}/v2/reference/splits/{}", self.api_uri, symbol);

		let resp = self.get_response::<SplitsResponse>(&uri)?;

		Ok(resp)
	}
}

#[cfg(test)]
mod splits {
	use crate::client::Client;

	#[test]
	fn works() {
		let mut client = Client::new().unwrap();
		let splits = client.get_splits("AAPL").unwrap();
		assert!(splits.results.len() > 3);
	}
}
