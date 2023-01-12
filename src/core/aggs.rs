extern crate serde_json;
extern crate ureq;

use super::Candle;
use crate::{
	client::{Client, Result},
	helpers::make_params,
	with_param
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Timespan {
	Minute,
	Hour,
	Day,
	Week,
	Month,
	Quarter,
	Year
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AggResponse {
	#[serde(rename(deserialize = "ticker"))]
	pub symbol: String,
	#[serde(rename(deserialize = "queryCount"))]
	pub query_count: usize,
	#[serde(rename(deserialize = "resultsCount"))]
	pub results_count: usize,
	pub adjusted: bool,
	#[serde(default)] // On 2020-12-07 started being omitted instead of empty
	pub results: Vec<Candle>,
	// For debugging
	pub request_id: String,
	pub uri: Option<String>
}

pub struct AggsParams<'a> {
	pub params: HashMap<&'a str, String>
}

impl<'a> AggsParams<'a> {
	with_param!(unadjusted, bool);

	with_param!(sort, &str);

	with_param!(limit, i32);

	pub fn new() -> Self {
		Self {
			params: HashMap::with_capacity(3)
		}
	}
}

impl Client {
	pub fn get_aggs(
		&mut self,
		symbol: &str,
		multiplier: i64,
		timespan: Timespan,
		from: &str,
		to: &str,
		params: Option<&HashMap<&str, String>>
	) -> Result<AggResponse> {
		let uri = format!(
			"{}/v2/aggs/ticker/{}/range/{}/{}/{}/{}{}",
			self.api_uri,
			symbol,
			multiplier,
			format!("{:?}", timespan).to_lowercase(),
			from,
			to,
			make_params(params),
		);
		let mut resp = self.get_response::<AggResponse>(&uri)?;

		let is_equity = !symbol.contains(":");
		let mut min_ts = i64::MAX;
		let mut max_ts = i64::MIN;
		for candle in resp.results.iter_mut() {
			// Polygon returns GMT milliseconds for this endpoint
			if is_equity {
				// Subtract 5h of milliseconds
				candle.ts -= 5 * 60 * 60 * 1_000;
			}
			// Convert to ns
			candle.ts *= 1_000_000;
			if candle.ts > max_ts {
				max_ts = candle.ts;
			}
			if candle.ts < min_ts {
				min_ts = candle.ts;
			}
			// Add symbol
			candle.symbol = resp.symbol.clone();
		}

		Ok(resp)
	}
}

#[cfg(test)]
mod aggs {
	use super::Timespan;
	use crate::{client::Client, core::aggs::AggsParams};

	#[test]
	fn aapl() {
		let mut client = Client::new().unwrap();
		let sym = String::from("AAPL");
		let resp = client
			.get_aggs(&sym, 1, Timespan::Minute, "2020-11-05", "2020-11-05", None)
			.unwrap();
		assert_eq!(resp.results.len(), 941);
		assert_eq!(resp.results.len(), resp.results_count);
		assert_eq!(resp.results.len(), resp.query_count);
		assert_eq!(resp.results[0].symbol, sym);
	}

	#[test]
	fn mac() {
		let mut client = Client::new().unwrap();
		let sym = String::from("MAC");
		let params = AggsParams::new().unadjusted(true).params;
		client
			.get_aggs(
				&sym,
				1,
				Timespan::Minute,
				"2004-01-01",
				"2020-02-01",
				Some(&params)
			)
			.unwrap();
	}
}
