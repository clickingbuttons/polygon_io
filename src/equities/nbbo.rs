extern crate serde_json;
extern crate ureq;
use crate::client::Error;

use crate::{
	client::{Client, Result},
	helpers::make_params,
	with_param
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

const MAX_LIMIT: usize = 50_000;

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
	pub next_url: Option<String>,
	pub status:   String, // For debugging
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
		&self,
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

	pub fn get_all_nbbo(&self, symbol: &str, date: &str) -> Result<Vec<NBBO>> {
		let mut params = NBBOsParams::new().limit(MAX_LIMIT).timestamp(date);
		let mut res = Vec::<NBBO>::new();
		loop {
			let page = self.get_nbbo(symbol, Some(&params.params))?;
			res.extend(page.results.into_iter());
			match page.next_url {
				Some(next_url) => {
					let split = next_url.split("cursor=").collect::<Vec<&str>>();
					if split.len() != 2 {
						let msg = format!("no cursor in next_url {}", next_url);
						let io_error = io::Error::new(io::ErrorKind::UnexpectedEof, msg);
						return Err(Error::IoError(io_error));
					}
					let cursor = split[1];
					params = NBBOsParams::new().cursor(cursor);
				}
				None => break
			};
		}

		Ok(res)
	}
}

#[cfg(test)]
mod nbbo {
	use crate::{client::Client, equities::nbbo::NBBOsParams};

	#[test]
	fn works() {
		let client = Client::new().unwrap();
		let limit = 500;
		let params = NBBOsParams::new()
			.timestamp("2005-01-03")
			.limit(limit)
			.params;
		let nbbo = client.get_nbbo("AAPL", Some(&params)).unwrap();
		assert_eq!(nbbo.results.len(), limit);
	}

	#[test]
	fn get_all_works() {
		let client = Client::new().unwrap();
		let trades = client.get_all_nbbo("AAPL", "2005-01-03").unwrap();
		let count = 58_819;
		assert_eq!(trades.len(), count);
	}
}
