extern crate serde_json;
extern crate ureq;

use super::Candle;
use crate::{client::{Client, Result}, helpers::make_params, with_param};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupedResponse {
	pub query_count: usize,
	pub results_count: usize,
	pub adjusted: bool,
	#[serde(default)] // On 2020-12-07 started being omitted instead of empty
	pub results: Vec<Candle>,
	// For debugging
	pub status: String,
	pub uri: Option<String>
}

#[derive(Debug)]
pub enum Locale {
	Global,
	US,
	GB,
	CA,
	NL,
	GR,
	SP,
	DE,
	BE,
	DK,
	FI,
	IE,
	PT,
	IN,
	MX,
	FR,
	CN,
	CH,
	SE
}

#[derive(Debug)]
pub enum Market {
	Stocks,
	Crypto,
	Bonds,
	MF,
	MMF,
	Indices,
	FX
}

pub struct GroupedParams<'a> {
	pub params: HashMap<&'a str, String>
}

impl<'a> GroupedParams<'a> {
	with_param!(unadjusted, bool);

	pub fn new() -> Self {
		Self {
			params: HashMap::with_capacity(1)
		}
	}
}

impl Client {
	pub fn get_grouped(
		&mut self,
		locale: Locale,
		market: Market,
		date: &str,
		params: Option<&HashMap<&str, String>>
	) -> Result<GroupedResponse> {
		let uri = format!(
			"{}/v2/aggs/grouped/locale/{}/market/{}/{}{}",
			self.api_uri,
			format!("{:?}", locale).to_lowercase(),
			format!("{:?}", market).to_lowercase(),
			date,
			make_params(params),
		);

		let mut resp = self.get_response::<GroupedResponse>(&uri)?;
		resp.uri = Some(uri);

		Ok(resp)
	}
}

#[cfg(test)]
mod grouped {
	use super::{GroupedParams, Locale, Market};
	use crate::client::Client;

	#[test]
	fn start() {
		let mut client = Client::new().unwrap();
		let params = GroupedParams::new().unadjusted(true).params;
		let grouped = client
			.get_grouped(Locale::US, Market::Stocks, "2004-01-02", Some(&params))
			.unwrap();
		assert_eq!(grouped.query_count, grouped.results_count);
		assert_eq!(grouped.query_count, 7670);
		assert_eq!(grouped.results.len(), 7670);
	}

	#[test]
	fn no_bad_ranges() {
		let mut client = Client::new().unwrap();
		let params = GroupedParams::new().unadjusted(true).params;
		for _ in 0..50 {
			client
				.get_grouped(Locale::US, Market::Stocks, "2004-01-02", Some(&params))
				.unwrap();
		}
	}

	#[test]
	fn no_garbage_tickers() {
		let mut client = Client::new().unwrap();
		let bad_dates = vec![
			"2020-04-07",
			"2020-04-08",
			"2020-04-09",
			"2020-04-13",
			"2020-04-14",
		];
		let params = GroupedParams::new().unadjusted(true).params;
		for date in bad_dates {
			let grouped = client
				.get_grouped(Locale::US, Market::Stocks, date, Some(&params))
				.unwrap();
			let mut has_bad_ticker = false;
			for candle in grouped.results {
				if !candle.symbol.chars().all(|c| c.is_ascii_graphic()) {
					eprintln!("Bad ticker {}", candle.symbol);
					has_bad_ticker = true;
				}
			}
			assert!(!has_bad_ticker);
		}
	}

	#[test]
	fn no_missing_vw() {
		let mut client = Client::new().unwrap();
		let bad_dates = vec![
			"2020-04-07",
			"2020-04-08",
			"2020-04-09",
			"2020-04-13",
			"2020-04-14",
		];
		let params = GroupedParams::new().unadjusted(true).params;
		for date in bad_dates {
			let grouped = client
				.get_grouped(Locale::US, Market::Stocks, date, Some(&params))
				.unwrap();
			let mut missing_vwap = false;
			for candle in grouped.results {
				if candle.volume > 0 && candle.vwap.is_nan() {
					eprintln!("Bad ticker {}", candle.symbol);
					missing_vwap = true;
				}
			}
			assert!(!missing_vwap);
		}
	}
}
