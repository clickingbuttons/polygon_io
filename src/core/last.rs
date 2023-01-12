extern crate serde_json;
extern crate ureq;

use super::Candle;
use crate::client::{Client, Result};
use serde::{Deserialize, Serialize};

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
	pub fn get_prev(&self, symbol: &str) -> Result<PrevResponse> {
		let uri = format!("{}/v2/aggs/ticker/{}/prev", self.api_uri, symbol);

		let mut resp = self.get_response::<PrevResponse>(&uri)?;
		resp.uri = Some(uri);

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
		let client = Client::new().unwrap();
		let prev = client.get_prev("AAPL").unwrap();
		assert_eq!(prev.results.len(), 1);
	}
}
