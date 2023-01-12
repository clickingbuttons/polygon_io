extern crate serde_json;
extern crate ureq;

use crate::{
	client::{Client, Result},
	helpers::make_params,
	with_param
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct NBBO {
	#[serde(rename(deserialize = "sip_timestamp"))]
	pub ts: i64,
	#[serde(rename(deserialize = "participant_timestamp"))]
	pub ts_participant: Option<i64>,
	#[serde(rename(deserialize = "trf_timestamp"))]
	pub ts_trf: Option<i64>,
	#[serde(default)]
	pub symbol: String,
	pub bid_exchange: u32,
	pub ask_exchange: u32,
	#[serde(rename(deserialize = "bid_size"))]
	pub bid_lots: u32,
	#[serde(rename(deserialize = "ask_size"))]
	pub ask_lots: u32,
	//#[serde(deserialize_with = "to_conditions", default)]
	// pub conditions: u32,
	pub bid_price: f32,
	pub ask_price: f32,
	pub tape: u32
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NBBOsResponse {
	pub results: Vec<NBBO>,
	// For debugging
	pub uri:     Option<String>
}

pub struct NBBOsParams<'a> {
	pub params: HashMap<&'a str, String>
}

impl<'a> NBBOsParams<'a> {
	with_param!(timestamp, &str);

	with_param!(timestamp_lt, &str);

	with_param!(timestamp_lte, &str);

	with_param!(timestamp_gt, &str);

	with_param!(timestamp_gte, &str);

	with_param!(order, &str);

	with_param!(reverse, bool);

	with_param!(limit, usize);

	// Undocumented but appears in next_page_path
	with_param!(cursor, &str);

	pub fn new() -> Self {
		Self {
			params: HashMap::with_capacity(4)
		}
	}
}

impl Client {
	pub fn get_nbbo(
		&mut self,
		symbol: &str,
		params: Option<&HashMap<&str, String>>
	) -> Result<NBBOsResponse> {
		let uri = format!(
			"{}/v3/quotes/{}{}",
			self.api_uri,
			symbol,
			make_params(params),
		);

		let mut resp = self.get_response::<NBBOsResponse>(&uri)?;
		resp.uri = Some(uri);

		// Polygon returns the exchange opening time in nanoseconds since epoch
		for row in resp.results.iter_mut() {
			row.symbol = symbol.to_string();
		}

		Ok(resp)
	}
}

#[cfg(test)]
mod nbbo {
	use crate::{client::Client, equities::nbbo::NBBOsParams};

	#[test]
	fn works() {
		let mut client = Client::new().unwrap();
		let limit = 500;
		let params = NBBOsParams::new()
			.timestamp("2005-01-03")
			.limit(limit)
			.params;
		let nbbo = client.get_nbbo("AAPL", Some(&params)).unwrap();
		assert_eq!(nbbo.results.len(), limit);
	}
}
